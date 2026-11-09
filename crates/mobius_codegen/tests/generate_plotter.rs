//! The plot-block gate: parse the plotter fixture, emit the project, and
//! `cargo check` it. SignalBench's shape on the generated path — the EDA
//! waveform view as declared source.

use std::path::Path;
use std::process::Command;

#[test]
fn plotter_project_cargo_checks() {
    let mobius_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate lives at <root>/crates/mobius_codegen")
        .to_path_buf();

    let source = include_str!("../../../examples/mobius/plotter/plotter.mobius");
    let generated = mobius_codegen::generate(
        source,
        &mobius_codegen::Options {
            mobius_root: mobius_root.clone(),
            asset_root: None,
            crate_name: None,
        },
    )
    .expect("plotter example generates");

    // The plot block: a Vec<f32> cell (negative literals included) becomes
    // a named egui_plot line, salted by the citizen's id; the egui_plot
    // dependency is gated in by the block's presence.
    let state = &generated.files["src/state.rs"];
    assert!(state.contains("pub trace: Dynamic<Vec<f32>>"));
    assert!(state.contains("vec![0.0, 0.31"));
    assert!(state.contains("-1.0"));
    let scope = &generated.files["src/citizens/scope.rs"];
    assert!(scope.contains("egui_plot::Plot::new((self.citizen_id.0.as_str(), \"plot\"))"));
    assert!(scope.contains("egui_plot::Line::new("));
    // `scale --` is the vertical gain, `show --` the channel select —
    // the controls' cells consumed by the view itself.
    assert!(scope.contains("let plot_scale = b.amplitude.get();"));
    assert!(scope.contains("let plot_show = b.enabled.get();"));
    assert!(scope.contains("if plot_show {"));
    assert!(scope.contains(".iter().map(|&y| y * plot_scale).collect::<Vec<f32>>()"));
    // `ymin`/`ymax`/`xmin`/`xmax` pin the window — the fixed graticule.
    assert!(scope.contains("let plot_y = b.ymin.get() as f64..=b.ymax.get() as f64;"));
    assert!(scope.contains("plot_ui.set_plot_bounds_y(plot_y.clone());"));
    assert!(scope.contains("plot_ui.set_plot_bounds_x(plot_x.clone());"));
    assert!(scope.contains("plot_ui.set_auto_bounds([false, false]);"));
    assert!(generated.files["Cargo.toml"].contains("egui_plot = \"0.37\""));

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
        "generated plotter failed cargo check:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
