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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {e:?}"),
            Error::Unsupported(what) => write!(f, "unsupported: {what}"),
            Error::Asset(what) => write!(f, "asset: {what}"),
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
    files.insert("src/state.rs".into(), model.emit_state());
    if let Some(theme_rs) = model.emit_theme() {
        files.insert("src/theme.rs".into(), theme_rs);
    }
    files.insert("src/citizens/mod.rs".into(), model.emit_citizens_mod());
    files.insert(
        format!("src/citizens/{}.rs", model.citizen_snake),
        model.emit_citizen()?,
    );
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
# Glow renderer, not the default wgpu — resizing under wgpu showed frozen
# frames and black rectangles; CopperTrace ships the same choice.
eframe = {{ version = "{egui}", default-features = false, features = [
    "default_fonts",
    "glow",
    "wayland",
    "x11",
] }}
egui = "{egui}"{image_deps}
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
        let field = self.theme_field.as_ref()?;
        self.state_fields
            .iter()
            .find(|f| &f.name == field)
            .and_then(|f| f.enum_name.clone())
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
        let _ = writeln!(out, "        ui.add_space(3.0);");
        if self.chrome_needs_theme() {
            let _ = writeln!(
                out,
                "        let theme = {}.{}.get();",
                self.citizen_port,
                self.theme_field.as_deref().unwrap_or("theme"),
            );
        }
        let mut body = String::new();
        self.emit_widgets(&self.body, 2, &mut body)?;
        out.push_str(&body);
        out.push_str("    }\n}\n");
        Ok(out)
    }

    /// Does the citizen body contain chrome that reads `theme.brand()` /
    /// `theme.accent()`? (wordmark, clock, or the about-trigger button)
    fn chrome_needs_theme(&self) -> bool {
        if self.theme_field.is_none() {
            return false;
        }
        let trigger = self.about.as_ref().map(|a| a.trigger_field.as_str());
        let mut needs = false;
        scan_widgets(&self.body, &mut |p| match (p.kind.as_str(), &p.binding) {
            ("wordmark", _) | ("clock", _) => needs = true,
            ("button", Binding::TwoWay { path, .. }) => {
                if path.segments.last().map(String::as_str) == trigger {
                    needs = true;
                }
            }
            _ => {}
        });
        needs
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
                if self.theme_field.is_some() {
                    // Two-tone wordmark in the theme's brand colours, split
                    // at the camel hump (CopperTrace house style).
                    let (head, tail) = split_wordmark(label);
                    let _ = writeln!(out, "{pad}ui.scope(|ui| {{");
                    let _ = writeln!(out, "{pad}    ui.spacing_mut().item_spacing.x = 0.0;");
                    let _ = writeln!(out, "{pad}    let (brand_a, brand_b) = theme.brand();");
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
                let field = self.port_field(path)?;
                if p.kind == "clock" {
                    let style = if self.theme_field.is_some() {
                        ".monospace().color(theme.accent())"
                    } else {
                        ".monospace().weak()"
                    };
                    let _ = writeln!(
                        out,
                        "{pad}ui.label(egui::RichText::new({port}.{field}.get()){style});",
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
                let is_about = self.theme_field.is_some()
                    && self.about.as_ref().map(|a| a.trigger_field.as_str()) == Some(field.as_str());
                let button = if is_about {
                    format!("ui.button(egui::RichText::new(\"{label}\").color(theme.accent()))")
                } else {
                    format!("ui.button(\"{label}\")")
                };
                let _ = writeln!(out, "{pad}if {button}.clicked() {{");
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
                if self.theme_field.as_deref() == Some(field.as_str()) {
                    let _ = writeln!(out, "{pad}    picked.apply(ui.ctx());");
                }
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

    fn emit_main(&self, opts: &Options) -> String {
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
             mod state;\n\
             {mod_theme}\n\
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
            mod_theme = if self.theme_field.is_some() { "mod theme;\n" } else { "" },
        );

        if let Some(clock) = &self.clock_field {
            let _ = writeln!(out, "        self.{inst}.{clock}.set(wall_clock());");
            let _ = writeln!(
                out,
                "        ctx.request_repaint_after(std::time::Duration::from_millis(250));"
            );
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
                trigger = about.trigger_field,
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

        let has_image = self.about.as_ref().is_some_and(|a| a.image.is_some());
        let creator = if self.theme_field.is_some() || has_image {
            let mut lines = String::new();
            if has_image {
                lines.push_str("            egui_extras::install_image_loaders(&cc.egui_ctx);\n");
            }
            let _ = writeln!(lines, "            let app = App::new();");
            if let Some(theme) = &self.theme_field {
                let _ = writeln!(lines, "            app.{inst}.{theme}.get().apply(&cc.egui_ctx);");
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
    let mut image = None;
    let mut deps = false;
    let mut resizable = true;
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
            ("image", Expr::Str(s, _)) => image = Some(s.clone()),
            ("deps", Expr::Bool(b, _)) => deps = *b,
            ("resizable", Expr::Bool(b, _)) => resizable = *b,
            (other, _) => {
                return Err(Error::Unsupported(format!("about() argument `{other}`")));
            }
        }
    }
    Ok(AboutModel {
        trigger_field: trigger.ok_or_else(|| Error::Unsupported("about() needs trigger".into()))?,
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
        let source = include_str!("../tests/fixtures/ribbon.mobius");
        let generated = generate(
            source,
            &Options {
                mobius_root: PathBuf::from("/tmp/mobius"),
                asset_root: Some(PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures"
                ))),
            },
        )
        .expect("fixture generates");
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
}
