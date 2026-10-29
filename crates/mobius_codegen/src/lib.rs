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
    /// Base directory for resolving asset paths in the source (e.g. the
    /// `about(image = "...")` banner) — usually the `.mobius` file's dir.
    pub asset_root: Option<PathBuf>,
    /// Override the generated crate name (default: snake_case of the app).
    pub crate_name: Option<String>,
}

/// Compiler directives from a `mobius.toml` next to the `.mobius` source —
/// the project-file half of the output-pathing story (the source stays
/// portable, the way RTL stays pure while `.qsf` / filelists carry paths).
/// All paths are resolved relative to the toml's directory. Precedence at
/// the CLI: explicit arguments > `mobius.toml` > defaults.
///
/// ```toml
/// [project]
/// out = "../../../../target/tmp/ribbon_demo"
/// # crate_name = "ribbon_demo"
/// # mobius_root = "../../../.."
/// # asset_root = "."
/// ```
#[derive(Debug, Default, PartialEq)]
pub struct ProjectConfig {
    pub out: Option<PathBuf>,
    pub crate_name: Option<String>,
    pub mobius_root: Option<PathBuf>,
    pub asset_root: Option<PathBuf>,
}

#[derive(serde::Deserialize)]
struct ProjectConfigFile {
    project: ProjectConfigTable,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfigTable {
    out: Option<String>,
    crate_name: Option<String>,
    mobius_root: Option<String>,
    asset_root: Option<String>,
}

/// Load the `mobius.toml` sitting next to `source`, if any. Relative paths
/// in the file resolve against its own directory.
pub fn load_project_config(source: &Path) -> Result<Option<ProjectConfig>, Error> {
    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    let path = dir.join("mobius.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    let file: ProjectConfigFile = toml::from_str(&text)
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    // Absolutize: these paths end up inside the generated Cargo.toml,
    // where a path relative to *this* file would silently resolve against
    // the output directory instead.
    let resolve = |p: Option<String>| {
        p.map(|p| {
            let joined = dir.join(p);
            std::path::absolute(&joined).unwrap_or(joined)
        })
    };
    Ok(Some(ProjectConfig {
        out: resolve(file.project.out),
        crate_name: file.project.crate_name,
        mobius_root: resolve(file.project.mobius_root),
        asset_root: resolve(file.project.asset_root),
    }))
}

/// A generated project: relative path → file contents, plus binary assets
/// copied from the source's asset references.
pub struct Generated {
    pub crate_name: String,
    pub files: BTreeMap<String, String>,
    pub assets: BTreeMap<String, Vec<u8>>,
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
        for (rel, bytes) in &self.assets {
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, bytes)?;
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
    /// A referenced asset could not be read.
    Asset(String),
    /// A malformed `mobius.toml`.
    Config(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {e:?}"),
            Error::Unsupported(what) => write!(f, "unsupported: {what}"),
            Error::Asset(what) => write!(f, "asset: {what}"),
            Error::Config(what) => write!(f, "mobius.toml: {what}"),
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
    let mut model = Model::build(app)?;
    if let Some(name) = &opts.crate_name {
        model.crate_name = name.clone();
    }

    let mut assets = BTreeMap::new();
    if let Some(image) = model.about.as_ref().and_then(|a| a.image.clone()) {
        let root = opts.asset_root.as_ref().ok_or_else(|| {
            Error::Asset(format!("about image `{image}` given but no asset_root to resolve it"))
        })?;
        let path = root.join(&image);
        let bytes = std::fs::read(&path)
            .map_err(|e| Error::Asset(format!("cannot read {}: {e}", path.display())))?;
        assets.insert("assets/about.png".to_owned(), bytes);
    }

    let mut files = BTreeMap::new();
    files.insert("Cargo.toml".into(), model.emit_manifest(opts));
    // Generated projects are committable — keep their build junk out.
    files.insert(".gitignore".into(), "/target\n/Cargo.lock\n".into());
    files.insert("src/state.rs".into(), model.emit_state());
    if let Some(theme_rs) = model.emit_theme() {
        files.insert("src/theme.rs".into(), theme_rs);
    }
    files.insert("src/citizens/mod.rs".into(), model.emit_citizens_mod());
    for decl in &model.decls {
        files.insert(
            format!("src/citizens/{}.rs", decl.snake),
            model.emit_citizen(decl)?,
        );
    }
    files.insert("src/main.rs".into(), model.emit_main(opts));

    Ok(Generated {
        crate_name: model.crate_name.clone(),
        files,
        assets,
    })
}

// ── The resolved model ──────────────────────────────────────────────────

struct EnumModel {
    name: String,
    variants: Vec<String>,
    /// Bound by a `combo` — gets `ALL` and `label()`.
    combo: bool,
}

struct FieldModel {
    name: String,
    rust_ty: String,
    default: String,
    enum_name: Option<String>,
}

struct IfaceModel {
    name: String,
    fields: Vec<FieldModel>,
    /// `(field_name, payload_enum)` of the single signal field, if any.
    signal: Option<(String, String)>,
}

struct PortModel {
    name: String,
    iface: String,
}

struct CitizenDeclModel {
    ty: String,
    snake: String,
    ports: Vec<PortModel>,
    body: Vec<WidgetNode>,
}

/// One `dock(instance, region = ..., split = ...)` placement.
struct DockedModel {
    inst: String,
    /// left / right / bottom / center.
    region: String,
    /// Sub-placement inside the region's node: `split = right` puts this
    /// group in a right-half split of the region instead of its tab bar.
    split: Option<String>,
}

struct CitizenInstModel {
    name: String,
    /// Declaration type name.
    decl: String,
    /// `(port_name, iface_instance)` in the declaration's port order.
    bindings: Vec<(String, String)>,
}

struct Model {
    app_name: String,
    crate_name: String,
    enums: Vec<EnumModel>,
    ifaces: Vec<IfaceModel>,
    /// `(instance_name, iface_type)` in wiring order.
    iface_insts: Vec<(String, String)>,
    decls: Vec<CitizenDeclModel>,
    insts: Vec<CitizenInstModel>,
    /// Citizen instances in the top bar, in order.
    top: Vec<String>,
    /// `(citizen_instance, region)` — region ∈ left/right/bottom/center.
    docked: Vec<DockedModel>,
    about: Option<AboutModel>,
    /// `(iface_instance, field)` written with the wall clock per frame.
    clock_writes: Vec<(String, String)>,
    /// `(iface_instance, field, enum)` of the combo-bound `theme` field.
    theme: Option<(String, String, String)>,
}

struct AboutModel {
    /// `(iface_instance, field)` of the bool cell that opens the window.
    trigger: (String, String),
    title: String,
    tagline: String,
    /// Source-relative path of the banner image, copied into the generated
    /// project as `assets/about.png`.
    image: Option<String>,
    /// Show the ShutterFrame-style "Built with" dependency table.
    deps: bool,
    /// User-resizable window (width fills, vertical shrink scrolls).
    resizable: bool,
}

impl Model {
    fn build(app: &App) -> Result<Model, Error> {
        let mut enums: Vec<EnumModel> = Vec::new();
        let mut iface_decls: Vec<&mobius_lang::ast::Interface> = Vec::new();
        let mut citizens: Vec<&CitizenDecl> = Vec::new();
        let mut wiring: Vec<&Stmt> = Vec::new();
        let mut layout: Vec<&Stmt> = Vec::new();

        for item in &app.items {
            match item {
                AppItem::Enum(e) => enums.push(EnumModel {
                    name: e.name.clone(),
                    variants: e.variants.clone(),
                    combo: false,
                }),
                AppItem::Interface(i) => iface_decls.push(i),
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

        // Interface declarations → Path A cell structs + Path B signals.
        let mut ifaces = Vec::new();
        for iface in &iface_decls {
            let mut fields = Vec::new();
            let mut signal = None;
            for field in &iface.fields {
                match &field.kind {
                    FieldKind::State { ty, default } => {
                        let ty_name = ty.path.joined();
                        let enum_name =
                            enums.iter().find(|e| e.name == ty_name).map(|e| e.name.clone());
                        fields.push(FieldModel {
                            name: field.name.clone(),
                            rust_ty: rust_type(&ty_name),
                            default: render_default(&field.name, &ty_name, default.as_ref())?,
                            enum_name,
                        });
                    }
                    FieldKind::Signal { ty } => {
                        if signal
                            .replace((field.name.clone(), ty.path.joined()))
                            .is_some()
                        {
                            return Err(Error::Unsupported(format!(
                                "multiple signal fields in `{}` (v0)",
                                iface.name
                            )));
                        }
                    }
                    FieldKind::Array { .. } => {
                        return Err(Error::Unsupported(format!(
                            "array field `{}` (v0)",
                            field.name
                        )));
                    }
                }
            }
            ifaces.push(IfaceModel {
                name: iface.name.clone(),
                fields,
                signal,
            });
        }

        // Citizen declarations: ports name their interface type. A citizen
        // with no ports is legal — fully self-contained prefabs (the
        // terminal) own all their state locally.
        let mut decls = Vec::new();
        for citizen in &citizens {
            let mut ports = Vec::new();
            for port in &citizen.ports {
                // `Chrome.ribbon` — the modport view is not checked here
                // (that's lower()'s job); codegen needs the interface type.
                let iface = port.ty.path.segments[0].clone();
                if !ifaces.iter().any(|i| i.name == iface) {
                    return Err(Error::Unsupported(format!(
                        "port `{}` of `{}` names unknown interface `{iface}`",
                        port.name, citizen.name
                    )));
                }
                ports.push(PortModel {
                    name: port.name.clone(),
                    iface,
                });
            }
            decls.push(CitizenDeclModel {
                ty: citizen.name.clone(),
                snake: snake_case(&citizen.name),
                ports,
                body: citizen.body.clone(),
            });
        }

        // Wiring: interface instances, then citizen instances with their
        // port bindings (`Ribbon(c = chrome.ribbon)` → c ↔ chrome).
        let mut iface_insts: Vec<(String, String)> = Vec::new();
        let mut insts: Vec<CitizenInstModel> = Vec::new();
        for stmt in &wiring {
            let Stmt::Instance { name, ty, args, .. } = stmt else {
                continue;
            };
            let ty = ty.joined();
            if ifaces.iter().any(|i| i.name == ty) {
                iface_insts.push((name.clone(), ty));
            } else if let Some(decl) = decls.iter().find(|d| d.ty == ty) {
                let mut bindings = Vec::new();
                for port in &decl.ports {
                    let arg = args
                        .iter()
                        .find(|a| a.name == port.name)
                        .ok_or_else(|| {
                            Error::Unsupported(format!(
                                "instance `{name}` does not bind port `{}`",
                                port.name
                            ))
                        })?;
                    let Expr::Path(p) = &arg.value else {
                        return Err(Error::Unsupported(format!(
                            "binding for `{name}.{}` must be `instance.modport`",
                            port.name
                        )));
                    };
                    let iface_inst = p.segments[0].clone();
                    match iface_insts.iter().find(|(n, _)| n == &iface_inst) {
                        Some((_, iface_ty)) if iface_ty == &port.iface => {}
                        Some((_, iface_ty)) => {
                            return Err(Error::Unsupported(format!(
                                "`{name}.{}` wants `{}` but `{iface_inst}` is `{iface_ty}`",
                                port.name, port.iface
                            )));
                        }
                        None => {
                            return Err(Error::Unsupported(format!(
                                "binding for `{name}.{}` names unknown instance `{iface_inst}`",
                                port.name
                            )));
                        }
                    }
                    bindings.push((port.name.clone(), iface_inst));
                }
                insts.push(CitizenInstModel {
                    name: name.clone(),
                    decl: decl.ty.clone(),
                    bindings,
                });
            } else {
                return Err(Error::Unsupported(format!(
                    "instance `{name}` of unknown type `{ty}`"
                )));
            }
        }

        // Layout directives.
        let mut about = None;
        let mut top = Vec::new();
        let mut docked = Vec::new();
        for stmt in &layout {
            if let Stmt::Directive { name, args, .. } = stmt {
                match name.as_str() {
                    "top" => {
                        for arg in args {
                            if let DirectiveArg::Positional(Expr::Path(p)) = arg {
                                top.push(p.joined());
                            }
                        }
                    }
                    "dock" => docked.push(parse_dock(args)?),
                    "about" => about = Some(parse_about(args, &iface_insts)?),
                    other => {
                        return Err(Error::Unsupported(format!("@layout directive `{other}`")));
                    }
                }
            }
        }
        for placed in top.iter().chain(docked.iter().map(|d| &d.inst)) {
            if !insts.iter().any(|i| &i.name == placed) {
                return Err(Error::Unsupported(format!(
                    "@layout places unknown citizen instance `{placed}`"
                )));
            }
        }
        if !docked.is_empty() && !docked.iter().any(|d| d.region == "center") {
            return Err(Error::Unsupported(
                "dock layout needs at least one `region = center`".into(),
            ));
        }
        for d in &docked {
            if d.split.is_some() && d.region == "center" {
                return Err(Error::Unsupported(
                    "`split = ...` applies to left/right/bottom regions, not center".into(),
                ));
            }
        }
        // The perspectives menu manipulates the dock, so it must live in a
        // top() citizen (the TabViewer cannot lend the dock state it is
        // rendering inside) — and only makes sense when something docks.
        for inst in &insts {
            let decl = decls.iter().find(|d| d.ty == inst.decl).unwrap();
            if decl_has_perspectives(decl) {
                if docked.iter().any(|d| d.inst == inst.name) {
                    return Err(Error::Unsupported(format!(
                        "`perspectives` in docked instance `{}` — it belongs in a top() citizen",
                        inst.name
                    )));
                }
                if docked.is_empty() {
                    return Err(Error::Unsupported(
                        "`perspectives` without any dock() layout".into(),
                    ));
                }
            }
        }

        // Scan widget trees for host obligations (clock writers, the theme
        // enum) and enum needs (combo). Per-instance: a clock in a
        // declaration obliges every instance's bound interface.
        let mut clock_writes: Vec<(String, String)> = Vec::new();
        let mut theme: Option<(String, String, String)> = None;
        for decl in &decls {
            let mut decl_clock_fields: Vec<(String, String)> = Vec::new(); // (port, field)
            let mut decl_theme: Option<(String, String, String)> = None; // (port, field, enum)
            scan_widgets(&decl.body, &mut |p: &mobius_lang::ast::Primitive| {
                match (p.kind.as_str(), &p.binding) {
                    ("clock", Binding::Read { path, .. }) => {
                        if let [port, field] = path.segments.as_slice() {
                            decl_clock_fields.push((port.clone(), field.clone()));
                        }
                    }
                    ("combo", Binding::TwoWay { path, .. }) => {
                        if let [port, field] = path.segments.as_slice() {
                            let iface = decl.ports.iter().find(|pm| &pm.name == port);
                            let enum_name = iface.and_then(|pm| {
                                ifaces
                                    .iter()
                                    .find(|i| i.name == pm.iface)
                                    .and_then(|i| i.fields.iter().find(|f| &f.name == field))
                                    .and_then(|f| f.enum_name.clone())
                            });
                            if let Some(enum_name) = enum_name {
                                if let Some(e) = enums.iter_mut().find(|e| e.name == enum_name) {
                                    e.combo = true;
                                }
                                if field == "theme" {
                                    decl_theme =
                                        Some((port.clone(), field.clone(), enum_name));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });
            for inst in insts.iter().filter(|i| i.decl == decl.ty) {
                for (port, field) in &decl_clock_fields {
                    if let Some((_, iface_inst)) =
                        inst.bindings.iter().find(|(p, _)| p == port)
                    {
                        clock_writes.push((iface_inst.clone(), field.clone()));
                    }
                }
                if let (Some((port, field, enum_name)), None) = (&decl_theme, &theme) {
                    if let Some((_, iface_inst)) =
                        inst.bindings.iter().find(|(p, _)| p == port)
                    {
                        theme = Some((iface_inst.clone(), field.clone(), enum_name.clone()));
                    }
                }
            }
        }
        clock_writes.dedup();

        Ok(Model {
            app_name: app.name.clone(),
            crate_name: snake_case(&app.name),
            enums,
            ifaces,
            iface_insts,
            decls,
            insts,
            top,
            docked,
            about,
            clock_writes,
            theme,
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
# Glow renderer, not the default wgpu — resizing under wgpu showed frozen
# frames and black rectangles; CopperTrace ships the same choice.
eframe = {{ version = "{egui}", default-features = false, features = [
    "default_fonts",
    "glow",
    "wayland",
    "x11",
] }}
egui = "{egui}"{image_deps}{dock_dep}{lens_dep}
egui_citizen = {{ path = "{citizen}" }}
egui_mobius_reactive = {{ path = "{reactive}" }}

# egui repaints continuously while the window resizes — unoptimized debug
# builds freeze. Light opt for the app, full for dependencies.
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 2

# Standalone — never a member of an enclosing workspace.
[workspace]
"#,
            name = self.crate_name,
            egui = EGUI_VERSION,
            image_deps = if self.about.as_ref().is_some_and(|a| a.image.is_some()) {
                "\negui_extras = { version = \"0.36\", features = [\"image\"] }\n\
                 image = { version = \"0.25\", default-features = false, features = [\"png\"] }"
            } else {
                ""
            },
            dock_dep = if self.docked.is_empty() {
                ""
            } else if self.has_perspectives_menu() {
                // Perspectives persist DockState as JSON (ForgePM's shape).
                "\negui_dock = { version = \"0.21\", features = [\"serde\"] }\n\
                 serde = { version = \"1\", features = [\"derive\"] }\n\
                 serde_json = \"1\"\n\
                 dirs = \"5\""
            } else {
                "\negui_dock = \"0.21\""
            },
            lens_dep = if self.logger_iface().is_some() {
                format!(
                    "\negui_lens = {{ path = \"{}\" }}",
                    opts.mobius_root.join("crates/egui_lens").display()
                )
            } else {
                String::new()
            },
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
             //! Each interface is a Path A boundary: one `Dynamic<T>` cell per\n\
             //! state field. Citizens hold `&Iface` and read / write the cells;\n\
             //! nothing here is passed `&mut`.\n\n\
             use egui_mobius_reactive::Dynamic;\n",
            app = self.app_name,
        );
        if self.logger_iface().is_some() {
            out.push_str("use egui_lens::{LogColors, ReactiveEventLoggerState};\n");
        }

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

            if e.combo {
                let all = e
                    .variants
                    .iter()
                    .map(|v| format!("{}::{v}", e.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = write!(
                    out,
                    "\nimpl {name} {{\n\
                     \x20   pub const ALL: [{name}; {n}] = [{all}];\n\n\
                     \x20   pub fn label(self) -> &'static str {{\n        match self {{\n",
                    name = e.name,
                    n = e.variants.len(),
                );
                for v in &e.variants {
                    let _ = writeln!(out, "            {}::{v} => \"{v}\",", e.name);
                }
                out.push_str("        }\n    }\n}\n");
            }
        }

        for iface in &self.ifaces {
            let _ = write!(
                out,
                "\n/// Path A cells of the `{name}` interface.\n\
                 #[derive(Clone)]\n\
                 pub struct {name} {{\n",
                name = iface.name,
            );
            for f in &iface.fields {
                let _ = writeln!(out, "    pub {}: Dynamic<{}>,", f.name, f.rust_ty);
            }
            let _ = write!(
                out,
                "}}\n\nimpl {name} {{\n    pub fn new() -> Self {{\n        Self {{\n",
                name = iface.name,
            );
            for f in &iface.fields {
                let _ = writeln!(out, "            {}: Dynamic::new({}),", f.name, f.default);
            }
            out.push_str("        }\n    }\n}\n");
        }
        out
    }

    // ── src/theme.rs ────────────────────────────────────────────────────

    /// The theme enum's palette + chrome colours, in the house style:
    /// `apply()` sets the full visuals, `brand()` colours the wordmark,
    /// `accent()` colours ribbon chrome (clock, about). Palettes are
    /// ported from CopperTrace's `theme.rs`; variants are matched to a
    /// palette by name (`Light`/`Tokyo`/`Copper`), unknown dark variants
    /// cycle through the dark palettes.
    fn emit_theme(&self) -> Option<String> {
        let enum_name = self.theme_enum()?;
        let e = self.enums.iter().find(|e| e.name == enum_name)?;

        let mut dark_index = 0usize;
        let palettes: Vec<&'static str> = e
            .variants
            .iter()
            .map(|v| {
                let p = palette_for(v, dark_index);
                if p != "light" && !v.contains("Tokyo") && !v.contains("Copper") {
                    dark_index += 1;
                }
                p
            })
            .collect();

        let mut out = format!(
            "//! Themes for {app} — generated by mobius_codegen. Palettes ported\n\
             //! from CopperTrace's `theme.rs` (Discord-dark / Tokyo Night Storm /\n\
             //! Forge-PM grey). Replace freely — this file is scaffolding.\n\n\
             use egui::Color32;\n\nuse crate::state::{enum_name};\n\n\
             impl {enum_name} {{\n\
             \x20   /// Apply this theme to the egui context.\n\
             \x20   pub fn apply(self, ctx: &egui::Context) {{\n\
             \x20       match self {{\n",
            app = self.app_name,
        );
        for (v, p) in e.variants.iter().zip(&palettes) {
            let _ = writeln!(out, "            {enum_name}::{v} => apply_{p}(ctx),");
        }
        out.push_str(
            "        }\n    }\n\n\
             \x20   /// Brand wordmark colours, chosen per theme.\n\
             \x20   pub fn brand(self) -> (Color32, Color32) {\n        match self {\n",
        );
        for (v, p) in e.variants.iter().zip(&palettes) {
            let brand = match *p {
                "tokyo" => "(Color32::from_rgb(0xff, 0x9e, 0x64), Color32::from_rgb(0x7a, 0xa2, 0xf7))",
                "light" => "(Color32::from_rgb(214, 127, 60), Color32::from_rgb(0xe6, 0xe6, 0xe6))",
                _ => "(Color32::from_rgb(214, 127, 60), Color32::from_rgb(225, 228, 230))",
            };
            let _ = writeln!(out, "            {enum_name}::{v} => {brand},");
        }
        out.push_str(
            "        }\n    }\n\n\
             \x20   /// Accent colour for ribbon chrome (clock, version / about tags).\n\
             \x20   pub fn accent(self) -> Color32 {\n        match self {\n",
        );
        for (v, p) in e.variants.iter().zip(&palettes) {
            let accent = match *p {
                "tokyo" => "Color32::from_rgb(0x7d, 0xcf, 0xff)",
                "light" => "Color32::from_rgb(0x8a, 0xb4, 0xf8)",
                _ => "Color32::from_rgb(0x58, 0x65, 0xf2)",
            };
            let _ = writeln!(out, "            {enum_name}::{v} => {accent},");
        }
        out.push_str("        }\n    }\n}\n");

        for p in ["discord", "tokyo", "light"] {
            if palettes.contains(&p) {
                out.push('\n');
                out.push_str(match p {
                    "discord" => PALETTE_DISCORD,
                    "tokyo" => PALETTE_TOKYO,
                    _ => PALETTE_LIGHT,
                });
            }
        }
        Some(out)
    }

    /// The enum behind the combo-bound `theme` field, if there is one.
    fn theme_enum(&self) -> Option<String> {
        self.theme.as_ref().map(|(_, _, e)| e.clone())
    }

    // ── src/citizens/ ───────────────────────────────────────────────────

    fn emit_citizens_mod(&self) -> String {
        let mut out = "//! Citizen modules — generated by mobius_codegen.\n\n".to_owned();
        for decl in &self.decls {
            let _ = writeln!(out, "pub mod {};", decl.snake);
        }
        out.push('\n');
        for decl in &self.decls {
            let _ = writeln!(out, "pub use {}::{};", decl.snake, decl.ty);
        }
        out
    }

    /// The Path B signal reachable from a declaration's ports, if any:
    /// `(port, signal_field, payload_enum)`.
    fn decl_signal(&self, decl: &CitizenDeclModel) -> Option<(String, String, String)> {
        decl.ports.iter().find_map(|port| {
            self.ifaces
                .iter()
                .find(|i| i.name == port.iface)
                .and_then(|i| i.signal.clone())
                .map(|(field, payload)| (port.name.clone(), field, payload))
        })
    }

    /// The egui_lens logger interface, if declared:
    /// `(iface, state_field, colors_field)`.
    fn logger_iface(&self) -> Option<(&IfaceModel, String, String)> {
        self.ifaces.iter().find_map(|i| {
            let state = i.fields.iter().find(|f| f.rust_ty == "ReactiveEventLoggerState")?;
            let colors = i.fields.iter().find(|f| f.rust_ty == "LogColors")?;
            Some((i, state.name.clone(), colors.name.clone()))
        })
    }

    /// The first wired instance of the logger interface.
    fn logger_inst(&self) -> Option<String> {
        let (iface, _, _) = self.logger_iface()?;
        self.iface_insts
            .iter()
            .find(|(_, ty)| ty == &iface.name)
            .map(|(n, _)| n.clone())
    }

    /// A top-bar citizen carries the `perspectives` menu (and a dock
    /// exists for it to manage).
    fn has_perspectives_menu(&self) -> bool {
        !self.docked.is_empty()
            && self.top.iter().any(|t| {
                self.insts
                    .iter()
                    .find(|i| &i.name == t)
                    .and_then(|i| self.decls.iter().find(|d| d.ty == i.decl))
                    .is_some_and(decl_has_perspectives)
            })
    }

    /// The field a binding path names: `(port, field_model)`.
    fn port_field(
        &self,
        decl: &CitizenDeclModel,
        path: &mobius_lang::ast::Path,
    ) -> Result<(String, FieldModel), Error> {
        if let [port, field] = path.segments.as_slice() {
            if let Some(pm) = decl.ports.iter().find(|pm| &pm.name == port) {
                if let Some(f) = self
                    .ifaces
                    .iter()
                    .find(|i| i.name == pm.iface)
                    .and_then(|i| i.fields.iter().find(|f| &f.name == field))
                {
                    return Ok((
                        port.clone(),
                        FieldModel {
                            name: f.name.clone(),
                            rust_ty: f.rust_ty.clone(),
                            default: f.default.clone(),
                            enum_name: f.enum_name.clone(),
                        },
                    ));
                }
            }
        }
        Err(Error::Unsupported(format!(
            "binding path `{}` in `{}` (expected `<port>.<field>`)",
            path.joined(),
            decl.ty
        )))
    }

    fn emit_citizen(&self, decl: &CitizenDeclModel) -> Result<String, Error> {
        let mut uses: Vec<String> = decl.ports.iter().map(|p| p.iface.clone()).collect();
        if let Some((_, _, payload)) = self.decl_signal(decl) {
            uses.push(payload);
        }
        for e in &self.enums {
            if e.combo
                && decl.ports.iter().any(|p| {
                    self.ifaces
                        .iter()
                        .find(|i| i.name == p.iface)
                        .is_some_and(|i| {
                            i.fields.iter().any(|f| f.enum_name.as_deref() == Some(&e.name))
                        })
                })
            {
                uses.push(e.name.clone());
            }
        }
        uses.sort();
        uses.dedup();

        let state_use = if uses.is_empty() {
            String::new()
        } else {
            format!("\nuse crate::state::{{{}}};\n", uses.join(", "))
        };
        let mut out = format!(
            "//! The {ty} citizen — generated by mobius_codegen.\n\n\
             use egui_citizen::citizen_panel;\n{state_use}\n",
            ty = decl.ty,
        );

        let has_terminal = decl_has_terminal(decl);
        let mut extra_fields = String::new();
        if let Some((_, signal_name, payload)) = self.decl_signal(decl) {
            let _ = write!(
                extra_fields,
                "    // Path B outbox for the `{signal_name}` signal — drained by the\n\
                 \x20   // app once per frame.\n\
                 \x20   outbox: Vec<{payload}> = Vec::new(),\n",
            );
        }
        if has_terminal {
            extra_fields.push_str(
                "    // Terminal prefab locals — scrollback and the input buffer\n\
                 \x20   // (citizen-local state, CopperTrace's terminal shape).\n\
                 \x20   output: Vec<String> = Vec::new(),\n\
                 \x20   cmd_buf: String = String::new(),\n",
            );
        }
        if extra_fields.is_empty() {
            let _ = write!(out, "citizen_panel!({}, \"{}\");\n\n", decl.ty, decl.snake);
        } else {
            let _ = write!(
                out,
                "citizen_panel!({ty}, \"{id}\",\n{extra_fields});\n\n",
                ty = decl.ty,
                id = decl.snake,
            );
        }

        let mut params: String = decl
            .ports
            .iter()
            .map(|p| format!(", {}: &{}", p.name, p.iface))
            .collect();
        if decl_has_perspectives(decl) {
            params.push_str(", workspace: &mut crate::Workspace");
        }
        let _ = write!(
            out,
            "impl {ty} {{\n    pub fn show(&mut self, ui: &mut egui::Ui{params}) {{\n",
            ty = decl.ty,
        );
        let _ = writeln!(out, "        ui.add_space(3.0);");
        if let Some((port, field)) = self.chrome_theme_binding(decl) {
            let _ = writeln!(out, "        let theme = {port}.{field}.get();");
        }
        let mut body = String::new();
        self.emit_widgets(decl, &decl.body, 2, &mut body)?;
        out.push_str(&body);
        out.push_str("    }\n}\n");
        if has_terminal {
            out.push_str(
                &TERMINAL_IMPL
                    .replace("__TY__", &decl.ty)
                    .replace("__SNAKE__", &decl.snake)
                    .replace("__APP__", &self.app_name),
            );
        }
        Ok(out)
    }

    /// If this declaration's chrome (wordmark, clock, about-trigger
    /// button) wants theme colours, the `(port, field)` to read the theme
    /// from — a port whose interface carries the combo-bound theme field.
    fn chrome_theme_binding(&self, decl: &CitizenDeclModel) -> Option<(String, String)> {
        let (_, theme_field, _) = self.theme.as_ref()?;
        let trigger = self.about.as_ref().map(|a| a.trigger.1.as_str());
        let mut needs = false;
        scan_widgets(&decl.body, &mut |p| match (p.kind.as_str(), &p.binding) {
            ("wordmark", _) | ("clock", _) => needs = true,
            ("button", Binding::TwoWay { path, .. }) => {
                if path.segments.last().map(String::as_str) == trigger {
                    needs = true;
                }
            }
            _ => {}
        });
        if !needs {
            return None;
        }
        decl.ports
            .iter()
            .find(|pm| {
                self.ifaces
                    .iter()
                    .find(|i| i.name == pm.iface)
                    .is_some_and(|i| i.fields.iter().any(|f| &f.name == theme_field))
            })
            .map(|pm| (pm.name.clone(), theme_field.clone()))
    }

    fn emit_widgets(
        &self,
        decl: &CitizenDeclModel,
        nodes: &[WidgetNode],
        depth: usize,
        out: &mut String,
    ) -> Result<(), Error> {
        let pad = "    ".repeat(depth);
        for node in nodes {
            match node {
                WidgetNode::Container { kind, children, span } => match kind.as_str() {
                    "row" => {
                        let _ = writeln!(out, "{pad}ui.horizontal(|ui| {{");
                        self.emit_widgets(decl, children, depth + 1, out)?;
                        let _ = writeln!(out, "{pad}}});");
                    }
                    "column" => {
                        let _ = writeln!(out, "{pad}ui.vertical(|ui| {{");
                        self.emit_widgets(decl, children, depth + 1, out)?;
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
                        self.emit_widgets(decl, &reversed, depth + 1, out)?;
                        let _ = writeln!(out, "{pad}}});");
                    }
                    other => {
                        return Err(Error::Unsupported(format!("container `{other}` at {span:?}")));
                    }
                },
                WidgetNode::For { span, .. } => {
                    return Err(Error::Unsupported(format!("`for` loops (v0) at {span:?}")));
                }
                WidgetNode::Primitive(p) => self.emit_primitive(decl, p, &pad, out)?,
            }
        }
        Ok(())
    }

    fn emit_primitive(
        &self,
        decl: &CitizenDeclModel,
        p: &mobius_lang::ast::Primitive,
        pad: &str,
        out: &mut String,
    ) -> Result<(), Error> {
        let label = p.label.as_deref().unwrap_or("");
        let themed = self.chrome_theme_binding(decl).is_some();
        match (p.kind.as_str(), &p.binding) {
            ("separator", Binding::None) => {
                let _ = writeln!(out, "{pad}ui.separator();");
            }
            ("wordmark", Binding::None) => {
                if themed {
                    // Two-tone wordmark in the theme's brand colours, split
                    // at the camel hump (CopperTrace house style).
                    let (head, tail) = split_wordmark(label);
                    let _ = writeln!(out, "{pad}ui.scope(|ui| {{");
                    let _ = writeln!(out, "{pad}    ui.spacing_mut().item_spacing.x = 0.0;");
                    let brands = if tail.is_empty() { "(brand_a, _)" } else { "(brand_a, brand_b)" };
                    let _ = writeln!(out, "{pad}    let {brands} = theme.brand();");
                    let _ = writeln!(
                        out,
                        "{pad}    ui.heading(egui::RichText::new(\"{head}\").size(20.0).strong().color(brand_a));"
                    );
                    if !tail.is_empty() {
                        let _ = writeln!(
                            out,
                            "{pad}    ui.heading(egui::RichText::new(\"{tail}\").size(20.0).strong().color(brand_b));"
                        );
                    }
                    let _ = writeln!(out, "{pad}}});");
                } else {
                    let _ = writeln!(
                        out,
                        "{pad}ui.heading(egui::RichText::new(\"{label}\").strong());"
                    );
                }
            }
            ("label", Binding::Read { path, .. }) | ("clock", Binding::Read { path, .. }) => {
                let (port, field) = self.port_field(decl, path)?;
                if p.kind == "clock" {
                    let style = if themed {
                        ".monospace().color(theme.accent())"
                    } else {
                        ".monospace().weak()"
                    };
                    let _ = writeln!(
                        out,
                        "{pad}ui.label(egui::RichText::new({port}.{name}.get()){style});",
                        name = field.name,
                    );
                } else {
                    // Typed display: strings label directly, floats get a
                    // stable precision, everything else goes through Display.
                    let line = match field.rust_ty.as_str() {
                        "String" => format!("ui.label({port}.{}.get());", field.name),
                        "f32" | "f64" => format!(
                            "ui.label(format!(\"{{:.2}}\", {port}.{}.get()));",
                            field.name
                        ),
                        _ => format!(
                            "ui.label(format!(\"{{}}\", {port}.{}.get()));",
                            field.name
                        ),
                    };
                    let _ = writeln!(out, "{pad}{line}");
                }
            }
            ("button", Binding::Event(expr)) => {
                let push = self.render_event(decl, expr)?;
                let _ = writeln!(out, "{pad}if ui.button(\"{label}\").clicked() {{");
                let _ = writeln!(out, "{pad}    {push}");
                let _ = writeln!(out, "{pad}}}");
            }
            ("button", Binding::TwoWay { path, .. }) => {
                // A button two-way bound to a bool cell: click sets it true;
                // whoever owns the consequence (a modal, the host) sets it
                // back to false.
                let (port, field) = self.port_field(decl, path)?;
                let is_about = themed
                    && self.about.as_ref().map(|a| a.trigger.1.as_str())
                        == Some(field.name.as_str());
                let button = if is_about {
                    format!("ui.button(egui::RichText::new(\"{label}\").color(theme.accent()))")
                } else {
                    format!("ui.button(\"{label}\")")
                };
                let _ = writeln!(out, "{pad}if {button}.clicked() {{");
                let _ = writeln!(out, "{pad}    {port}.{}.set(true);", field.name);
                let _ = writeln!(out, "{pad}}}");
            }
            ("slider", Binding::TwoWay { path, .. }) => {
                let (port, field) = self.port_field(decl, path)?;
                let (lo, hi) = p.range.as_ref().ok_or_else(|| {
                    Error::Unsupported(format!("slider `{}` needs a `lo..hi` range", field.name))
                })?;
                let _ = writeln!(out, "{pad}let mut value = {port}.{}.get();", field.name);
                let _ = writeln!(
                    out,
                    "{pad}if ui.add(egui::Slider::new(&mut value, {lo}..={hi}).text(\"{label}\")).changed() {{",
                    lo = render_number(lo),
                    hi = render_number(hi),
                );
                let _ = writeln!(out, "{pad}    {port}.{}.set(value);", field.name);
                let _ = writeln!(out, "{pad}}}");
            }
            ("combo", Binding::TwoWay { path, .. }) => {
                let (port, field) = self.port_field(decl, path)?;
                let enum_name = field.enum_name.clone().ok_or_else(|| {
                    Error::Unsupported(format!("combo on non-enum field `{}`", field.name))
                })?;
                let selected = if label.is_empty() {
                    "picked.label().to_owned()".to_owned()
                } else {
                    format!("format!(\"{label} {{}}\", picked.label())")
                };
                let _ = writeln!(out, "{pad}let mut picked = {port}.{}.get();", field.name);
                let _ = writeln!(
                    out,
                    "{pad}egui::ComboBox::from_id_salt(\"{salt}\")",
                    salt = format!("{}_{}", decl.snake, field.name),
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
                let _ = writeln!(out, "{pad}if picked != {port}.{}.get() {{", field.name);
                let _ = writeln!(out, "{pad}    {port}.{}.set(picked);", field.name);
                if self.theme.as_ref().map(|(_, f, _)| f.as_str()) == Some(field.name.as_str()) {
                    let _ = writeln!(out, "{pad}    picked.apply(ui.ctx());");
                }
                let _ = writeln!(out, "{pad}}}");
            }
            ("perspectives", Binding::None) => {
                // The 🗔 Perspectives menu — rendered by the host-owned
                // Workspace (it holds the dock state the menu manipulates).
                let _ = writeln!(out, "{pad}workspace.perspectives_menu(ui);");
            }
            ("terminal", Binding::None) => {
                // The shell prefab: scrollback + prompt + built-ins + bash
                // passthrough, all citizen-local (see TERMINAL_IMPL).
                let _ = writeln!(out, "{pad}self.terminal_ui(ui);");
            }
            ("logger", Binding::Read { path, .. }) => {
                // The egui_lens prefab: renders the reactive event log from
                // the bound interface's state + colors cells.
                let (port, _) = self.port_field(decl, path)?;
                let iface_name = decl
                    .ports
                    .iter()
                    .find(|pm| pm.name == port)
                    .map(|pm| pm.iface.clone())
                    .unwrap_or_default();
                let (_, state_field, colors_field) = self
                    .logger_iface()
                    .filter(|(i, _, _)| i.name == iface_name)
                    .ok_or_else(|| {
                        Error::Unsupported(
                            "logger needs an interface with LoggerState + LogColors fields"
                                .into(),
                        )
                    })?;
                let _ = writeln!(
                    out,
                    "{pad}egui_lens::ReactiveEventLogger::with_colors(&{port}.{state_field}, &{port}.{colors_field}).show(ui);"
                );
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

    /// `-> c.actions(RibbonCmd::New)` → `self.outbox.push(RibbonCmd::New);`
    fn render_event(&self, decl: &CitizenDeclModel, expr: &Expr) -> Result<String, Error> {
        let Expr::Call { path, args, .. } = expr else {
            return Err(Error::Unsupported("event binding must be a call".into()));
        };
        let (signal_port, signal_name, _) = self.decl_signal(decl).ok_or_else(|| {
            Error::Unsupported(format!("event fired but `{}` reaches no signal field", decl.ty))
        })?;
        match path.segments.as_slice() {
            [port, field] if *port == signal_port && *field == signal_name => {}
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

    fn emit_main(&self, opts: &Options) -> String {
        let has_dock = !self.docked.is_empty();
        let need_ctx = self.about.is_some() || !self.clock_writes.is_empty();
        let has_splits = self.docked.iter().any(|d| d.region != "center");
        let dock_use = if !has_dock {
            ""
        } else if has_splits {
            "use egui_dock::{DockArea, DockState, NodeIndex};\n"
        } else {
            "use egui_dock::{DockArea, DockState};\n"
        };
        let mut out = format!(
            "//! {app} — generated by mobius_codegen.\n\
             //!\n\
             //! Wiring follows the egui_mobius book: Path A state lives in the\n\
             //! `state::*` cells, citizens' Path B events drain through actions\n\
             //! once per frame, and the registry handles lifecycle only.\n\n\
             mod citizens;\n\
             mod state;\n\
             {mod_theme}\n\
             use eframe::egui;\n\
             use egui_citizen::Registry;\n{dock_use}\n",
            app = self.app_name,
            mod_theme = if self.theme.is_some() { "mod theme;\n" } else { "" },
        );

        // ── Tab enum (docked instances only) ────────────────────────────
        if has_dock {
            out.push_str(
                if self.has_perspectives_menu() {
                    "/// One dock tab per docked citizen instance.\n\
                     #[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]\n\
                     pub enum Tab {\n"
                } else {
                    "/// One dock tab per docked citizen instance.\n\
                     #[derive(Clone, Copy, PartialEq)]\n\
                     pub enum Tab {\n"
                },
            );
            for DockedModel { inst, .. } in &self.docked {
                let _ = writeln!(out, "    {},", camel_case(inst));
            }
            out.push_str(
                "}\n\nimpl Tab {\n    /// The instance id — registry name and citizen id.\n\
                 \x20   fn id(self) -> &'static str {\n        match self {\n",
            );
            for DockedModel { inst, .. } in &self.docked {
                let _ = writeln!(out, "            Tab::{} => \"{inst}\",", camel_case(inst));
            }
            out.push_str("        }\n    }\n}\n\n");
        }

        // Does a top-bar citizen carry the perspectives menu? Then the
        // dock lives in a Workspace with named layouts and a default.
        let has_persp = self.has_perspectives_menu();

        // The dock construction, built once — emitted inline in new() or
        // as default_layout() when perspectives need to rebuild it.
        let mut dock_build = String::new();
        if has_dock {
            let center: Vec<String> = self
                .docked
                .iter()
                .filter(|d| d.region == "center" && d.split.is_none())
                .map(|d| format!("Tab::{}", camel_case(&d.inst)))
                .collect();
            let _ = write!(
                dock_build,
                "\n        // Dock layout from the @layout dock() directives.\n\
                 \x20       let mut dock_state = DockState::new(vec![{}]);\n",
                center.join(", ")
            );
            for (region, method, fraction) in [
                ("left", "split_left", "0.30"),
                ("right", "split_right", "0.70"),
                ("bottom", "split_below", "0.65"),
            ] {
                let tabs: Vec<String> = self
                    .docked
                    .iter()
                    .filter(|d| d.region == region && d.split.is_none())
                    .map(|d| format!("Tab::{}", camel_case(&d.inst)))
                    .collect();
                if tabs.is_empty() {
                    continue;
                }
                // Sub-splits inside this region (`split = right/below`) need
                // the region's node index; a bare region discards it.
                let splits: Vec<(&str, String)> = self
                    .docked
                    .iter()
                    .filter(|d| d.region == region && d.split.is_some())
                    .map(|d| {
                        (
                            d.split.as_deref().unwrap(),
                            format!("Tab::{}", camel_case(&d.inst)),
                        )
                    })
                    .collect();
                if splits.is_empty() {
                    let _ = writeln!(
                        dock_build,
                        "        let _ = dock_state.main_surface_mut().{method}(NodeIndex::root(), {fraction}, vec![{}]);",
                        tabs.join(", ")
                    );
                } else {
                    let _ = writeln!(
                        dock_build,
                        "        let [_, {region}_node] = dock_state.main_surface_mut().{method}(NodeIndex::root(), {fraction}, vec![{}]);",
                        tabs.join(", ")
                    );
                    for (dir, split_method) in
                        [("right", "split_right"), ("below", "split_below")]
                    {
                        let split_tabs: Vec<&str> = splits
                            .iter()
                            .filter(|(d, _)| *d == dir)
                            .map(|(_, t)| t.as_str())
                            .collect();
                        if !split_tabs.is_empty() {
                            let _ = writeln!(
                                dock_build,
                                "        let _ = dock_state.main_surface_mut().{split_method}({region}_node, 0.5, vec![{}]);",
                                split_tabs.join(", ")
                            );
                        }
                    }
                }
            }
        }

        if has_persp {
            // Workspace + PerspectiveStore: the ForgePM perspectives.rs
            // shape (BTreeMap name -> DockState JSON, starred startup
            // default, persist on every mutation).
            out.push_str(&WORKSPACE_IMPL.replace("__CRATE__", &self.crate_name));
            out.push_str(
                "/// The @layout default — Reset returns here.\nfn default_layout() -> DockState<Tab> {\n",
            );
            for line in dock_build.lines() {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    let _ = writeln!(out, "{}", line.strip_prefix("    ").unwrap_or(line));
                }
            }
            out.push_str("    dock_state\n}\n\n");
        }

        // ── App struct ──────────────────────────────────────────────────
        out.push_str("struct App {\n    registry: Registry,\n");
        if has_persp {
            out.push_str("    workspace: Workspace,\n");
        } else if has_dock {
            out.push_str("    dock_state: DockState<Tab>,\n");
        }
        for (inst, iface) in &self.iface_insts {
            let _ = writeln!(out, "    {inst}: state::{iface},");
        }
        for inst in &self.insts {
            let _ = writeln!(out, "    {}: citizens::{},", inst.name, inst.decl);
        }
        out.push_str("}\n\nimpl App {\n    fn new() -> Self {\n        let mut registry = Registry::new();\n");
        for inst in &self.insts {
            let _ = writeln!(
                out,
                "        let {}_state = registry.add().with_name(\"{}\");",
                inst.name, inst.name
            );
        }
        if let Some(DockedModel { inst: first, .. }) = self.docked.first() {
            let _ = writeln!(out, "        registry.activate(\"{first}\");");
        }
        out.push_str("        let _ = registry.drain_messages(); // discard setup messages\n");
        if has_dock && !has_persp {
            out.push_str(&dock_build);
        }
        let logger_inst = self.logger_inst();
        out.push_str(if logger_inst.is_some() {
            "        let app = Self {\n            registry,\n"
        } else {
            "        Self {\n            registry,\n"
        });
        if has_persp {
            out.push_str("            workspace: Workspace::new(),\n");
        } else if has_dock {
            out.push_str("            dock_state,\n");
        }
        for (inst, iface) in &self.iface_insts {
            let _ = writeln!(out, "            {inst}: state::{iface}::new(),");
        }
        for inst in &self.insts {
            let _ = writeln!(
                out,
                "            {}: citizens::{}::with_id({}_state, \"{}\"),",
                inst.name, inst.decl, inst.name, inst.name
            );
        }
        if let Some(log_inst) = &logger_inst {
            out.push_str("        };\n");
            let _ = writeln!(out, "        startup::banner(&app.{log_inst});");
            out.push_str("        app\n    }\n}\n\n");
        } else {
            out.push_str("        }\n    }\n}\n\n");
        }

        // ── TabViewer (docked instances route by tab id) ─────────────────
        if has_dock {
            let mut used_ifaces: Vec<&str> = Vec::new();
            for DockedModel { inst: inst_name, .. } in &self.docked {
                let inst = self.insts.iter().find(|i| &i.name == inst_name).unwrap();
                for (_, iface_inst) in &inst.bindings {
                    if !used_ifaces.contains(&iface_inst.as_str()) {
                        used_ifaces.push(iface_inst);
                    }
                }
            }
            out.push_str("struct TabViewer<'a> {\n    registry: &'a mut Registry,\n");
            for iface_inst in &used_ifaces {
                let (_, iface) = self
                    .iface_insts
                    .iter()
                    .find(|(n, _)| n == iface_inst)
                    .unwrap();
                let _ = writeln!(out, "    {iface_inst}: &'a state::{iface},");
            }
            for DockedModel { inst: inst_name, .. } in &self.docked {
                let inst = self.insts.iter().find(|i| &i.name == inst_name).unwrap();
                let _ = writeln!(out, "    {}: &'a mut citizens::{},", inst.name, inst.decl);
            }
            out.push_str(
                "}\n\nimpl egui_dock::TabViewer for TabViewer<'_> {\n    type Tab = Tab;\n\n\
                 \x20   fn id(&mut self, tab: &mut Tab) -> egui::Id {\n\
                 \x20       egui::Id::new(tab.id())\n    }\n\n\
                 \x20   fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {\n\
                 \x20       tab.id().into()\n    }\n\n\
                 \x20   fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {\n        match tab {\n",
            );
            for DockedModel { inst: inst_name, .. } in &self.docked {
                let inst = self.insts.iter().find(|i| &i.name == inst_name).unwrap();
                let args: String = inst
                    .bindings
                    .iter()
                    .map(|(_, iface_inst)| format!(", self.{iface_inst}"))
                    .collect();
                let _ = writeln!(
                    out,
                    "            Tab::{} => self.{}.show(ui{args}),",
                    camel_case(inst_name),
                    inst.name
                );
            }
            out.push_str(
                "        }\n    }\n\n\
                 \x20   fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {\n\
                 \x20       if response.clicked() {\n\
                 \x20           self.registry.activate(tab.id());\n\
                 \x20       }\n    }\n}\n\n",
            );
        }

        // ── eframe::App ─────────────────────────────────────────────────
        out.push_str(
            "impl eframe::App for App {\n    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {\n",
        );
        if need_ctx {
            out.push_str("        let ctx = ui.ctx().clone();\n");
        }
        for (inst, field) in &self.clock_writes {
            let _ = writeln!(out, "        self.{inst}.{field}.set(wall_clock());");
        }
        if !self.clock_writes.is_empty() {
            out.push_str(
                "        ctx.request_repaint_after(std::time::Duration::from_millis(250));\n",
            );
        }
        for top_inst in &self.top {
            let inst = self.insts.iter().find(|i| &i.name == top_inst).unwrap();
            let mut args: String = inst
                .bindings
                .iter()
                .map(|(_, iface_inst)| format!(", &self.{iface_inst}"))
                .collect();
            let decl = self.decls.iter().find(|d| d.ty == inst.decl).unwrap();
            if decl_has_perspectives(decl) {
                args.push_str(", &mut self.workspace");
            }
            let _ = write!(
                out,
                "\n        egui::Panel::top(\"{name}\").show(ui, |ui| {{\n\
                 \x20           self.{name}.show(ui{args});\n\
                 \x20       }});\n",
                name = inst.name,
            );
        }
        if has_dock {
            out.push_str("\n        let mut viewer = TabViewer {\n            registry: &mut self.registry,\n");
            let mut used_ifaces: Vec<&str> = Vec::new();
            for DockedModel { inst: inst_name, .. } in &self.docked {
                let inst = self.insts.iter().find(|i| &i.name == inst_name).unwrap();
                for (_, iface_inst) in &inst.bindings {
                    if !used_ifaces.contains(&iface_inst.as_str()) {
                        used_ifaces.push(iface_inst);
                    }
                }
            }
            for iface_inst in &used_ifaces {
                let _ = writeln!(out, "            {iface_inst}: &self.{iface_inst},");
            }
            for DockedModel { inst: inst_name, .. } in &self.docked {
                let _ = writeln!(out, "            {inst_name}: &mut self.{inst_name},");
            }
            out.push_str(
                if has_persp {
                    "        };\n        DockArea::new(&mut self.workspace.dock).show_inside(ui, &mut viewer);\n"
                } else {
                    "        };\n        DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);\n"
                },
            );
        } else {
            let _ = write!(
                out,
                "        egui::CentralPanel::default().show(ui, |ui| {{\n\
                 \x20           ui.weak(\"Generated by mobius_codegen.\");\n\
                 \x20       }});\n",
            );
        }
        if let Some(log_inst) = self.logger_inst() {
            let _ = write!(
                out,
                "\n        // egui_lens's System Info button sets a temp-memory flag;\n\
                 \x20       // answering it is the host's job (per egui_lens's own comment).\n\
                 \x20       let info_id = egui::Id::new(\"show_system_info\");\n\
                 \x20       if ui.ctx().memory(|mem| mem.data.get_temp::<bool>(info_id).unwrap_or(false)) {{\n\
                 \x20           startup::system_info(&self.{log_inst});\n\
                 \x20           ui.ctx().memory_mut(|mem| mem.data.remove::<bool>(info_id));\n\
                 \x20       }}\n",
            );
        }
        if let Some(about) = &self.about {
            let _ = writeln!(out, "\n        about_window(&ctx, &self.{});", about.trigger.0);
        }
        for inst in &self.insts {
            let decl = self.decls.iter().find(|d| d.ty == inst.decl).unwrap();
            if let Some((signal_port, _, _)) = self.decl_signal(decl) {
                let (_, iface_inst) = inst
                    .bindings
                    .iter()
                    .find(|(p, _)| p == &signal_port)
                    .unwrap();
                let log_arg = self
                    .logger_inst()
                    .map(|l| format!(", &self.{l}"))
                    .unwrap_or_default();
                let _ = write!(
                    out,
                    "\n        // Path B: drain the outbox and route through actions.\n\
                     \x20       for cmd in std::mem::take(&mut self.{name}.outbox) {{\n\
                     \x20           actions::handle(cmd, &self.{iface_inst}{log_arg});\n\
                     \x20       }}\n",
                    name = inst.name,
                );
            }
        }
        out.push_str(
            "        // Registry lifecycle messages — drained after the panels show.\n\
             \x20       for _msg in self.registry.drain_messages() {}\n    }\n}\n",
        );

        if let Some(about) = &self.about {
            let banner = if about.image.is_some() {
                "            ui.vertical_centered(|ui| {\n\
                 \x20               ui.add_space(10.0);\n\
                 \x20               // Width tracks the window so horizontal resize sticks.\n\
                 \x20               ui.add(\n\
                 \x20                   egui::Image::new(egui::include_image!(\"../assets/about.png\"))\n\
                 \x20                       .max_size(egui::vec2(ui.available_width(), f32::INFINITY)),\n\
                 \x20               );\n\
                 \x20               ui.add_space(12.0);\n\
                 \x20           });\n"
            } else {
                ""
            };
            let deps_table = if about.deps {
                "            ui.add_space(8.0);\n\
                 \x20           ui.separator();\n\
                 \x20           ui.label(egui::RichText::new(\"Built with\").strong());\n\
                 \x20           ui.add_space(2.0);\n\
                 \x20           egui::Grid::new(\"about_deps\")\n\
                 \x20               .num_columns(3)\n\
                 \x20               .spacing([12.0, 2.0])\n\
                 \x20               .show(ui, |ui| {\n\
                 \x20                   for (name, version, what) in DEPENDENCIES {\n\
                 \x20                       ui.monospace(*name);\n\
                 \x20                       ui.weak(*version);\n\
                 \x20                       ui.weak(*what);\n\
                 \x20                       ui.end_row();\n\
                 \x20                   }\n\
                 \x20               });\n"
            } else {
                ""
            };
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
                 \x20       .resizable({resizable})\n\
                 \x20       .default_width(460.0)\n\
                 \x20       .pivot(egui::Align2::CENTER_CENTER)\n\
                 \x20       .default_pos(ctx.content_rect().center())\n\
                 \x20       .open(&mut open)\n\
                 \x20       .show(ctx, |ui| {{\n\
                 \x20           // Opens at full content height; shrinking it scrolls, and\n\
                 \x20           // the width-filling scroll area makes horizontal resize stick.\n\
                 \x20           egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {{\n\
                 {banner}\
                 \x20           ui.heading(\"{title}\");\n\
                 \x20           ui.label(egui::RichText::new(\"{tagline}\").italics());\n\
                 \x20           ui.add_space(8.0);\n\
                 \x20           ui.separator();\n\
                 \x20           ui.weak(format!(\"version {{}}\", env!(\"CARGO_PKG_VERSION\")));\n\
                 {deps_table}\
                 \x20           }});\n\
                 \x20       }});\n\
                 \x20   if !open {{\n\
                 \x20       {inst}.{trigger}.set(false);\n\
                 \x20   }}\n\
                 }}\n",
                inst = about.trigger.0,
                iface = self
                    .iface_insts
                    .iter()
                    .find(|(n, _)| n == &about.trigger.0)
                    .map(|(_, i)| i.as_str())
                    .unwrap_or_default(),
                trigger = about.trigger.1,
                title = about.title,
                tagline = about.tagline,
                resizable = about.resizable,
            );

            if about.deps {
                let mobius_v = mobius_version(&opts.mobius_root);
                let mut rows = vec![
                    ("egui_citizen".to_owned(), mobius_v.clone(), "citizen component model"),
                    ("egui_mobius_reactive".to_owned(), mobius_v, "Dynamic<T> reactive cells"),
                    ("egui / eframe".to_owned(), EGUI_VERSION.to_owned(), "immediate-mode UI"),
                ];
                if about.image.is_some() {
                    rows.push((
                        "egui_extras / image".to_owned(),
                        "0.36 / 0.25".to_owned(),
                        "PNG loading",
                    ));
                }
                out.push_str(
                    "\n/// `Built with` rows for the About window — regenerate or edit freely.\n\
                     const DEPENDENCIES: &[(&str, &str, &str)] = &[\n",
                );
                for (name, version, what) in rows {
                    let _ = writeln!(out, "    (\"{name}\", \"{version}\", \"{what}\"),");
                }
                out.push_str("];\n");
            }
        }

        if let Some(signal_iface) = self.ifaces.iter().find(|i| i.signal.is_some()) {
            let payload = &signal_iface.signal.as_ref().unwrap().1;
            let iface = &signal_iface.name;
            let param = snake_case(iface);
            let logger = self.logger_iface().map(|(i, s, c)| (i.name.clone(), s, c));
            let (log_param, log_use) = match &logger {
                Some((log_iface, _, _)) => (
                    format!(", log: &{log_iface}"),
                    format!(", {log_iface}"),
                ),
                None => (String::new(), String::new()),
            };
            let _ = write!(
                out,
                "\nmod actions {{\n\
                 \x20   //! Path B routing stubs — replace each arm with real work:\n\
                 \x20   //! spawn it, send it over a channel, write results back into\n\
                 \x20   //! `{iface}` cells.\n\n\
                 \x20   use crate::state::{{{iface}, {payload}{log_use}}};\n\n\
                 \x20   pub fn handle(cmd: {payload}, _{param}: &{iface}{log_param}) {{\n",
            );
            if let Some((_, state_field, colors_field)) = &logger {
                let _ = writeln!(
                    out,
                    "        let logger = egui_lens::ReactiveEventLogger::with_colors(&log.{state_field}, &log.{colors_field});"
                );
            }
            out.push_str("        match cmd {\n");
            let variants = self
                .enums
                .iter()
                .find(|e| &e.name == payload)
                .map(|e| e.variants.clone())
                .unwrap_or_default();
            for v in &variants {
                let act = if logger.is_some() {
                    format!("logger.log_info(\"[actions] {v}\")")
                } else {
                    format!("eprintln!(\"[actions] {v}\")")
                };
                let _ = writeln!(out, "            {payload}::{v} => {act},");
            }
            out.push_str("        }\n    }\n}\n");
        }

        let has_terminal_any = self.decls.iter().any(decl_has_terminal);
        if self.logger_iface().is_some() || has_terminal_any {
            let mobius_v = mobius_version(&opts.mobius_root);
            let mut rows = vec![
                ("egui_citizen", mobius_v.clone()),
                ("egui_mobius_reactive", mobius_v.clone()),
            ];
            if self.logger_iface().is_some() {
                rows.push(("egui_lens", mobius_v));
            }
            rows.push(("egui / eframe", EGUI_VERSION.to_owned()));
            if has_dock {
                rows.push(("egui_dock", "0.21".to_owned()));
            }
            let mut dep_lines = String::new();
            for (n, v) in &rows {
                let _ = writeln!(
                    dep_lines,
                    "        \"    {:<21}: {v}\\n\",",
                    format!("{n} ")
                );
            }
            // WELCOME and system_details are pub: the terminal prefab's
            // `ver` / `info` built-ins read them from the citizen modules.
            let _ = write!(
                out,
                "\nmod startup {{\n\
                 \x20   //! Identity & environment — the welcome/dependency banner and\n\
                 \x20   //! host system details (the CopperForge convention), shared by\n\
                 \x20   //! the logger banner and the terminal's `ver` / `info`.\n\n\
                 \x20   pub const WELCOME: &str = concat!(\n\
                 \x20       \"**** Welcome to {app}, version \", env!(\"CARGO_PKG_VERSION\"), \"\\n\\n\",\n\
                 \x20       \"    DEPENDENCIES\\n\",\n\
                 {dep_lines}\
                 \x20   );\n\n\
                 \x20   /// Linux: /etc/os-release + kernel version; elsewhere the OS constant.\n\
                 \x20   pub fn system_details() -> String {{\n\
                 \x20       let name = std::fs::read_to_string(\"/etc/os-release\")\n\
                 \x20           .ok()\n\
                 \x20           .and_then(|s| {{\n\
                 \x20               s.lines().find_map(|l| {{\n\
                 \x20                   l.strip_prefix(\"PRETTY_NAME=\").map(|v| v.trim_matches('\"').to_owned())\n\
                 \x20               }})\n\
                 \x20           }})\n\
                 \x20           .unwrap_or_else(|| std::env::consts::OS.to_owned());\n\
                 \x20       let kernel = std::fs::read_to_string(\"/proc/sys/kernel/osrelease\")\n\
                 \x20           .map(|s| s.trim().to_owned())\n\
                 \x20           .unwrap_or_else(|_| \"unknown\".to_owned());\n\
                 \x20       format!(\"SYSTEM DETAILS\\n\\n    SYSTEM\\n    System Name   : {{name}}\\n    System Kernel : {{kernel}}\")\n\
                 \x20   }}\n",
                app = self.app_name,
            );
            if let Some((iface, state_field, colors_field)) = self.logger_iface() {
                let _ = write!(
                    out,
                    "\n    use egui_lens::ReactiveEventLogger;\n\n\
                     \x20   pub fn banner(log: &crate::state::{iface_name}) {{\n\
                     \x20       let logger = ReactiveEventLogger::with_colors(&log.{state_field}, &log.{colors_field});\n\
                     \x20       logger.log_info(\"Startup: init\");\n\
                     \x20       logger.log_info(WELCOME);\n\
                     \x20       logger.log_info(&system_details());\n\
                     \x20   }}\n\n\
                     \x20   /// The System Info button: details first, banner last, so the\n\
                     \x20   /// banner reads on top in the circular buffer (CopperForge order).\n\
                     \x20   pub fn system_info(log: &crate::state::{iface_name}) {{\n\
                     \x20       let logger = ReactiveEventLogger::with_colors(&log.{state_field}, &log.{colors_field});\n\
                     \x20       logger.log_info(&system_details());\n\
                     \x20       logger.log_info(WELCOME);\n\
                     \x20   }}\n",
                    iface_name = iface.name,
                );
            }
            out.push_str("}\n");
        }

        if !self.clock_writes.is_empty() {
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

        let has_image = self.about.as_ref().is_some_and(|a| a.image.is_some());
        let creator = if self.theme.is_some() || has_image {
            let mut lines = String::new();
            if has_image {
                lines.push_str("            egui_extras::install_image_loaders(&cc.egui_ctx);\n");
            }
            let _ = writeln!(lines, "            let app = App::new();");
            if let Some((inst, field, _)) = &self.theme {
                let _ = writeln!(lines, "            app.{inst}.{field}.get().apply(&cc.egui_ctx);");
            }
            format!("Box::new(|cc| {{\n{lines}            Ok(Box::new(app))\n        }}),")
        } else {
            "Box::new(|_cc| Ok(Box::new(App::new()))),".to_owned()
        };
        let _ = write!(
            out,
            "\nfn main() -> Result<(), eframe::Error> {{\n\
             \x20   eframe::run_native(\n\
             \x20       \"{app}\",\n\
             \x20       eframe::NativeOptions {{\n\
             \x20           renderer: eframe::Renderer::Glow,\n\
             \x20           viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 420.0]),\n\
             \x20           ..Default::default()\n\
             \x20       }},\n\
             \x20       {creator}\n\
             \x20   )\n\
             }}\n",
            app = self.app_name,
        );
        out
    }
}

// ── helpers ─────────────────────────────────────────────────────────────

/// Match a theme variant name to one of the shipped palettes.
fn palette_for(variant: &str, dark_index: usize) -> &'static str {
    if variant.contains("Light") {
        "light"
    } else if variant.contains("Tokyo") {
        "tokyo"
    } else if variant.contains("Copper") || variant.contains("Discord") {
        "discord"
    } else if dark_index % 2 == 0 {
        "discord"
    } else {
        "tokyo"
    }
}

/// Discord-dark (CopperTrace's Copper Dark, lineage Forge-PM).
const PALETTE_DISCORD: &str = r#"/// Discord-dark palette (base #313338, sidebar #2B2D31, blurple #5865F2).
fn apply_discord(ctx: &egui::Context) {
    let base = Color32::from_rgb(0x31, 0x33, 0x38);
    let sidebar = Color32::from_rgb(0x2b, 0x2d, 0x31);
    let darkest = Color32::from_rgb(0x1e, 0x1f, 0x22);
    let hover = Color32::from_rgb(0x39, 0x3c, 0x41);
    let raised = Color32::from_rgb(0x38, 0x3a, 0x40);
    let fg = Color32::from_rgb(0xdb, 0xde, 0xe1);
    let fg_dim = Color32::from_rgb(0x94, 0x9b, 0xa4);
    let blurple = Color32::from_rgb(0x58, 0x65, 0xf2);
    let border = Color32::from_rgb(0x26, 0x28, 0x2c);

    let mut v = egui::Visuals::dark();
    v.panel_fill = base;
    v.window_fill = sidebar;
    v.faint_bg_color = raised;
    v.extreme_bg_color = darkest;
    v.selection.bg_fill = Color32::from_rgb(0x3e, 0x42, 0x70);
    v.selection.stroke = egui::Stroke::new(1.0, blurple);
    v.widgets.noninteractive.bg_fill = base;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_dim);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.bg_fill = raised;
    v.widgets.inactive.weak_bg_fill = raised;
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.hovered.bg_fill = hover;
    v.widgets.hovered.weak_bg_fill = hover;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, blurple);
    v.widgets.active.bg_fill = blurple;
    // `.strong()` text derives from this — keep it near-white.
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0xf2, 0xf3, 0xf5));
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, blurple);
    v.override_text_color = Some(fg);
    v.hyperlink_color = Color32::from_rgb(0x00, 0xa8, 0xfc);
    v.warn_fg_color = Color32::from_rgb(0xf0, 0xb2, 0x32);
    v.error_fg_color = Color32::from_rgb(0xf2, 0x3f, 0x43);
    v.window_stroke = egui::Stroke::new(1.0, border);
    ctx.set_visuals(v);
}
"#;

/// Tokyo Night Storm (CopperTrace / CopperForge lineage).
const PALETTE_TOKYO: &str = r#"/// Tokyo Night Storm palette.
fn apply_tokyo(ctx: &egui::Context) {
    let bg = Color32::from_rgb(0x24, 0x28, 0x3b);
    let bg_dark = Color32::from_rgb(0x1f, 0x23, 0x35);
    let bg_hi = Color32::from_rgb(0x2f, 0x33, 0x4d);
    let fg = Color32::from_rgb(0xc0, 0xca, 0xf5);
    let fg_dim = Color32::from_rgb(0xa9, 0xb1, 0xd6);
    let blue = Color32::from_rgb(0x7a, 0xa2, 0xf7);
    let cyan = Color32::from_rgb(0x7d, 0xcf, 0xff);
    let border = Color32::from_rgb(0x3b, 0x41, 0x61);
    let selection = Color32::from_rgb(0x2e, 0x3c, 0x64);

    let mut v = egui::Visuals::dark();
    v.panel_fill = bg;
    v.window_fill = bg_dark;
    v.faint_bg_color = bg_hi;
    v.extreme_bg_color = bg_dark;
    v.selection.bg_fill = selection;
    v.selection.stroke = egui::Stroke::new(1.0, blue);
    v.widgets.noninteractive.bg_fill = bg;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_dim);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.bg_fill = bg_hi;
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.hovered.bg_fill = selection;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, cyan);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, blue);
    // `.strong()` text derives from widgets.active — keep it light.
    v.widgets.active.bg_fill = blue;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, cyan);
    v.override_text_color = Some(fg);
    v.hyperlink_color = cyan;
    v.warn_fg_color = Color32::from_rgb(0xff, 0x9e, 0x64);
    v.error_fg_color = Color32::from_rgb(0xf7, 0x76, 0x8e);
    v.window_stroke = egui::Stroke::new(1.0, border);
    ctx.set_visuals(v);
}
"#;

/// Neutral grey "Light" (Forge-PM lineage — stock egui light is too bright).
const PALETTE_LIGHT: &str = r#"/// Medium-grey light palette (the stock egui Light is far too bright).
fn apply_light(ctx: &egui::Context) {
    let panel = Color32::from_rgb(0x4a, 0x4a, 0x4d);
    let window = Color32::from_rgb(0x3f, 0x3f, 0x42);
    let dark = Color32::from_rgb(0x33, 0x33, 0x36);
    let hi = Color32::from_rgb(0x5a, 0x5a, 0x5e);
    let fg = Color32::from_rgb(0xe6, 0xe6, 0xe6);
    let fg_dim = Color32::from_rgb(0xb0, 0xb0, 0xb4);
    let accent = Color32::from_rgb(0x8a, 0xb4, 0xf8);
    let border = Color32::from_rgb(0x6a, 0x6a, 0x6e);

    let mut v = egui::Visuals::dark();
    v.panel_fill = panel;
    v.window_fill = window;
    v.faint_bg_color = hi;
    v.extreme_bg_color = dark;
    v.selection.bg_fill = Color32::from_rgb(0x50, 0x62, 0x7a);
    v.selection.stroke = egui::Stroke::new(1.0, accent);
    v.widgets.noninteractive.bg_fill = panel;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_dim);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.inactive.bg_fill = hi;
    v.widgets.inactive.weak_bg_fill = hi;
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x66, 0x66, 0x6a);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x66, 0x66, 0x6a);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);
    // A dark-on-accent active style turns every `.strong()` label invisible.
    v.widgets.active.bg_fill = Color32::from_rgb(0x50, 0x62, 0x7a);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);
    v.override_text_color = Some(fg);
    v.hyperlink_color = accent;
    v.window_stroke = egui::Stroke::new(1.0, border);
    ctx.set_visuals(v);
}
"#;

/// Does the declaration contain the `terminal` prefab primitive?
fn decl_has_terminal(decl: &CitizenDeclModel) -> bool {
    decl_has_kind(decl, "terminal")
}

/// Does the declaration contain the `perspectives` menu primitive?
fn decl_has_perspectives(decl: &CitizenDeclModel) -> bool {
    decl_has_kind(decl, "perspectives")
}

fn decl_has_kind(decl: &CitizenDeclModel, kind: &str) -> bool {
    let mut found = false;
    scan_widgets(&decl.body, &mut |p| {
        if p.kind == kind {
            found = true;
        }
    });
    found
}

/// The Workspace + PerspectiveStore — ForgePM's `perspectives.rs` shape:
/// named dock layouts as serialized `DockState` JSON in a `BTreeMap`, a
/// starred startup default, persisted to
/// `~/.config/__CRATE__/perspectives.json` on every mutation.
const WORKSPACE_IMPL: &str = r#"/// The dock layout plus named perspectives — saveable, deletable, with
/// a ★ startup default, persisted across restarts.
pub struct Workspace {
    pub dock: DockState<Tab>,
    store: PerspectiveStore,
    name_buf: String,
}

/// name → serialized `DockState` JSON, plus the startup default
/// (ForgePM's perspectives.json, verbatim shape).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct PerspectiveStore {
    perspectives: std::collections::BTreeMap<String, String>,
    default: Option<String>,
}

impl PerspectiveStore {
    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("__CRATE__").join("perspectives.json"))
    }

    fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default()
    }

    fn persist(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(j) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, j);
        }
    }
}

impl Workspace {
    fn new() -> Self {
        let store = PerspectiveStore::load();
        // The ★ default loads on startup; otherwise the @layout default.
        let dock = store
            .default
            .as_ref()
            .and_then(|n| store.perspectives.get(n))
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_else(default_layout);
        Self {
            dock,
            store,
            name_buf: String::new(),
        }
    }

    /// The 🗔 Perspectives menu — ★ marks the startup default.
    ///
    /// Close-on-click-OUTSIDE, not egui's default close-on-any-click —
    /// the default kills the name text field the moment it is clicked.
    pub fn perspectives_menu(&mut self, ui: &mut egui::Ui) {
        let config = egui::containers::menu::MenuConfig::new()
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        egui::containers::menu::MenuButton::new("🗔 Perspectives")
            .config(config)
            .ui(ui, |ui| {
                if ui.button("Reset to default").clicked() {
                    self.dock = default_layout();
                    ui.close();
                }
                if !self.store.perspectives.is_empty() {
                    ui.separator();
                }
                let mut apply = None;
                let mut delete = None;
                let mut star = None;
                for (name, json) in &self.store.perspectives {
                    ui.horizontal(|ui| {
                        let starred = self.store.default.as_deref() == Some(name.as_str());
                        if ui
                            .small_button(if starred { "★" } else { "☆" })
                            .on_hover_text("Load this perspective at startup")
                            .clicked()
                        {
                            star = Some(name.clone());
                        }
                        if ui.button(name).clicked() {
                            apply = Some(json.clone());
                        }
                        if ui.small_button("✖").clicked() {
                            delete = Some(name.clone());
                        }
                    });
                }
                if let Some(name) = star {
                    // Toggle: starring the current default clears it.
                    self.store.default =
                        if self.store.default.as_deref() == Some(name.as_str()) {
                            None
                        } else {
                            Some(name)
                        };
                    self.store.persist();
                }
                if let Some(json) = apply {
                    if let Ok(layout) = serde_json::from_str(&json) {
                        self.dock = layout;
                    }
                    ui.close();
                }
                if let Some(name) = delete {
                    self.store.perspectives.remove(&name);
                    if self.store.default.as_deref() == Some(name.as_str()) {
                        self.store.default = None;
                    }
                    self.store.persist();
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.name_buf)
                            .desired_width(120.0)
                            .hint_text("name"),
                    );
                    let ok = !self.name_buf.trim().is_empty();
                    if ui.add_enabled(ok, egui::Button::new("💾 Save current")).clicked() {
                        if let Ok(json) = serde_json::to_string(&self.dock) {
                            self.store
                                .perspectives
                                .insert(self.name_buf.trim().to_owned(), json);
                            self.store.persist();
                        }
                        self.name_buf.clear();
                    }
                });
            });
    }
}

