//! Execute: walk the IR and build the live reactive fabric.
//!
//! This is the back half of the pipeline. The IR arrives fully checked, so
//! this stage *performs* and never decides: construct one `Dynamic<T>` per
//! value, one queued channel per signal, expand each party's bindings into
//! typed handles, and pass the layout through to the dock host.
//!
//! The [`Host`] is the single typed doorway between erased source names and
//! concrete Rust types. Value types register with a parser for their
//! default literals; lookups after wiring are checked downcasts. Everything
//! on both sides of that doorway is ordinary typed Rust.
//!
//! Rendering is deliberately absent here: a [`WiredApp`] is the complete
//! reactive fabric of the application — values, routes, and layout — and a
//! UI host consumes it. This headless split is what makes the wiring
//! testable without a window.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use egui_mobius_reactive::Dynamic;

use crate::error::Span;
use crate::ir::{Access, InstanceKind, Ir, IrDock, Party};

/// A wiring failure. The IR is pre-checked, so these are host-vocabulary
/// gaps (an unregistered value type), not composition errors.
#[derive(Debug, Clone)]
pub struct ExecError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span, self.message)
    }
}

impl std::error::Error for ExecError {}

// ----------------------------------------------------------------------
// Erased handles
// ----------------------------------------------------------------------

/// A type-erased `Dynamic<T>`. The checked downcast back to `Dynamic<T>`
/// is the one point where source-level names meet Rust types.
#[derive(Clone)]
pub struct ErasedDynamic {
    type_name: String,
    inner: Arc<dyn Any + Send + Sync>,
}

impl ErasedDynamic {
    fn new<T: Clone + Send + Sync + 'static>(type_name: &str, value: Dynamic<T>) -> Self {
        Self {
            type_name: type_name.to_owned(),
            inner: Arc::new(value),
        }
    }

    /// The source-level type name this value was declared with.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Checked downcast to the concrete reactive value.
    pub fn typed<T: Clone + Send + Sync + 'static>(&self) -> Option<Dynamic<T>> {
        self.inner.downcast_ref::<Dynamic<T>>().cloned()
    }
}

impl std::fmt::Debug for ErasedDynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErasedDynamic<{}>", self.type_name)
    }
}

/// Observe-only view of a shared value (an `in` binding): `get()` and
/// change subscription, no `set()`.
#[derive(Clone)]
pub struct Watch<T: Clone + Send + 'static> {
    inner: Dynamic<T>,
}

impl<T: Clone + Send + 'static> Watch<T> {
    pub fn get(&self) -> T {
        self.inner.get()
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> Watch<T> {
    /// Register a change callback (delegates to `ValueExt::on_change`).
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        use egui_mobius_reactive::ValueExt;
        let _ = self.inner.on_change(callback);
    }
}

/// The producing end of a signal (`emit`). Events are boxed across the
/// erased channel; the event type was checked at elaboration.
#[derive(Clone)]
pub struct EmitHandle {
    signal: String,
    sender: Sender<Box<dyn Any + Send>>,
}

impl EmitHandle {
    pub fn send<E: Any + Send>(&self, event: E) {
        // A closed channel means the drainer is gone (app shutdown).
        let _ = self.sender.send(Box::new(event));
    }

    /// Send an already-boxed event — for generic renderers that construct
    /// events through a registered constructor rather than a static type.
    pub fn send_boxed(&self, event: Box<dyn Any + Send>) {
        let _ = self.sender.send(event);
    }

    pub fn signal_name(&self) -> &str {
        &self.signal
    }
}

/// The consuming end of a signal (`drain`) — exactly one per signal,
/// enforced by elaboration.
pub struct DrainHandle {
    signal: String,
    receiver: Receiver<Box<dyn Any + Send>>,
}

impl DrainHandle {
    /// Take all pending events of type `E`, leaving the queue empty.
    /// Events of the wrong type are a wiring bug and are dropped.
    pub fn drain<E: Any + Send>(&self) -> Vec<E> {
        let mut events = Vec::new();
        while let Ok(boxed) = self.receiver.try_recv() {
            if let Ok(event) = boxed.downcast::<E>() {
                events.push(*event);
            }
        }
        events
    }

