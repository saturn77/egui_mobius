//! Lowering: AST → IR.
//!
//! Implements the middle of the pipeline: *resolve* names against the
//! [`Registry`], *check* (modport verbs, the one-writer rule, dangling
//! signals, unknown references), and *lower* to the flat [`Ir`] netlist.
//! Validation completes before the IR exists; every diagnostic carries the
//! source span that caused it, with related spans where two sites conflict
//! (e.g. both writers of a twice-written value).

use std::collections::HashMap;

use crate::ast::*;
use crate::error::Span;
use crate::ir::*;

/// The host-provided vocabulary: compiled-in citizen types, registered
/// handlers, and event types. The registry is the single typed doorway
/// between source and Rust; v0 records names only — port tables and type
/// checking against real Rust types arrive with the execute stage.
#[derive(Debug, Default, Clone)]
pub struct Registry {
    citizens: Vec<String>,
    handlers: Vec<String>,
    events: Vec<String>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a compiled-in citizen type name (e.g. `"PlotPanel"`).
    pub fn citizen(mut self, name: impl Into<String>) -> Self {
        self.citizens.push(name.into());
        self
    }

    /// Register a backend handler name (e.g. `"bench_worker"`).
    pub fn handler(mut self, name: impl Into<String>) -> Self {
        self.handlers.push(name.into());
        self
    }

    /// Register an event type name (e.g. `"BenchCmd"`).
    pub fn event(mut self, name: impl Into<String>) -> Self {
        self.events.push(name.into());
        self
    }

    fn has_citizen(&self, name: &str) -> bool {
        self.citizens.iter().any(|c| c == name)
    }

    fn has_handler(&self, name: &str) -> bool {
        self.handlers.iter().any(|h| h == name)
    }

    fn has_event(&self, name: &str) -> bool {
        self.events.iter().any(|e| e == name)
    }
}

/// An elaboration failure, anchored to the offending span, with related
/// sites where a conflict has two ends.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub related: Vec<(String, Span)>,
}

impl Diagnostic {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span, self.message)?;
        for (note, span) in &self.related {
            write!(f, "\n  {span}: {note}")?;
        }
        Ok(())
    }
}

/// Lower one application to IR against the host registry.
pub fn lower(app: &App, registry: &Registry) -> Result<Ir, Vec<Diagnostic>> {
    Lowerer::new(app, registry).run()
}