"#;

/// The shell prefab body — CopperTrace's `citizens/terminal.rs` shape:
/// scrolling colour-coded output, a `$` prompt, built-ins, `bash -c`
/// passthrough. Appended to the citizen module; `__TY__` / `__SNAKE__` /
/// `__APP__` are substituted at emission.
const TERMINAL_IMPL: &str = r#"
const PROMPT: &str = "$ ";

impl __TY__ {
    fn terminal_ui(&mut self, ui: &mut egui::Ui) {
        // Panel tone (faint_bg), not extreme_bg — near-black in dark themes.
        let frame = egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0);
        frame.show(ui, |ui| {
            let text_id = ui.make_persistent_id("__SNAKE___input");
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.output {
                        render_line(ui, line);
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(
                            egui::RichText::new(PROMPT)
                                .color(egui::Color32::from_rgb(0x9e, 0xce, 0x6a))
                                .monospace()
                                .strong(),
                        );
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.cmd_buf)
                                .id(text_id)
                                .desired_width(ui.available_width())
                                .font(egui::TextStyle::Monospace)
                                .frame(egui::Frame::NONE),
                        );
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let input = self.cmd_buf.trim().to_string();
                            if !input.is_empty() {
                                if input == "clear" || input == "cls" {
                                    self.output.clear();
                                } else {
                                    self.output.push(format!("{PROMPT}{input}"));
                                    let lines = execute_command(&input);
                                    self.output.extend(lines);
                                }
                            }
                            self.cmd_buf.clear();
                            ui.memory_mut(|m| m.request_focus(text_id));
                        }
                    });
                });
        });
    }
}

