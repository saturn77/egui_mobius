//! mobius-lang abstract syntax tree.
//!
//! A minimal, growing core for the composition layer: one `app` holding
//! shared values, interfaces (state + signal fields with modport views),
//! citizens declared from widget primitives, and `@wiring` / `@layout`
//! sections. Types are held as names and resolved during elaboration.

use crate::error::Span;

/// Root of a parsed `.mobius` file.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    pub apps: Vec<App>,
}

/// The composition root: `app Name { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    pub name: String,
    pub items: Vec<AppItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppItem {
    /// `let name : type = default;` — an app-owned shared value.
    Value(ValueDecl),
    /// `interface Name { ... }` — state + signal fields with modports.
    Interface(Interface),
    /// `citizen Name (ports) { ... }` — an interior built from primitives.
    Citizen(CitizenDecl),
    /// `@wiring { ... }` / `@layout { ... }`.
    Section(Section),
}

/// `let name : type = default;`
#[derive(Debug, Clone, PartialEq)]
pub struct ValueDecl {
    pub name: String,
    pub ty: TypeRef,
    pub default: Option<Expr>,
    pub span: Span,
}

/// A named type reference: `f32`, `Vec<f32>`, `Bench.controls`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    pub path: Path,
    pub params: Vec<TypeRef>,
    pub span: Span,
}

/// Dotted / double-colon path: `bench.enabled`, `BenchCmd::Apply`.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

impl Path {
    pub fn joined(&self) -> String {
        self.segments.join(".")
    }
}

/// `interface Name { fields... modports... }` — the complete boundary:
/// shared state (pull edges) and signal fields (push edges).
#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub name: String,
    pub fields: Vec<Field>,
    pub modports: Vec<Modport>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub kind: FieldKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    /// `name : type = default` — elaborates to a `Dynamic<T>`.
    State { ty: TypeRef, default: Option<Expr> },
    /// `name : [type; N] = {..}` — elaborates to N `Dynamic<T>` values
    /// (`name.0` … `name.N-1`). Indexed in source as `name[i]`.
    Array {
        elem: TypeRef,
        len: usize,
        default: Option<ArrayDefault>,
    },
    /// `name : signal Type` — a threaded `Signal`/`Slot` edge.
    Signal { ty: TypeRef },
}

/// The `{..}` initializer of an array field.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayDefault {
    /// `{expr}` — the same value for every element.
    Broadcast(Expr),
    /// `{e0, e1, ...}` — one value per element (count must match `len`).
    Elements(Vec<Expr>),
}

/// `modport name (verb field, ...)` — one party's complete view of the
/// boundary: write (`out`), observe (`in`), produce (`emit`), consume
/// (`drain`).
#[derive(Debug, Clone, PartialEq)]
pub struct Modport {
    pub name: String,
    pub entries: Vec<ModportEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModportEntry {
    pub verb: Verb,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Out,
    In,
    Emit,
    Drain,
}

/// `citizen Name (name : Iface.view, ...) { widgets }` — an input panel
/// declared from widget primitives; pure data, live-reloadable.
#[derive(Debug, Clone, PartialEq)]
pub struct CitizenDecl {
    pub name: String,
    pub ports: Vec<PortParam>,
    pub body: Vec<WidgetNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortParam {
    pub name: String,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetNode {
    /// `column { ... }`, `row { ... }`, `group { ... }`, `grid { ... }`.
    Container {
        kind: String,
        children: Vec<WidgetNode>,
        span: Span,
    },
    /// `for VAR in LO..HI { ... }` — a generate loop, unrolled at lowering.
    /// Inside a `grid`, each pass is one row.
    For {
        var: String,
        lo: i64,
        hi: i64,
        body: Vec<WidgetNode>,
        span: Span,
    },
    /// `checkbox "Enabled" <-> bench.enabled;` and friends.
    Primitive(Box<Primitive>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive {
    pub kind: String,
    pub label: Option<String>,
    /// `lo..hi` for `slider` / `drag`.
    pub range: Option<(Expr, Expr)>,
    pub binding: Binding,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// `<-> path` / `<-> path[i]` — widget edits an `out` port.
    TwoWay { path: Path, index: Option<Expr> },
    /// `<- path` / `<- path[i]` — read-only display of a port.
    Read { path: Path, index: Option<Expr> },
    /// `-> path(args)` — fire into an `emit` signal field.
    Event(Expr),
    /// e.g. `separator;`
    None,
}

/// `@wiring { ... }` / `@layout { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let name = Type(port = value, ...);` — instantiation.
    Instance {
        name: String,
        ty: Path,
        args: Vec<NamedArg>,
        span: Span,
    },
    /// `let name : type = default;` — a section-scoped shared value.
    Value(ValueDecl),
    /// `bind "handler_name" = iface.view;` — hand a modport view to a
    /// registered handler.
    Bind {
        handler: String,
        view: Path,
        span: Span,
    },
    /// `dock(plot, region = center);` — a backend directive.
    Directive {
        name: String,
        args: Vec<DirectiveArg>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedArg {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveArg {
    Positional(Expr),
    Named(NamedArg),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Str(String, Span),
    /// `[]` and `[a, b, c]`.
    Array(Vec<Expr>, Span),
    /// `bench.enabled`, `center`, `BenchCmd::Apply`.
    Path(Path),
    /// `bench.commands.send(BenchCmd::Apply)`.
    Call {
        path: Path,
        args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s)
            | Expr::Float(_, s)
            | Expr::Bool(_, s)
            | Expr::Str(_, s)
            | Expr::Array(_, s) => *s,
            Expr::Path(p) => p.span,
            Expr::Call { span, .. } => *span,
        }
    }
}