struct Lowerer<'a> {
    app: &'a App,
    registry: &'a Registry,
    interfaces: HashMap<&'a str, &'a Interface>,
    source_citizens: HashMap<&'a str, &'a CitizenDecl>,
    /// interface-instance name → interface name
    iface_instances: HashMap<String, &'a str>,
    /// qualified value name (`bench.amplitude`) → value id
    value_ids: HashMap<String, usize>,
    /// qualified signal name (`bench.commands`) → signal id
    signal_ids: HashMap<String, usize>,
    /// citizen-instance name → instance id
    instance_ids: HashMap<String, usize>,
    ir: Ir,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lowerer<'a> {
    fn new(app: &'a App, registry: &'a Registry) -> Self {
        Self {
            app,
            registry,
            interfaces: HashMap::new(),
            source_citizens: HashMap::new(),
            iface_instances: HashMap::new(),
            value_ids: HashMap::new(),
            signal_ids: HashMap::new(),
            instance_ids: HashMap::new(),
            ir: Ir {
                app: app.name.clone(),
                values: Vec::new(),
                instances: Vec::new(),
                signals: Vec::new(),
                handlers: Vec::new(),
                layout: Vec::new(),
            },
            diagnostics: Vec::new(),
        }
    }

    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic::new(message, span));
    }

    fn run(mut self) -> Result<Ir, Vec<Diagnostic>> {
        // Pass 1: collect declarations.
        for item in &self.app.items {
            match item {
                AppItem::Interface(interface) => {
                    self.interfaces.insert(interface.name.as_str(), interface);
                }
                AppItem::Citizen(citizen) => {
                    self.source_citizens.insert(citizen.name.as_str(), citizen);
                }
                _ => {}
            }
        }

        // Pass 2: validate declarations that stand alone.
        for interface in self.interfaces.values() {
            check_interface(interface, &mut self.diagnostics);
        }
        let citizens: Vec<&CitizenDecl> = self.source_citizens.values().copied().collect();
        for citizen in citizens {
            self.check_source_citizen(citizen);
        }

        // Pass 3: walk sections in order.
        for item in &self.app.items {
            match item {
                AppItem::Value(value) => self.lower_app_value(value),
                AppItem::Section(section) => self.lower_section(section),
                _ => {}
            }
        }

        // Pass 4: whole-graph checks.
        self.check_signals();

        if self.diagnostics.is_empty() {
            Ok(self.ir)
        } else {
            Err(self.diagnostics)
        }
    }

    // ------------------------------------------------------------------
    // Declaration checks
    // ------------------------------------------------------------------

    /// Widget bindings in a source citizen must respect the modport verbs
    /// of the port they go through.
    fn check_source_citizen(&mut self, citizen: &'a CitizenDecl) {
        let mut ports: HashMap<&str, &Modport> = HashMap::new();
        for port in &citizen.ports {
            let segments = &port.ty.path.segments;
            if segments.len() != 2 {
                self.error(
                    format!(
                        "port `{}` must be typed by a modport view (`Interface.view`)",
                        port.name
                    ),
                    port.span,
                );
                continue;
            }
            let Some(interface) = self.interfaces.get(segments[0].as_str()) else {
                self.error(format!("unknown interface `{}`", segments[0]), port.span);
                continue;
            };
            let Some(modport) = interface.modports.iter().find(|m| m.name == segments[1]) else {
                self.error(
                    format!(
                        "interface `{}` has no modport `{}`",
                        segments[0], segments[1]
                    ),
                    port.span,
                );
                continue;
            };
            ports.insert(port.name.as_str(), modport);
        }
        for node in &citizen.body {
            self.check_widget(node, &ports);
        }
    }

    fn check_widget(&mut self, node: &WidgetNode, ports: &HashMap<&str, &Modport>) {
        match node {
            WidgetNode::Container { children, .. } => {
                for child in children {
                    self.check_widget(child, ports);
                }
            }
            WidgetNode::Primitive(primitive) => match &primitive.binding {
                Binding::TwoWay(path) => {
                    self.check_port_access(path, ports, Verb::Out, "`<->`", primitive.span);
                }
                Binding::Read(path) => {
                    self.check_port_access(path, ports, Verb::In, "`<-`", primitive.span);
                }
                Binding::Event(expr) => self.check_event(expr, ports),
                Binding::None => {}
            },
        }
    }

    fn check_port_access(
        &mut self,
        path: &Path,
        ports: &HashMap<&str, &Modport>,
        needed: Verb,
        arrow: &str,
        span: Span,
    ) {
        if path.segments.len() != 2 {
            self.error(
                format!("binding must be `port.field`, found `{}`", path.joined()),
                span,
            );
            return;
        }
        let Some(modport) = ports.get(path.segments[0].as_str()) else {
            self.error(format!("unknown port `{}`", path.segments[0]), span);
            return;
        };
        let field = path.segments[1].as_str();
        let entry = modport.entries.iter().find(|e| e.field == field);
        match entry {
            None => self.error(
                format!("modport `{}` has no field `{}`", modport.name, field),
                span,
            ),
            Some(entry)
                if entry.verb != needed && !(needed == Verb::In && entry.verb == Verb::Out) =>
            {
                self.error(
                    format!(
                        "{arrow} needs `{}` on `{}`, but modport `{}` marks it `{}`",
                        verb_name(needed),
                        field,
                        modport.name,
                        verb_name(entry.verb),
                    ),
                    span,
                );
            }
            _ => {}
        }
    }

    /// `-> port.signal_field.send(EventType::Variant)`
    fn check_event(&mut self, expr: &Expr, ports: &HashMap<&str, &Modport>) {
        let Expr::Call { path, args, span } = expr else {
            self.error("`->` must call `port.signal.send(...)`", expr.span());
            return;
        };
        if path.segments.len() != 3 || path.segments[2] != "send" {
            self.error(
                format!(
                    "`->` must be `port.signal.send(...)`, found `{}`",
                    path.joined()
                ),
                *span,
            );
            return;
        }
        let Some(modport) = ports.get(path.segments[0].as_str()) else {
            self.error(format!("unknown port `{}`", path.segments[0]), *span);
            return;
        };
        let field = path.segments[1].as_str();
        match modport.entries.iter().find(|e| e.field == field) {
            None => self.error(
                format!("modport `{}` has no field `{}`", modport.name, field),
                *span,
            ),
            Some(entry) if entry.verb != Verb::Emit => self.error(
                format!(
                    "firing into `{}` needs `emit`, but modport `{}` marks it `{}`",
                    field,
                    modport.name,
                    verb_name(entry.verb),
                ),
                *span,
            ),
            _ => {}
        }
        if let Some(Expr::Path(event)) = args.first() {
            if let Some(event_ty) = event.segments.first() {
                if !self.registry.has_event(event_ty) {
                    let message = format!("unknown event type `{event_ty}`");
                    self.error(message, event.span);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Section lowering
    // ------------------------------------------------------------------

    fn lower_app_value(&mut self, value: &ValueDecl) {
        let id = self.ir.values.len();
        self.value_ids.insert(value.name.clone(), id);
        self.ir.values.push(IrValue {
            id,
            name: value.name.clone(),
            ty: type_name(&value.ty),
            default: value.default.as_ref().map(expr_text),
            writer: None,
            readers: Vec::new(),
            span: value.span,
        });
    }

    fn lower_section(&mut self, section: &Section) {
        for stmt in &section.stmts {
            match stmt {
                Stmt::Value(value) => self.lower_app_value(value),
                Stmt::Instance {
                    name,
                    ty,
                    args,
                    span,
                } => self.lower_instance(name, ty, args, *span),
                Stmt::Bind {
                    handler,
                    view,
                    span,
                } => self.lower_bind(handler, view, *span),
                Stmt::Directive { name, args, span } => {
                    if name == "dock" {
                        self.lower_dock(args, *span);
                    } else {
                        self.error(format!("unknown directive `{name}`"), *span);
                    }
                }
            }
        }
    }

    fn lower_instance(&mut self, name: &str, ty: &Path, args: &[NamedArg], span: Span) {
        let ty_name = ty.segments.join("::");

        // An interface instantiation mints the shared values and signals.
        if let Some(interface) = self.interfaces.get(ty_name.as_str()).copied() {
            self.iface_instances
                .insert(name.to_owned(), interface.name.as_str());
            for field in &interface.fields {
                let qualified = format!("{name}.{}", field.name);
                match &field.kind {
                    FieldKind::State { ty, default } => {
                        let id = self.ir.values.len();
                        self.value_ids.insert(qualified.clone(), id);
                        self.ir.values.push(IrValue {
                            id,
                            name: qualified,
                            ty: type_name(ty),
                            default: default.as_ref().map(expr_text),
                            writer: None,
                            readers: Vec::new(),
                            span: field.span,
                        });
                    }
                    FieldKind::Signal { ty } => {
                        let id = self.ir.signals.len();
                        self.signal_ids.insert(qualified.clone(), id);
                        self.ir.signals.push(IrSignal {
                            id,
                            name: qualified,
                            ty: type_name(ty),
                            emitters: Vec::new(),
                            drainer: None,
                            span: field.span,
                        });
                    }
                }
            }
            return;
        }

        // A citizen instantiation: source-declared or registry.
        let kind = if self.source_citizens.contains_key(ty_name.as_str()) {
            InstanceKind::Source
        } else if self.registry.has_citizen(&ty_name) {
            InstanceKind::Registry
        } else {
            self.error(
                format!(
                    "unknown citizen type `{ty_name}` (not declared in source, not in the registry)"
                ),
                ty.span,
            );
            return;
        };

        let id = self.ir.instances.len();
        self.instance_ids.insert(name.to_owned(), id);
        let mut bindings = Vec::new();
        for arg in args {
            let Expr::Path(view) = &arg.value else {
                self.error(
                    format!(
                        "argument `{}` must be a modport view (`instance.view`)",
                        arg.name
                    ),
                    arg.span,
                );
                continue;
            };
            self.apply_view(Party::Instance(id), view, &mut bindings);
        }
        self.ir.instances.push(IrInstance {
            id,
            name: name.to_owned(),
            ty: ty_name,
            kind,
            bindings,
            span,
        });
    }

    /// Expand a modport view (`bench.controls`) into bindings for `party`.
    fn apply_view(&mut self, party: Party, view: &Path, bindings: &mut Vec<IrBinding>) {
        if view.segments.len() != 2 {
            self.error(
                format!("expected `instance.view`, found `{}`", view.joined()),
                view.span,
            );
            return;
        }
        let instance = view.segments[0].as_str();
        let Some(interface_name) = self.iface_instances.get(instance).copied() else {
            self.error(
                format!("unknown interface instance `{instance}`"),
                view.span,
            );
            return;
        };
        let interface = self.interfaces[interface_name];
        let Some(modport) = interface
            .modports
            .iter()
            .find(|m| m.name == view.segments[1])
        else {
            self.error(
                format!(
                    "interface `{interface_name}` has no modport `{}`",
                    view.segments[1]
                ),
                view.span,
            );
            return;
        };

        for entry in &modport.entries {
            let qualified = format!("{instance}.{}", entry.field);
            match entry.verb {
                Verb::Out | Verb::In => {
                    let Some(&value_id) = self.value_ids.get(&qualified) else {
                        continue; // field/modport mismatch reported by check_interface
                    };
                    if entry.verb == Verb::Out {
                        let value = &mut self.ir.values[value_id];
                        if let Some(existing) = value.writer {
                            let value_span = value.span;
                            let message = format!(
                                "one-writer rule: `{qualified}` already has writer {}",
                                existing.display_name(),
                            );
                            self.diagnostics.push(Diagnostic {
                                message,
                                span: view.span,
                                related: vec![(format!("`{qualified}` declared here"), value_span)],
                            });
                        } else {
                            value.writer = Some(party);
                        }
                        bindings.push(IrBinding {
                            port: qualified,
                            value: value_id,
                            access: Access::Rw,
                        });
                    } else {
                        self.ir.values[value_id].readers.push(party);
                        bindings.push(IrBinding {
                            port: qualified,
                            value: value_id,
                            access: Access::Ro,
                        });
                    }
                }
                Verb::Emit => {
                    if let Some(&signal_id) = self.signal_ids.get(&qualified) {
                        self.ir.signals[signal_id].emitters.push(party);
                    }
                }
                Verb::Drain => {
                    if let Some(&signal_id) = self.signal_ids.get(&qualified) {
                        let signal = &mut self.ir.signals[signal_id];
                        if let Some(existing) = signal.drainer {
                            let signal_span = signal.span;
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "signal `{qualified}` already drained by {}",
                                    existing.display_name(),
                                ),
                                span: view.span,
                                related: vec![(
                                    format!("`{qualified}` declared here"),
                                    signal_span,
                                )],
                            });
                        } else {
                            signal.drainer = Some(party);
                        }
                        if let Party::Handler(handler_id) = party {
                            self.ir.handlers[handler_id].drains.push(signal_id);
                        }
                    }
                }
            }
        }
    }

    fn lower_bind(&mut self, handler: &str, view: &Path, span: Span) {
        if !self.registry.has_handler(handler) {
            self.error(
                format!("unknown handler `{handler}` (not registered)"),
                span,
            );
            return;
        }
        let id = self.ir.handlers.len();
        self.ir.handlers.push(IrHandler {
            id,
            name: handler.to_owned(),
            bindings: Vec::new(),
            drains: Vec::new(),
            span,
        });
        let mut bindings = Vec::new();
        self.apply_view(Party::Handler(id), view, &mut bindings);
        self.ir.handlers[id].bindings = bindings;
    }

    fn lower_dock(&mut self, args: &[DirectiveArg], span: Span) {
        let mut instance = None;
        let mut region = None;
        let mut fraction = None;
        for arg in args {
            match arg {
                DirectiveArg::Positional(Expr::Path(path)) if path.segments.len() == 1 => {
                    match self.instance_ids.get(path.segments[0].as_str()) {
                        Some(&id) => instance = Some(id),
                        None => self.error(
                            format!("unknown citizen instance `{}`", path.segments[0]),
                            path.span,
                        ),
                    }
                }
                DirectiveArg::Positional(expr) => {
                    self.error("dock expects a citizen instance name", expr.span());
                }
                DirectiveArg::Named(named) => match named.name.as_str() {
                    "region" => match &named.value {
                        Expr::Path(path) if path.segments.len() == 1 => {
                            region = Some(path.segments[0].clone());
                        }
                        other => self.error("region expects a name (e.g. `center`)", other.span()),
                    },
                    "fraction" => match named.value {
                        Expr::Float(value, _) => fraction = Some(value),
                        _ => self.error("fraction expects a float", named.value.span()),
                    },
                    other => self.error(format!("unknown dock argument `{other}`"), named.span),
                },
            }
        }
        if let (Some(instance), Some(region)) = (instance, region) {
            self.ir.layout.push(IrDock {
                instance,
                region,
                fraction,
                span,
            });
        } else {
            self.error("dock requires an instance and a region", span);
        }
    }

    // ------------------------------------------------------------------
    // Whole-graph checks
    // ------------------------------------------------------------------

    fn check_signals(&mut self) {
        let mut errors = Vec::new();
        for signal in &self.ir.signals {
            if !signal.emitters.is_empty() && signal.drainer.is_none() {
                errors.push(Diagnostic::new(
                    format!(
                        "dangling signal: `{}` has emitters but nothing drains it",
                        signal.name
                    ),
                    signal.span,
                ));
            }
        }
        self.diagnostics.extend(errors);
    }
}