fn render_line(ui: &mut egui::Ui, line: &str) {
    let color = if line.starts_with("error:") {
        egui::Color32::from_rgb(0xf7, 0x76, 0x8e)
    } else if line.starts_with(PROMPT) || line.ends_with(':') {
        egui::Color32::from_rgb(0x9e, 0xce, 0x6a)
    } else {
        ui.visuals().text_color()
    };
    ui.label(egui::RichText::new(line).color(color).monospace());
}

/// Built-ins + `bash -c` passthrough (the CopperForge terminal contract).
/// Grow app subcommands here — this is scaffolding, not a sealed shell.
fn execute_command(input: &str) -> Vec<String> {
    let first = input.split_whitespace().next().unwrap_or("").to_lowercase();
    match first.as_str() {
        "help" | "?" => vec![
            "__APP__ terminal.".into(),
            "  ver              version + dependency pedigree".into(),
            "  info             host system details".into(),
            "  clear, cls       clear the terminal".into(),
            "  <anything else>  run it in bash".into(),
        ],
        "ver" | "version" => crate::startup::WELCOME
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|s| s.to_string())
            .collect(),
        "info" => crate::startup::system_details()
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => run_os_command(input),
    }
}

fn run_os_command(cmd: &str) -> Vec<String> {
    match std::process::Command::new("bash").arg("-c").arg(cmd).output() {
        Ok(output) => {
            let mut lines = Vec::new();
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                lines.push(line.to_string());
            }
            for line in String::from_utf8_lossy(&output.stderr).lines() {
                lines.push(format!("error: {line}"));
            }
            if lines.is_empty() && !output.status.success() {
                lines.push(format!("error: exit code {}", output.status));
            }
            lines
        }
        Err(e) => vec![format!("error: {e}")],
    }
}
"#;

