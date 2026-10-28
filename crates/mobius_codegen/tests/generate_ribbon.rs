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

    let source = include_str!("fixtures/ribbon.mobius");
    let generated = mobius_codegen::generate(
        source,
        &mobius_codegen::Options {
            mobius_root: mobius_root.clone(),
            asset_root: Some(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
            ),
        },
    )
    .expect("fixture generates");

    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&generated.crate_name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("clean previous output");
    }
    generated.write_to(&out).expect("write project");

    let output = Command::new("cargo")
        .arg("check")
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