/// Modport verbs must reference declared fields, with state verbs on state
/// fields and queue verbs on signal fields.
fn check_interface(interface: &Interface, diagnostics: &mut Vec<Diagnostic>) {
    for modport in &interface.modports {
        for entry in &modport.entries {
            let Some(field) = interface.fields.iter().find(|f| f.name == entry.field) else {
                diagnostics.push(Diagnostic::new(
                    format!(
                        "modport `{}` references unknown field `{}`",
                        modport.name, entry.field
                    ),
                    entry.span,
                ));
                continue;
            };
            let ok = match field.kind {
                FieldKind::State { .. } => matches!(entry.verb, Verb::Out | Verb::In),
                FieldKind::Signal { .. } => matches!(entry.verb, Verb::Emit | Verb::Drain),
            };
            if !ok {
                diagnostics.push(Diagnostic::new(
                    format!(
                        "verb `{}` does not apply to field `{}` (state takes out/in, signals take emit/drain)",
                        verb_name(entry.verb),
                        entry.field
                    ),
                    entry.span,
                ));
            }
        }
    }
}

impl Party {
    fn display_name(&self) -> String {
        match self {
            Party::Instance(id) => format!("instance i{id}"),
            Party::Handler(id) => format!("handler h{id}"),
        }
    }
}