fn scan_widgets(nodes: &[WidgetNode], f: &mut impl FnMut(&mobius_lang::ast::Primitive)) {
    for node in nodes {
        match node {
            WidgetNode::Container { children, .. } => scan_widgets(children, f),
            WidgetNode::For { body, .. } => scan_widgets(body, f),
            WidgetNode::Primitive(p) => f(p),
        }
    }
}

/// `dock(instance, region = left [, split = right]);`
fn parse_dock(args: &[DirectiveArg]) -> Result<DockedModel, Error> {
    let mut instance = None;
    let mut region = "center".to_owned();
    let mut split = None;
    for arg in args {
        match arg {
            DirectiveArg::Positional(Expr::Path(p)) => instance = Some(p.joined()),
            DirectiveArg::Named(named) if named.name == "region" => {
                let Expr::Path(p) = &named.value else {
                    return Err(Error::Unsupported("dock region must be a name".into()));
                };
                region = p.joined();
            }
            DirectiveArg::Named(named) if named.name == "split" => {
                let Expr::Path(p) = &named.value else {
                    return Err(Error::Unsupported("dock split must be a name".into()));
                };
                split = Some(p.joined());
            }
            _ => {
                return Err(Error::Unsupported(
                    "dock() takes `(instance, region = ..., split = ...)`".into(),
                ));
            }
        }
    }
    if !matches!(region.as_str(), "left" | "right" | "bottom" | "center") {
        return Err(Error::Unsupported(format!(
            "dock region `{region}` (expected left/right/bottom/center)"
        )));
    }
    if let Some(s) = &split {
        if !matches!(s.as_str(), "right" | "below") {
            return Err(Error::Unsupported(format!(
                "dock split `{s}` (expected right/below)"
            )));
        }
    }
    Ok(DockedModel {
        inst: instance.ok_or_else(|| Error::Unsupported("dock() needs an instance".into()))?,
        region,
        split,
    })
}

