//! The mobius-lang intermediate representation.
//!
//! The IR is a *netlist*, not a program: flat, fully resolved, nothing left
//! to look up and no decisions left to make. It is only ever constructed
//! from a composition that already passed every check — the execute stage
//! performs, it never decides. Every entity retains the source [`Span`]
//! that produced it (spans are never dropped), and [`Ir::emit_text`]
//! renders the canonical textual form: greppable, diffable, and
//! snapshot-testable.

use crate::error::Span;
use std::fmt::Write;

/// One elaborated application.
#[derive(Debug, Clone, PartialEq)]
pub struct Ir {
    pub app: String,
    pub values: Vec<IrValue>,
    pub instances: Vec<IrInstance>,
    pub signals: Vec<IrSignal>,
    pub handlers: Vec<IrHandler>,
    pub layout: Vec<IrDock>,
}

/// A party that can hold a binding: a citizen instance or a handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Party {
    Instance(usize),
    Handler(usize),
}

impl Party {
    fn display(&self) -> String {
        match self {
            Party::Instance(id) => format!("i{id}"),
            Party::Handler(id) => format!("h{id}"),
        }
    }
}

/// A shared value — elaborates to one `Dynamic<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct IrValue {
    pub id: usize,
    /// Qualified name, e.g. `bench.amplitude`.
    pub name: String,
    /// Type name as written in source, e.g. `Vec<f32>`.
    pub ty: String,
    /// Default literal as written, e.g. `1.0`, `true`, `[]`.
    pub default: Option<String>,
    /// The single writer, if any — enforced by the one-writer check.
    pub writer: Option<Party>,
    pub readers: Vec<Party>,
    pub span: Span,
}

/// How a party touches a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Writable `Dynamic<T>` clone (`out`).
    Rw,
    /// Observe-only handle (`in`).
    Ro,
}

/// One port binding on an instance or handler.
#[derive(Debug, Clone, PartialEq)]
pub struct IrBinding {
    /// Qualified field path, e.g. `bench.amplitude`.
    pub port: String,
    pub value: usize,
    pub access: Access,
}

/// Where a citizen implementation comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceKind {
    /// Declared in source from widget primitives — interpreted, live.
    Source,
    /// A compiled Rust citizen resolved from the registry.
    Registry,
}

/// One citizen instance.
#[derive(Debug, Clone, PartialEq)]
pub struct IrInstance {
    pub id: usize,
    pub name: String,
    pub ty: String,
    pub kind: InstanceKind,
    pub bindings: Vec<IrBinding>,
    pub span: Span,
}

/// One queued event edge (a `signal` interface field).
#[derive(Debug, Clone, PartialEq)]
pub struct IrSignal {
    pub id: usize,
    /// Qualified name, e.g. `bench.commands`.
    pub name: String,
    /// Event type name, e.g. `BenchCmd`.
    pub ty: String,
    pub emitters: Vec<Party>,
    pub drainer: Option<Party>,
    pub span: Span,
}

/// One registered backend handler bound to a modport view.
#[derive(Debug, Clone, PartialEq)]
pub struct IrHandler {
    pub id: usize,
    pub name: String,
    pub bindings: Vec<IrBinding>,
    /// Signals this handler drains.
    pub drains: Vec<usize>,
    pub span: Span,
}

/// One `dock(...)` layout directive.
#[derive(Debug, Clone, PartialEq)]
pub struct IrDock {
    pub instance: usize,
    pub region: String,
    pub fraction: Option<f64>,
    pub span: Span,
}

impl Ir {
    /// Render the canonical textual form.
    pub fn emit_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "app      {}", self.app);

        let name_width = self.values.iter().map(|v| v.name.len()).max().unwrap_or(0);
        let ty_width = self.values.iter().map(|v| v.ty.len()).max().unwrap_or(0);
        for value in &self.values {
            let _ = write!(
                out,
                "value    v{}  {:<name_width$} : {:<ty_width$}",
                value.id, value.name, value.ty,
            );
            if let Some(default) = &value.default {
                let _ = write!(out, " = {default}");
            }
            if let Some(writer) = &value.writer {
                let _ = write!(out, "  writer={}", writer.display());
            }
            let _ = writeln!(out);
        }

        for instance in &self.instances {
            let kind = match instance.kind {
                InstanceKind::Source => "source",
                InstanceKind::Registry => "registry",
            };
            let _ = write!(
                out,
                "instance i{}  {} : {} ({}) ",
                instance.id, instance.name, instance.ty, kind,
            );
            for binding in &instance.bindings {
                let access = match binding.access {
                    Access::Rw => "rw",
                    Access::Ro => "ro",
                };
                let _ = write!(out, " {}=v{}({})", binding.port, binding.value, access);
            }
            let _ = writeln!(out);
        }

        for signal in &self.signals {
            let emitters: Vec<String> = signal.emitters.iter().map(|p| p.display()).collect();
            let drainer = signal
                .drainer
                .map(|p| p.display())
                .unwrap_or_else(|| "-".to_owned());
            let _ = writeln!(
                out,
                "signal   s{}  {} : {}  emit={}  drain={}",
                signal.id,
                signal.name,
                signal.ty,
                emitters.join(","),
                drainer,
            );
        }

        for handler in &self.handlers {
            let _ = write!(out, "handler  h{}  \"{}\" ", handler.id, handler.name);
            for binding in &handler.bindings {
                let access = match binding.access {
                    Access::Rw => "rw",
                    Access::Ro => "ro",
                };
                let _ = write!(out, " {}=v{}({})", binding.port, binding.value, access);
            }
            let _ = writeln!(out);
        }

        for dock in &self.layout {
            let _ = write!(
                out,
                "layout   dock i{} region={}",
                dock.instance, dock.region
            );
            if let Some(fraction) = dock.fraction {
                let _ = write!(out, " fraction={fraction}");
            }
            let _ = writeln!(out);
        }

        out
    }
}