    pub fn signal_name(&self) -> &str {
        &self.signal
    }
}

// ----------------------------------------------------------------------
// Host
// ----------------------------------------------------------------------

type ValueFactory = Box<dyn Fn(Option<&str>) -> Result<ErasedDynamic, String>>;

/// The host's typed vocabulary for values: how each source-level type name
/// constructs a `Dynamic<T>` from its default literal.
///
/// `f32`, `bool`, and `String` are built in, plus `Vec<f32>` with the `[]`
/// empty default. Hosts register further types with
/// [`value_type`](Host::value_type).
pub struct Host {
    value_types: HashMap<String, ValueFactory>,
}

impl Host {
    pub fn new() -> Self {
        Host {
            value_types: HashMap::new(),
        }
        .value_type::<f32, _>("f32", |text| text.parse().ok())
        .value_type::<bool, _>("bool", |text| text.parse().ok())
        .value_type::<String, _>("String", |text| Some(text.trim_matches('"').to_owned()))
        .value_type::<Vec<f32>, _>("Vec<f32>", |text| {
            if text.trim() == "[]" {
                Some(Vec::new())
            } else {
                None
            }
        })
    }

    /// Register a value type: its source-level name and a parser for its
    /// default literal. A missing default uses `T::default()`.
    pub fn value_type<T, F>(mut self, name: &str, parse: F) -> Self
    where
        T: Clone + Default + Send + Sync + 'static,
        F: Fn(&str) -> Option<T> + 'static,
    {
        let type_name = name.to_owned();
        self.value_types.insert(
            name.to_owned(),
            Box::new(move |default| {
                let value = match default {
                    None => T::default(),
                    Some(text) => parse(text)
                        .ok_or_else(|| format!("cannot parse `{text}` as `{type_name}`"))?,
                };
                Ok(ErasedDynamic::new(&type_name, Dynamic::new(value)))
            }),
        );
        self
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------
// Wired application
// ----------------------------------------------------------------------

/// One party's expanded bindings: writable clones, observe-only views, and
/// signal endpoints — keyed by the qualified field name from source.
#[derive(Default)]
pub struct BindingSet {
    writes: HashMap<String, ErasedDynamic>,
    reads: HashMap<String, ErasedDynamic>,
    emits: HashMap<String, EmitHandle>,
}

impl BindingSet {
    /// Writable `Dynamic<T>` for an `out` port.
    pub fn write<T: Clone + Send + Sync + 'static>(&self, port: &str) -> Option<Dynamic<T>> {
        self.writes.get(port)?.typed()
    }

    /// Observe-only view for an `in` port.
    pub fn read<T: Clone + Send + Sync + 'static>(&self, port: &str) -> Option<Watch<T>> {
        let inner = self.reads.get(port)?.typed()?;
        Some(Watch { inner })
    }

    /// Producing end of an `emit` signal field.
    pub fn emit(&self, port: &str) -> Option<EmitHandle> {
        self.emits.get(port).cloned()
    }
}

/// A wired citizen instance, ready for a UI host.
pub struct WiredInstance {
    pub name: String,
    pub ty: String,
    pub kind: InstanceKind,
    pub bindings: BindingSet,
    /// Widget tree for source-declared citizens (empty for registry
    /// citizens) — a generic renderer draws these against `bindings`.
    pub widgets: Vec<crate::ir::IrWidget>,
}

/// A wired backend handler: its bindings plus the drain ends it owns.
pub struct WiredHandler {
    pub name: String,
    pub bindings: BindingSet,
    pub drains: Vec<DrainHandle>,
}

/// The complete reactive fabric of one application. A UI host renders the
/// instances and resolves the layout; backend code drives the handlers.
pub struct WiredApp {
    pub app: String,
    values: HashMap<String, ErasedDynamic>,
    pub instances: Vec<WiredInstance>,
    pub handlers: Vec<WiredHandler>,
    pub layout: Vec<IrDock>,
}

