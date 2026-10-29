//! The third gate: the Workbench — a port-less terminal prefab citizen
//! alongside the ribbon, logger, and docked channels — must generate a
//! project that passes cargo check.

use std::path::Path;
use std::process::Command;

#[test]
fn terminal_project_cargo_checks() {
    let mobius_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate lives at <root>/crates/mobius_codegen")
        .to_path_buf();

    let source = include_str!("../../../examples/mobius/terminal/terminal.mobius");
    let generated = mobius_codegen::generate(
        source,
        &mobius_codegen::Options {
            mobius_root: mobius_root.clone(),
            asset_root: None,
            crate_name: None,
        },
    )
    .expect("terminal example generates");

    // The prefab invariants: port-less citizen, local state, bash
    // passthrough, and ver/info reaching the shared startup module.
    let shell = &generated.files["src/citizens/shell.rs"];
    assert!(shell.contains("pub fn show(&mut self, ui: &mut egui::Ui)"));
    assert!(shell.contains("cmd_buf: String = String::new(),"));
    assert!(shell.contains("std::process::Command::new(\"bash\")"));
    assert!(shell.contains("crate::startup::WELCOME"));
    let main = &generated.files["src/main.rs"];
    assert!(main.contains("Shell::with_id(shell_state, \"shell\")"));
    assert!(main.contains("Tab::Shell => self.shell.show(ui),"));
    // Ribbon actions route into the logger, not stderr.
    assert!(main.contains("logger.log_info(\"[actions] Build\")"));

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
        "generated workbench failed cargo check:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