fn parse_about(
    args: &[DirectiveArg],
    iface_insts: &[(String, String)],
) -> Result<AboutModel, Error> {
    let mut trigger = None;
    let mut title = None;
    let mut tagline = None;
    let mut image = None;
    let mut deps = false;
    let mut resizable = true;
    for arg in args {
        let DirectiveArg::Named(named) = arg else {
            return Err(Error::Unsupported("about() takes named arguments".into()));
        };
        match (named.name.as_str(), &named.value) {
            ("trigger", Expr::Path(p)) => match p.segments.as_slice() {
                [inst, field] if iface_insts.iter().any(|(n, _)| n == inst) => {
                    trigger = Some((inst.clone(), field.clone()));
                }
                _ => {
                    return Err(Error::Unsupported(format!(
                        "about trigger `{}` (expected `<iface-instance>.<field>`)",
                        p.joined()
                    )));
                }
            },
            ("title", Expr::Str(s, _)) => title = Some(s.clone()),
            ("tagline", Expr::Str(s, _)) => tagline = Some(s.clone()),
            ("image", Expr::Str(s, _)) => image = Some(s.clone()),
            ("deps", Expr::Bool(b, _)) => deps = *b,
            ("resizable", Expr::Bool(b, _)) => resizable = *b,
            (other, _) => {
                return Err(Error::Unsupported(format!("about() argument `{other}`")));
            }
        }
    }
    Ok(AboutModel {
        trigger: trigger.ok_or_else(|| Error::Unsupported("about() needs trigger".into()))?,
        title: title.ok_or_else(|| Error::Unsupported("about() needs title".into()))?,
        tagline: tagline.unwrap_or_default(),
        image,
        deps,
        resizable,
    })
}

