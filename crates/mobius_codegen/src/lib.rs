//! # mobius_codegen
//!
//! Generate a compilable, idiomatic `egui_mobius` cargo project from a
//! `.mobius` source file — the scaffolding back end to `mobius_lang`'s
//! front end (which parses the same sources for runtime interpretation).
//!
//! The generated project follows the egui_mobius book's architecture:
//!
//! - interface **state fields** become `Dynamic<T>` cells in a shared
//!   boundary struct (Path A),
//! - interface **signal fields** become a typed outbox on the emitting
//!   citizen, drained once per frame into an `actions` routing stub
//!   (Path B),
//! - each **citizen** becomes a `citizen_panel!` module registered with
//!   the lifecycle `Registry`,
//! - `@layout` directives place citizens (`top(...)`) and declare chrome
//!   (`about(...)`).
//!
//! Output is scaffolding: generate once, then edit by hand. The emitted
//! code is meant to read like the hand-written citizens in CopperTrace.
//!
//! ## v0 limits
//!
//! One `app`, one `interface`, one `citizen`. Container kinds `row`,
//! `column`, `right`; primitive kinds `wordmark`, `separator`, `label`,
//! `clock`, `button`, `combo`. Array and modport declarations parse but
//! modport direction checking is not enforced here (that's `lower`'s
//! one-writer check, which this crate does not yet run).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use mobius_lang::ast::{
    App, AppItem, Binding, CitizenDecl, DirectiveArg, Expr, FieldKind, Stmt, WidgetNode,
};

/// Pinned for the generated project; tracks the workspace's egui line.
const EGUI_VERSION: &str = "0.36";

/// Codegen options.
pub struct Options {
    /// An `egui_mobius` checkout; the generated `Cargo.toml` path-depends
    /// on `crates/egui_citizen` and `crates/egui_mobius_reactive` under it.
    pub mobius_root: PathBuf,
}

/// A generated project: relative path → file contents.
pub struct Generated {
    pub crate_name: String,
    pub files: BTreeMap<String, String>,
}

impl Generated {
    /// Write every file under `dir`, creating directories as needed.
    pub fn write_to(&self, dir: &Path) -> std::io::Result<()> {
        for (rel, contents) in &self.files {
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, contents)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Parse(mobius_lang::ParseError),
    /// A construct the generator does not handle (yet) — carries a
    /// human-readable description of what and where.
    Unsupported(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {e:?}"),
            Error::Unsupported(what) => write!(f, "unsupported: {what}"),
        }
    }
}

impl std::error::Error for Error {}

/// Parse `.mobius` source and generate the project files.
pub fn generate(source: &str, opts: &Options) -> Result<Generated, Error> {
    let (file, _comments) = mobius_lang::parse(source).map_err(Error::Parse)?;
    let app = file
        .apps
        .first()
        .ok_or_else(|| Error::Unsupported("no `app` in source".into()))?;
    let model = Model::build(app)?;

    let mut files = BTreeMap::new();
    files.insert("Cargo.toml".into(), model.emit_manifest(opts));
    files.insert("src/state.rs".into(), model.emit_state());
    files.insert("src/citizens/mod.rs".into(), model.emit_citizens_mod());
    files.insert(
        format!("src/citizens/{}.rs", model.citizen_snake),
        model.emit_citizen()?,
    );
    files.insert("src/main.rs".into(), model.emit_main());

    Ok(Generated {
        crate_name: model.crate_name.clone(),
        files,
    })
}

// ── The resolved model ──────────────────────────────────────────────────

struct EnumModel {
    name: String,
    variants: Vec<String>,
    /// Bound by a `combo` — gets `ALL` and `label()`.
    combo: bool,
    /// The app's theme enum — gets the `visuals()` stub.
    visuals: bool,
}

struct FieldModel {
    name: String,
    rust_ty: String,
    default: String,
    enum_name: Option<String>,
}

