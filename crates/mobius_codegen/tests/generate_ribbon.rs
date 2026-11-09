//! The crate's gate: parse the ribbon fixture, emit the project, and
//! `cargo check` it. If this stays green while the language grows, the
//! codegen idea is working.

use std::path::Path;
use std::process::Command;

#[test]
fn ribbon_project_cargo_checks() {
    let mobius_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate lives at <root>/crates/mobius_codegen")
        .to_path_buf();

    let source = include_str!("../../../examples/mobius/ribbon/ribbon.mobius");
    let generated = mobius_codegen::generate(
        source,
        &mobius_codegen::Options {
            mobius_root: mobius_root.clone(),
            asset_root: Some(mobius_root.join("examples/mobius/ribbon")),
            crate_name: None,
        },
    )
    .expect("example generates");

    // The menu block: `title` names the dropdown button, the entries render
    // inside the popup — and the headless test asserts the title (always on
    // screen), never the popup entries (hidden until clicked).
    let ribbon = &generated.files["src/citizens/ribbon.rs"];
    assert!(ribbon.contains("ui.menu_button(\"🖽 Layout\", |ui| {"));
    assert!(ribbon.contains("if ui.button(\"Side by side\").clicked() {"));
    let main = &generated.files["src/main.rs"];
    assert!(main.contains("query_all_by_label(\"🖽 Layout\")"));
    assert!(!main.contains("query_all_by_label(\"Side by side\")"));

    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&generated.crate_name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("clean previous output");
    }
    generated.write_to(&out).expect("write project");

    let output = Command::new("cargo")
        .arg("check")
        // --all-targets: the generated headless test module must compile
        // too, not just the binary.
        .arg("--all-targets")
        .arg("--quiet")
        .current_dir(&out)
        // Its own target dir: the parent `cargo test` may still hold the
        // workspace target lock, and the generated project must build as a
        // genuinely standalone crate anyway.
        .env("CARGO_TARGET_DIR", out.join("target"))
        .output()
        .expect("run cargo check");

    assert!(
        output.status.success(),
        "generated project failed cargo check:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