fn rust_type(mobius_ty: &str) -> String {
    match mobius_ty {
        "string" => "String".into(),
        // Builtin prefab types — the egui_lens logger pair.
        "LoggerState" => "ReactiveEventLoggerState".into(),
        other => other.into(),
    }
}

fn render_default(field: &str, ty: &str, default: Option<&Expr>) -> Result<String, Error> {
    let Some(expr) = default else {
        // Builtin types default themselves; everything else must say.
        return match ty {
            "LoggerState" => Ok("ReactiveEventLoggerState::new()".into()),
            "LogColors" => Ok("LogColors::default()".into()),
            _ => Err(Error::Unsupported(format!("field `{field}` needs a default (v0)"))),
        };
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

/// The `[workspace.package] version` of the egui_mobius checkout, for the
/// About window's dependency rows. Falls back to `"path"` when unreadable.
fn mobius_version(root: &Path) -> String {
    let Ok(manifest) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return "path".to_owned();
    };
    let mut in_workspace_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_workspace_package = line == "[workspace.package]";
        } else if in_workspace_package {
            if let Some(rest) = line.strip_prefix("version") {
                if let Some(v) = rest.split('"').nth(1) {
                    return v.to_owned();
                }
            }
        }
    }
    "path".to_owned()
}

/// `controls0` → `Controls0`, `my_scope` → `MyScope` — Tab variant names.
fn camel_case(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Render a slider-range literal as an f32-friendly number.
fn render_number(expr: &Expr) -> String {
    match expr {
        Expr::Int(i, _) => format!("{i}.0"),
        Expr::Float(f, _) => format!("{f:?}"),
        other => format!("/* unsupported range expr {other:?} */"),
    }
}

/// Split a CamelCase wordmark at its first hump: `"RibbonDemo"` →
/// `("Ribbon", "Demo")`. No hump → the whole label and an empty tail.
fn split_wordmark(label: &str) -> (&str, &str) {
    for (i, ch) in label.char_indices().skip(1) {
        if ch.is_uppercase() {
            return (&label[..i], &label[i..]);
        }
    }
    (label, "")
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
        let source = include_str!("../../../examples/mobius/ribbon/ribbon.mobius");
        let generated = generate(
            source,
            &Options {
                mobius_root: PathBuf::from("/tmp/mobius"),
                asset_root: Some(PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../examples/mobius/ribbon"
                ))),
                crate_name: None,
            },
        )
        .expect("example generates");
        assert!(generated.assets.contains_key("assets/about.png"));
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

    #[test]
    fn project_config_loads_and_resolves() {
        let example = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/mobius/ribbon"
        ));
        let config = load_project_config(&example.join("ribbon.mobius"))
            .expect("parses")
            .expect("example mobius.toml exists");
        // Relative paths resolve against the toml's directory and are
        // absolutized (they end up inside the generated Cargo.toml).
        let abs = |p: &str| std::path::absolute(example.join(p)).unwrap();
        assert_eq!(config.out, Some(abs("generated")));
        assert_eq!(config.asset_root, Some(abs(".")));
        assert_eq!(config.crate_name, None);
        // No sibling mobius.toml → None, not an error.
        assert!(load_project_config(Path::new("/nonexistent/x.mobius")).unwrap().is_none());
    }
}
