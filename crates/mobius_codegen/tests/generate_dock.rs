//! The second-citizen gate: the SAME citizen declarations instantiated
//! twice (Controls ×2, Scope ×2 against Channel ×2), docked via @layout,
//! must generate a project that passes cargo check.

use std::path::Path;
use std::process::Command;

#[test]
fn dock_project_cargo_checks() {
    let mobius_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate lives at <root>/crates/mobius_codegen")
        .to_path_buf();

    let source = include_str!("../../../examples/dock/dock.mobius");
    let generated = mobius_codegen::generate(
        source,
        &mobius_codegen::Options {
            mobius_root: mobius_root.clone(),
            asset_root: None,
            crate_name: None,
        },
    )
    .expect("dock example generates");

    // Multi-instance invariants worth pinning before the compile check.
    let main = &generated.files["src/main.rs"];
    assert!(main.contains("Controls::with_id(controls0_state, \"controls0\")"));
    assert!(main.contains("Controls::with_id(controls1_state, \"controls1\")"));
    assert!(main.contains("Tab::Scope1 => self.scope1.show(ui, self.ch1),"));
    assert!(main.contains("startup::banner(&app.log);"));
    assert!(main.contains("startup::system_info(&self.log);"));
    assert!(main.contains("about_window(&ctx, &self.chrome);"));
    let citizens_mod = &generated.files["src/citizens/mod.rs"];
    assert!(citizens_mod.contains("pub mod controls;"));
    assert!(citizens_mod.contains("pub mod scope;"));
    assert!(citizens_mod.contains("pub mod system;"));
    assert!(generated.files["Cargo.toml"].contains("egui_lens"));
    assert!(generated.files["src/citizens/system.rs"]
        .contains("egui_lens::ReactiveEventLogger::with_colors(&l.state, &l.colors).show(ui);"));

    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&generated.crate_name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("clean previous output");
    }
    generated.write_to(&out).expect("write project");

    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&out)
        .env("CARGO_TARGET_DIR", out.join("target"))
        .output()
        .expect("run cargo check");

    assert!(
        output.status.success(),
        "generated dock project failed cargo check:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