struct Model {
    app_name: String,
    crate_name: String,
    enums: Vec<EnumModel>,
    iface_name: String,
    iface_instance: String,
    state_fields: Vec<FieldModel>,
    /// `(field_name, payload_enum)` of the single signal field, if any.
    signal: Option<(String, String)>,
    citizen_ty: String,
    citizen_snake: String,
    citizen_port: String,
    body: Vec<WidgetNode>,
    about: Option<AboutModel>,
    /// Interface field a `clock` primitive reads — the host writes it
    /// per frame.
    clock_field: Option<String>,
    /// Combo-bound enum field named `theme` → the host applies visuals.
    theme_field: Option<String>,
}

struct AboutModel {
    trigger_field: String,
    title: String,
    tagline: String,
}

impl Model {
    fn build(app: &App) -> Result<Model, Error> {
        let mut enums: Vec<EnumModel> = Vec::new();
        let mut iface = None;
        let mut citizens: Vec<&CitizenDecl> = Vec::new();
        let mut wiring: Vec<&Stmt> = Vec::new();
        let mut layout: Vec<&Stmt> = Vec::new();

        for item in &app.items {
            match item {
                AppItem::Enum(e) => enums.push(EnumModel {
                    name: e.name.clone(),
                    variants: e.variants.clone(),
                    combo: false,
                    visuals: false,
                }),
                AppItem::Interface(i) => {
                    if iface.replace(i).is_some() {
                        return Err(Error::Unsupported("multiple interfaces (v0 handles one)".into()));
                    }
                }
                AppItem::Citizen(c) => citizens.push(c),
                AppItem::Section(s) => match s.name.as_str() {
                    "wiring" => wiring.extend(&s.stmts),
                    "layout" => layout.extend(&s.stmts),
                    other => {
                        return Err(Error::Unsupported(format!("section `@{other}`")));
                    }
                },
                AppItem::Value(_) | AppItem::Style(_) => {
                    return Err(Error::Unsupported(
                        "app-level `let` / `@style` (v0 skips them)".into(),
                    ));
                }
            }
        }

        let iface = iface.ok_or_else(|| Error::Unsupported("no `interface`".into()))?;
        let [citizen]: [&CitizenDecl; 1] = citizens
            .try_into()
            .map_err(|_| Error::Unsupported("exactly one citizen (v0)".into()))?;

        // Interface fields → Path A cells + the Path B signal.
        let mut state_fields = Vec::new();
        let mut signal = None;
        for field in &iface.fields {
            match &field.kind {
                FieldKind::State { ty, default } => {
                    let ty_name = ty.path.joined();
                    let enum_name = enums.iter().find(|e| e.name == ty_name).map(|e| e.name.clone());
                    state_fields.push(FieldModel {
                        name: field.name.clone(),
                        rust_ty: rust_type(&ty_name),
                        default: render_default(&field.name, default.as_ref())?,
                        enum_name,
                    });
                }
                FieldKind::Signal { ty } => {
                    if signal
                        .replace((field.name.clone(), ty.path.joined()))
                        .is_some()
                    {
                        return Err(Error::Unsupported("multiple signal fields (v0)".into()));
                    }
                }
                FieldKind::Array { .. } => {
                    return Err(Error::Unsupported(format!("array field `{}` (v0)", field.name)));
                }
            }
        }

        // Wiring: instance names for the interface and the citizen.
        let mut iface_instance = None;
        for stmt in &wiring {
            if let Stmt::Instance { name, ty, .. } = stmt {
                if ty.joined() == iface.name {
                    iface_instance = Some(name.clone());
                }
            }
        }
        let iface_instance = iface_instance
            .ok_or_else(|| Error::Unsupported(format!("`{}` never instantiated in @wiring", iface.name)))?;

        // Layout directives.
        let mut about = None;
        for stmt in &layout {
            if let Stmt::Directive { name, args, .. } = stmt {
                match name.as_str() {
                    // v0: the single citizen is the top bar; the directive
                    // is accepted and checked for presence only.
                    "top" => {}
                    "about" => about = Some(parse_about(args, &iface_instance)?),
                    other => {
                        return Err(Error::Unsupported(format!("@layout directive `{other}`")));
                    }
                }
            }
        }

        let port = citizen
            .ports
            .first()
            .ok_or_else(|| Error::Unsupported(format!("citizen `{}` has no port", citizen.name)))?;

        // Scan the widget tree for primitives that put obligations on the
        // host (clock writer, theme visuals) or on the enums (combo).
        let mut clock_field = None;
        let mut theme_field = None;
        scan_widgets(&citizen.body, &mut |p: &mobius_lang::ast::Primitive| {
            match (p.kind.as_str(), &p.binding) {
                ("clock", Binding::Read { path, .. }) => {
                    clock_field = path.segments.last().cloned();
                }
                ("combo", Binding::TwoWay { path, .. }) => {
                    let field_name = path.segments.last().cloned().unwrap_or_default();
                    if let Some(f) = state_fields.iter().find(|f| f.name == field_name) {
                        if let Some(enum_name) = &f.enum_name {
                            if let Some(e) = enums.iter_mut().find(|e| &e.name == enum_name) {
                                e.combo = true;
                                if field_name == "theme" {
                                    e.visuals = true;
                                    theme_field = Some(field_name.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });

        Ok(Model {
            app_name: app.name.clone(),
            crate_name: snake_case(&app.name),
            enums,
            iface_name: iface.name.clone(),
            iface_instance,
            state_fields,
            signal,
            citizen_ty: citizen.name.clone(),
            citizen_snake: snake_case(&citizen.name),
            citizen_port: port.name.clone(),
            body: citizen.body.clone(),
            about,
            clock_field,
            theme_field,
        })
    }

    // ── Cargo.toml ──────────────────────────────────────────────────────

    fn emit_manifest(&self, opts: &Options) -> String {
        let citizen_dep = opts.mobius_root.join("crates/egui_citizen");
        let reactive_dep = opts.mobius_root.join("crates/egui_mobius_reactive");
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
eframe = "{egui}"
egui = "{egui}"
egui_citizen = {{ path = "{citizen}" }}
egui_mobius_reactive = {{ path = "{reactive}" }}

# Standalone — never a member of an enclosing workspace.
[workspace]
"#,
            name = self.crate_name,
            egui = EGUI_VERSION,
            citizen = citizen_dep.display(),
            reactive = reactive_dep.display(),
        )
    }

    // ── src/state.rs ────────────────────────────────────────────────────

    fn emit_state(&self) -> String {
        let mut out = String::new();
        let _ = write!(
            out,
            "//! Shared state for {app} — generated by mobius_codegen.\n\
             //!\n\
             //! `{iface}` is the app's Path A boundary: one `Dynamic<T>` cell per\n\
             //! interface state field. Citizens hold `&{iface}` and read / write\n\
             //! the cells; nothing here is passed `&mut`.\n\n\
             use egui_mobius_reactive::Dynamic;\n",
            app = self.app_name,
            iface = self.iface_name,
        );

        for e in &self.enums {
            let _ = write!(
                out,
                "\n/// `enum {name}` from the `.mobius` source.\n\
                 #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n\
                 pub enum {name} {{\n",
                name = e.name,
            );
            for v in &e.variants {
                let _ = writeln!(out, "    {v},");
            }
            out.push_str("}\n");

            if e.combo || e.visuals {
                let _ = write!(out, "\nimpl {} {{\n", e.name);
                if e.combo {
                    let all = e
                        .variants
                        .iter()
                        .map(|v| format!("{}::{v}", e.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let _ = write!(
                        out,
                        "    pub const ALL: [{name}; {n}] = [{all}];\n\n\
                         \x20   pub fn label(self) -> &'static str {{\n        match self {{\n",
                        name = e.name,
                        n = e.variants.len(),
                    );
                    for v in &e.variants {
                        let _ = writeln!(out, "            {}::{v} => \"{v}\",", e.name);
                    }
                    out.push_str("        }\n    }\n");
                }
                if e.visuals {
                    let light_arms: String = e
                        .variants
                        .iter()
                        .filter(|v| v.contains("Light"))
                        .map(|v| format!("            {}::{v} => egui::Visuals::light(),\n", e.name))
                        .collect();
                    let _ = write!(
                        out,
                        "\n    /// Theme stub: `Light`-named variants map to light visuals,\n\
                         \x20   /// everything else to dark. Replace with real per-variant styles.\n\
                         \x20   pub fn visuals(self) -> egui::Visuals {{\n\
                         \x20       match self {{\n{light_arms}\
                         \x20           _ => egui::Visuals::dark(),\n\
                         \x20       }}\n\
                         \x20   }}\n",
                    );
                }
                out.push_str("}\n");
            }
        }

        let _ = write!(
            out,
            "\n/// Path A cells of the `{iface}` interface.\n\
             #[derive(Clone)]\n\
             pub struct {iface} {{\n",
            iface = self.iface_name,
        );
        for f in &self.state_fields {
            let _ = writeln!(out, "    pub {}: Dynamic<{}>,", f.name, f.rust_ty);
        }
        let _ = write!(
            out,
            "}}\n\nimpl {iface} {{\n    pub fn new() -> Self {{\n        Self {{\n",
            iface = self.iface_name,
        );
        for f in &self.state_fields {
            let _ = writeln!(out, "            {}: Dynamic::new({}),", f.name, f.default);
        }
        out.push_str("        }\n    }\n}\n");
        out
    }

    // ── src/citizens/ ───────────────────────────────────────────────────

    fn emit_citizens_mod(&self) -> String {
        format!(
            "//! Citizen modules — generated by mobius_codegen.\n\n\
             pub mod {snake};\n\npub use {snake}::{ty};\n",
            snake = self.citizen_snake,
            ty = self.citizen_ty,
        )
    }

    fn emit_citizen(&self) -> Result<String, Error> {
        let mut uses = vec![self.iface_name.clone()];
        if let Some((_, payload)) = &self.signal {
            uses.push(payload.clone());
        }
        for e in &self.enums {
            if e.combo && !uses.contains(&e.name) {
                uses.push(e.name.clone());
            }
        }
        uses.sort();

        let mut out = format!(
            "//! The {ty} citizen — generated by mobius_codegen.\n\n\
             use egui_citizen::citizen_panel;\n\n\
             use crate::state::{{{uses}}};\n\n",
            ty = self.citizen_ty,
            uses = uses.join(", "),
        );

        if let Some((signal_name, payload)) = &self.signal {
            let _ = write!(
                out,
                "citizen_panel!({ty}, \"{id}\",\n\
                 \x20   // Path B outbox for the `{signal_name}` signal — drained by the\n\
                 \x20   // app once per frame.\n\
                 \x20   outbox: Vec<{payload}> = Vec::new(),\n\
                 );\n\n",
                ty = self.citizen_ty,
                id = self.citizen_snake,
            );
        } else {
            let _ = write!(out, "citizen_panel!({}, \"{}\");\n\n", self.citizen_ty, self.citizen_snake);
        }

        let _ = write!(
            out,
            "impl {ty} {{\n    pub fn show(&mut self, ui: &mut egui::Ui, {port}: &{iface}) {{\n",
            ty = self.citizen_ty,
            port = self.citizen_port,
            iface = self.iface_name,
        );
        let mut body = String::new();
        self.emit_widgets(&self.body, 2, &mut body)?;
        out.push_str(&body);
        out.push_str("    }\n}\n");
        Ok(out)
    }

    fn emit_widgets(&self, nodes: &[WidgetNode], depth: usize, out: &mut String) -> Result<(), Error> {
        let pad = "    ".repeat(depth);
        for node in nodes {
            match node {
                WidgetNode::Container { kind, children, span } => match kind.as_str() {
                    "row" => {
                        let _ = writeln!(out, "{pad}ui.horizontal(|ui| {{");
                        self.emit_widgets(children, depth + 1, out)?;
                        let _ = writeln!(out, "{pad}}});");
                    }
                    "column" => {
                        let _ = writeln!(out, "{pad}ui.vertical(|ui| {{");
                        self.emit_widgets(children, depth + 1, out)?;
                        let _ = writeln!(out, "{pad}}});");
                    }
                    "right" => {
                        // right_to_left renders in reverse insertion order, so
                        // the children are emitted reversed — they appear on
                        // screen left-to-right as declared in the source.
                        let _ = writeln!(
                            out,
                            "{pad}ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {{"
                        );
                        let reversed: Vec<WidgetNode> = children.iter().rev().cloned().collect();
                        self.emit_widgets(&reversed, depth + 1, out)?;
                        let _ = writeln!(out, "{pad}}});");
                    }
                    other => {
                        return Err(Error::Unsupported(format!("container `{other}` at {span:?}")));
                    }
                },
                WidgetNode::For { span, .. } => {
                    return Err(Error::Unsupported(format!("`for` loops (v0) at {span:?}")));
                }
                WidgetNode::Primitive(p) => self.emit_primitive(p, &pad, out)?,
            }
        }
        Ok(())
    }

    fn emit_primitive(
        &self,
        p: &mobius_lang::ast::Primitive,
        pad: &str,
        out: &mut String,
    ) -> Result<(), Error> {
        let label = p.label.as_deref().unwrap_or("");
        match (p.kind.as_str(), &p.binding) {
            ("separator", Binding::None) => {
                let _ = writeln!(out, "{pad}ui.separator();");
            }
            ("wordmark", Binding::None) => {
                let _ = writeln!(
                    out,
                    "{pad}ui.heading(egui::RichText::new(\"{label}\").strong());"
                );
            }
            ("label", Binding::Read { path, .. }) | ("clock", Binding::Read { path, .. }) => {
                let field = self.port_field(path)?;
                if p.kind == "clock" {
                    let _ = writeln!(
                        out,
                        "{pad}ui.label(egui::RichText::new({port}.{field}.get()).monospace().weak());",
                        port = self.citizen_port,
                    );
                } else {
                    let _ = writeln!(out, "{pad}ui.label({}.{field}.get());", self.citizen_port);
                }
            }
            ("button", Binding::Event(expr)) => {
                let push = self.render_event(expr)?;
                let _ = writeln!(out, "{pad}if ui.button(\"{label}\").clicked() {{");
                let _ = writeln!(out, "{pad}    {push}");
                let _ = writeln!(out, "{pad}}}");
            }
            ("button", Binding::TwoWay { path, .. }) => {
                // A button two-way bound to a bool cell: click sets it true;
                // whoever owns the consequence (a modal, the host) sets it
                // back to false.
                let field = self.port_field(path)?;
                let _ = writeln!(out, "{pad}if ui.button(\"{label}\").clicked() {{");
                let _ = writeln!(out, "{pad}    {}.{field}.set(true);", self.citizen_port);
                let _ = writeln!(out, "{pad}}}");
            }
            ("combo", Binding::TwoWay { path, .. }) => {
                let field = self.port_field(path)?;
                let enum_name = self
                    .state_fields
                    .iter()
                    .find(|f| f.name == field)
                    .and_then(|f| f.enum_name.clone())
                    .ok_or_else(|| {
                        Error::Unsupported(format!("combo on non-enum field `{field}`"))
                    })?;
                let port = &self.citizen_port;
                let selected = if label.is_empty() {
                    "picked.label().to_owned()".to_owned()
                } else {
                    format!("format!(\"{label} {{}}\", picked.label())")
                };
                let _ = writeln!(out, "{pad}let mut picked = {port}.{field}.get();");
                let _ = writeln!(
                    out,
                    "{pad}egui::ComboBox::from_id_salt(\"{salt}\")",
                    salt = format!("{}_{field}", self.citizen_snake),
                );
                let _ = writeln!(out, "{pad}    .selected_text({selected})");
                let _ = writeln!(out, "{pad}    .show_ui(ui, |ui| {{");
                let _ = writeln!(out, "{pad}        for variant in {enum_name}::ALL {{");
                let _ = writeln!(
                    out,
                    "{pad}            ui.selectable_value(&mut picked, variant, variant.label());"
                );
                let _ = writeln!(out, "{pad}        }}");
                let _ = writeln!(out, "{pad}    }});");
                let _ = writeln!(out, "{pad}if picked != {port}.{field}.get() {{");
                let _ = writeln!(out, "{pad}    {port}.{field}.set(picked);");
                let _ = writeln!(out, "{pad}}}");
            }
            (kind, _) => {
                return Err(Error::Unsupported(format!(
                    "primitive `{kind}` with this binding at {:?}",
                    p.span
                )));
            }
        }
        Ok(())
    }

    /// `c.field` → `field`, checking the port prefix.
    fn port_field(&self, path: &mobius_lang::ast::Path) -> Result<String, Error> {
        match path.segments.as_slice() {
            [port, field] if *port == self.citizen_port => Ok(field.clone()),
            _ => Err(Error::Unsupported(format!(
                "binding path `{}` (expected `{}.<field>`)",
                path.joined(),
                self.citizen_port
            ))),
        }
    }

    /// `-> c.actions(RibbonCmd::New)` → `self.outbox.push(RibbonCmd::New);`
    fn render_event(&self, expr: &Expr) -> Result<String, Error> {
        let Expr::Call { path, args, .. } = expr else {
            return Err(Error::Unsupported("event binding must be a call".into()));
        };
        let (signal_name, _) = self
            .signal
            .as_ref()
            .ok_or_else(|| Error::Unsupported("event fired but no signal field".into()))?;
        match path.segments.as_slice() {
            [port, field] if *port == self.citizen_port && field == signal_name => {}
            _ => {
                return Err(Error::Unsupported(format!(
                    "event target `{}` is not the signal field",
                    path.joined()
                )));
            }
        }
        let [arg] = args.as_slice() else {
            return Err(Error::Unsupported("event call takes exactly one argument".into()));
        };
        let Expr::Path(variant) = arg else {
            return Err(Error::Unsupported("event argument must be an enum variant".into()));
        };
        Ok(format!("self.outbox.push({});", variant.segments.join("::")))
    }

    // ── src/main.rs ─────────────────────────────────────────────────────

    fn emit_main(&self) -> String {
        let iface = &self.iface_name;
        let inst = &self.iface_instance;
        let citizen_ty = &self.citizen_ty;
        let snake = &self.citizen_snake;
        let mut out = format!(
            "//! {app} — generated by mobius_codegen.\n\
             //!\n\
             //! Wiring follows the egui_mobius book: Path A state lives in the\n\
             //! `state::{iface}` cells, the citizen's Path B events drain through\n\
             //! `actions::handle` once per frame, and the registry handles\n\
             //! lifecycle only.\n\n\
             mod citizens;\n\
             mod state;\n\n\
             use eframe::egui;\n\
             use egui_citizen::Registry;\n\n\
             struct App {{\n\
             \x20   registry: Registry,\n\
             \x20   {inst}: state::{iface},\n\
             \x20   {snake}: citizens::{citizen_ty},\n\
             }}\n\n\
             impl App {{\n\
             \x20   fn new() -> Self {{\n\
             \x20       let mut registry = Registry::new();\n\
             \x20       let {snake}_state = registry.add().with_name(\"{snake}\");\n\
             \x20       let _ = registry.drain_messages(); // discard setup messages\n\
             \x20       Self {{\n\
             \x20           registry,\n\
             \x20           {inst}: state::{iface}::new(),\n\
             \x20           {snake}: citizens::{citizen_ty}::new({snake}_state),\n\
             \x20       }}\n\
             \x20   }}\n\
             }}\n\n\
             impl eframe::App for App {{\n\
             \x20   fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {{\n\
             \x20       let ctx = ui.ctx().clone();\n",
            app = self.app_name,
        );

        if let Some(clock) = &self.clock_field {
            let _ = writeln!(out, "        self.{inst}.{clock}.set(wall_clock());");
            let _ = writeln!(
                out,
                "        ctx.request_repaint_after(std::time::Duration::from_millis(250));"
            );
        }
        if let Some(theme) = &self.theme_field {
            let _ = writeln!(out, "        ctx.set_visuals(self.{inst}.{theme}.get().visuals());");
        }

        let _ = write!(
            out,
            "\n        egui::Panel::top(\"{snake}\").show(ui, |ui| {{\n\
             \x20           self.{snake}.show(ui, &self.{inst});\n\
             \x20       }});\n\
             \x20       egui::CentralPanel::default().show(ui, |ui| {{\n\
             \x20           ui.weak(\"Generated by mobius_codegen — the bar above is the {citizen_ty} citizen.\");\n\
             \x20       }});\n\n",
        );

        if self.about.is_some() {
            let _ = writeln!(out, "        about_window(&ctx, &self.{inst});");
            out.push('\n');
        }

        if self.signal.is_some() {
            let _ = write!(
                out,
                "        // Path B: drain the citizen's outbox and route through actions.\n\
                 \x20       for cmd in std::mem::take(&mut self.{snake}.outbox) {{\n\
                 \x20           actions::handle(cmd, &self.{inst});\n\
                 \x20       }}\n",
            );
        }
        let _ = write!(
            out,
            "        // Registry lifecycle messages — drained after the panels show.\n\
             \x20       for _msg in self.registry.drain_messages() {{}}\n\
             \x20   }}\n\
             }}\n",
        );

        if let Some(about) = &self.about {
            let _ = write!(
                out,
                "\n/// The About window, from the `about(...)` layout directive.\n\
                 fn about_window(ctx: &egui::Context, {inst}: &state::{iface}) {{\n\
                 \x20   if !{inst}.{trigger}.get() {{\n\
                 \x20       return;\n\
                 \x20   }}\n\
                 \x20   let mut open = true;\n\
                 \x20   egui::Window::new(\"About {title}\")\n\
                 \x20       .collapsible(false)\n\
                 \x20       .resizable(true)\n\
                 \x20       .default_width(460.0)\n\
                 \x20       .pivot(egui::Align2::CENTER_CENTER)\n\
                 \x20       .default_pos(ctx.content_rect().center())\n\
                 \x20       .open(&mut open)\n\
                 \x20       .show(ctx, |ui| {{\n\
                 \x20           ui.heading(\"{title}\");\n\
                 \x20           ui.label(egui::RichText::new(\"{tagline}\").italics());\n\
                 \x20           ui.add_space(8.0);\n\
                 \x20           ui.separator();\n\
                 \x20           ui.weak(format!(\"version {{}}\", env!(\"CARGO_PKG_VERSION\")));\n\
                 \x20       }});\n\
                 \x20   if !open {{\n\
                 \x20       {inst}.{trigger}.set(false);\n\
                 \x20   }}\n\
                 }}\n",
                trigger = about.trigger_field,
                title = about.title,
                tagline = about.tagline,
            );
        }

        if let Some((_, payload)) = &self.signal {
            let _ = write!(
                out,
                "\nmod actions {{\n\
                 \x20   //! Path B routing stubs — replace each arm with real work:\n\
                 \x20   //! spawn it, send it over a channel, write results back into\n\
                 \x20   //! `{iface}` cells.\n\n\
                 \x20   use crate::state::{{{iface}, {payload}}};\n\n\
                 \x20   pub fn handle(cmd: {payload}, _{inst}: &{iface}) {{\n\
                 \x20       match cmd {{\n",
            );
            let variants = self
                .enums
                .iter()
                .find(|e| &e.name == payload)
                .map(|e| e.variants.clone())
                .unwrap_or_default();
            for v in &variants {
                let _ = writeln!(
                    out,
                    "            {payload}::{v} => eprintln!(\"[actions] {v}\"),"
                );
            }
            out.push_str("        }\n    }\n}\n");
        }

        if self.clock_field.is_some() {
            out.push_str(
                "\n/// Wall clock (UTC, std-only) for the ribbon's clock slot.\n\
                 fn wall_clock() -> String {\n\
                 \x20   let secs = std::time::SystemTime::now()\n\
                 \x20       .duration_since(std::time::UNIX_EPOCH)\n\
                 \x20       .unwrap_or_default()\n\
                 \x20       .as_secs();\n\
                 \x20   format!(\"{:02}:{:02}:{:02} UTC\", (secs / 3600) % 24, (secs / 60) % 60, secs % 60)\n\
                 }\n",
            );
        }

        let _ = write!(
            out,
            "\nfn main() -> Result<(), eframe::Error> {{\n\
             \x20   eframe::run_native(\n\
             \x20       \"{app}\",\n\
             \x20       eframe::NativeOptions {{\n\
             \x20           viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 420.0]),\n\
             \x20           ..Default::default()\n\
             \x20       }},\n\
             \x20       Box::new(|_cc| Ok(Box::new(App::new()))),\n\
             \x20   )\n\
             }}\n",
            app = self.app_name,
        );
        out
    }
}

// ── helpers ─────────────────────────────────────────────────────────────

fn scan_widgets(nodes: &[WidgetNode], f: &mut impl FnMut(&mobius_lang::ast::Primitive)) {
    for node in nodes {
        match node {
            WidgetNode::Container { children, .. } => scan_widgets(children, f),
            WidgetNode::For { body, .. } => scan_widgets(body, f),
            WidgetNode::Primitive(p) => f(p),
        }
    }
}

fn parse_about(args: &[DirectiveArg], iface_instance: &str) -> Result<AboutModel, Error> {
    let mut trigger = None;
    let mut title = None;
    let mut tagline = None;
    for arg in args {
        let DirectiveArg::Named(named) = arg else {
            return Err(Error::Unsupported("about() takes named arguments".into()));
        };
        match (named.name.as_str(), &named.value) {
            ("trigger", Expr::Path(p)) => match p.segments.as_slice() {
                [inst, field] if inst == iface_instance => trigger = Some(field.clone()),
                _ => {
                    return Err(Error::Unsupported(format!(
                        "about trigger `{}` (expected `{iface_instance}.<field>`)",
                        p.joined()
                    )));
                }
            },
            ("title", Expr::Str(s, _)) => title = Some(s.clone()),
            ("tagline", Expr::Str(s, _)) => tagline = Some(s.clone()),
            (other, _) => {
                return Err(Error::Unsupported(format!("about() argument `{other}`")));
            }
        }
    }
    Ok(AboutModel {
        trigger_field: trigger.ok_or_else(|| Error::Unsupported("about() needs trigger".into()))?,
        title: title.ok_or_else(|| Error::Unsupported("about() needs title".into()))?,
        tagline: tagline.unwrap_or_default(),
    })
}

fn rust_type(mobius_ty: &str) -> String {
    match mobius_ty {
        "string" => "String".into(),
        other => other.into(),
    }
}

fn render_default(field: &str, default: Option<&Expr>) -> Result<String, Error> {
    let Some(expr) = default else {
        return Err(Error::Unsupported(format!("field `{field}` needs a default (v0)")));
    };
    Ok(match expr {
        Expr::Bool(b, _) => b.to_string(),
        Expr::Int(i, _) => i.to_string(),
        Expr::Float(f, _) => format!("{f:?}"),
        Expr::Str(s, _) if s.is_empty() => "String::new()".into(),
        Expr::Str(s, _) => format!("{s:?}.to_string()"),
        Expr::Path(p) => p.segments.join("::"),
        other => {
            return Err(Error::Unsupported(format!(
                "default for `{field}`: {other:?}"
            )));
        }
    })
}

fn snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ribbon_fixture_generates() {
        let source = include_str!("../tests/fixtures/ribbon.mobius");
        let generated = generate(
            source,
            &Options {
                mobius_root: PathBuf::from("/tmp/mobius"),
            },
        )
        .expect("fixture generates");
        assert_eq!(generated.crate_name, "ribbon_demo");
        let ribbon = &generated.files["src/citizens/ribbon.rs"];
        // The right group is emitted reversed for right_to_left.
        let clock = ribbon.find("c.clock.get()").expect("clock emitted");
        let about = ribbon.find("ℹ About").expect("about emitted");
        assert!(clock < about, "clock must be added first (rightmost)");
        assert!(ribbon.contains("self.outbox.push(RibbonCmd::Build);"));
        let main = &generated.files["src/main.rs"];
        assert!(main.contains("actions::handle(cmd, &self.chrome);"));
        assert!(main.contains("about_window(&ctx, &self.chrome);"));
    }

    #[test]
    fn snake_cases() {
        assert_eq!(snake_case("RibbonDemo"), "ribbon_demo");
        assert_eq!(snake_case("Ribbon"), "ribbon");
    }
}