impl std::fmt::Debug for WiredApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WiredApp")
            .field("app", &self.app)
            .field("values", &self.values.len())
            .field("instances", &self.instances.len())
            .field("handlers", &self.handlers.len())
            .field("layout", &self.layout.len())
            .finish()
    }
}

impl WiredApp {
    /// Typed access to any shared value by its qualified source name —
    /// the debugger's door into the live graph.
    pub fn value<T: Clone + Send + Sync + 'static>(&self, name: &str) -> Option<Dynamic<T>> {
        self.values.get(name)?.typed()
    }

    /// Every shared value, for graph inspection.
    pub fn values(&self) -> impl Iterator<Item = (&str, &ErasedDynamic)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }
}

/// Walk the IR and build the live fabric.
pub fn wire(ir: &Ir, host: &Host) -> Result<WiredApp, ExecError> {
    // Values: one Dynamic<T> each.
    let mut values = HashMap::new();
    for value in &ir.values {
        let factory = host.value_types.get(&value.ty).ok_or_else(|| ExecError {
            message: format!("value type `{}` is not registered with the host", value.ty),
            span: value.span,
        })?;
        let erased = factory(value.default.as_deref()).map_err(|message| ExecError {
            message,
            span: value.span,
        })?;
        values.insert(value.name.clone(), erased);
    }

    // Bindings per party, expanded from the IR's already-derived routes.
    let mut instance_sets: Vec<BindingSet> =
        ir.instances.iter().map(|_| BindingSet::default()).collect();
    let mut handler_sets: Vec<BindingSet> =
        ir.handlers.iter().map(|_| BindingSet::default()).collect();
    let mut handler_drains: Vec<Vec<DrainHandle>> =
        ir.handlers.iter().map(|_| Vec::new()).collect();

    let mut apply = |party: Party, port: &str, value: &ErasedDynamic, access: Access| {
        let set = match party {
            Party::Instance(id) => &mut instance_sets[id],
            Party::Handler(id) => &mut handler_sets[id],
        };
        match access {
            Access::Rw => set.writes.insert(port.to_owned(), value.clone()),
            Access::Ro => set.reads.insert(port.to_owned(), value.clone()),
        };
    };

    for instance in &ir.instances {
        for binding in &instance.bindings {
            let value = &values[&ir.values[binding.value].name];
            apply(
                Party::Instance(instance.id),
                &binding.port,
                value,
                binding.access,
            );
        }
    }
    for handler in &ir.handlers {
        for binding in &handler.bindings {
            let value = &values[&ir.values[binding.value].name];
            apply(
                Party::Handler(handler.id),
                &binding.port,
                value,
                binding.access,
            );
        }
    }

    // Signals: one queued channel each; emit ends to every emitter, the
    // single drain end to the drainer.
    for signal in &ir.signals {
        let (sender, receiver) = channel();
        for emitter in &signal.emitters {
            let handle = EmitHandle {
                signal: signal.name.clone(),
                sender: sender.clone(),
            };
            match emitter {
                Party::Instance(id) => {
                    instance_sets[*id].emits.insert(signal.name.clone(), handle);
                }
                Party::Handler(id) => {
                    handler_sets[*id].emits.insert(signal.name.clone(), handle);
                }
            }
        }
        if let Some(Party::Handler(id)) = signal.drainer {
            handler_drains[id].push(DrainHandle {
                signal: signal.name.clone(),
                receiver,
            });
        }
        // An instance drainer would land here symmetrically once a
        // citizen-to-citizen route exists in a design.
    }

    let instances = ir
        .instances
        .iter()
        .zip(instance_sets)
        .map(|(instance, bindings)| WiredInstance {
            name: instance.name.clone(),
            ty: instance.ty.clone(),
            kind: instance.kind,
            bindings,
            widgets: instance.widgets.clone(),
        })
        .collect();

    let handlers = ir
        .handlers
        .iter()
        .zip(handler_sets.into_iter().zip(handler_drains))
        .map(|(handler, (bindings, drains))| WiredHandler {
            name: handler.name.clone(),
            bindings,
            drains,
        })
        .collect();

    Ok(WiredApp {
        app: ir.app.clone(),
        values,
        instances,
        handlers,
        layout: ir.layout.clone(),
    })
}