fn verb_name(verb: Verb) -> &'static str {
    match verb {
        Verb::Out => "out",
        Verb::In => "in",
        Verb::Emit => "emit",
        Verb::Drain => "drain",
    }
}

fn type_name(ty: &TypeRef) -> String {
    let mut name = ty.path.segments.join("::");
    if !ty.params.is_empty() {
        let params: Vec<String> = ty.params.iter().map(type_name).collect();
        name.push('<');
        name.push_str(&params.join(", "));
        name.push('>');
    }
    name
}

fn expr_text(expr: &Expr) -> String {
    match expr {
        Expr::Int(value, _) => value.to_string(),
        Expr::Float(value, _) => {
            if value.fract() == 0.0 {
                format!("{value:.1}")
            } else {
                value.to_string()
            }
        }
        Expr::Bool(value, _) => value.to_string(),
        Expr::Str(value, _) => format!("\"{value}\""),
        Expr::Array(items, _) => {
            let inner: Vec<String> = items.iter().map(expr_text).collect();
            format!("[{}]", inner.join(", "))
        }
        Expr::Path(path) => path.segments.join("."),
        Expr::Call { path, args, .. } => {
            let inner: Vec<String> = args.iter().map(expr_text).collect();
            format!("{}({})", path.segments.join("."), inner.join(", "))
        }
    }
}
