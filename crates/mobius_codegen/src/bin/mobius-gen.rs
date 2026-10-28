//! `mobius-gen <file.mobius> [out-dir] [egui_mobius-root]` — generate a
//! compilable cargo project from a `.mobius` source file.
//!
//! Compiler directives come from a `mobius.toml` next to the source (see
//! [`mobius_codegen::ProjectConfig`]); explicit arguments win over it, and
//! it wins over the defaults (egui_mobius root = the checkout this binary
//! was built from, asset root = the source's directory).

use std::path::PathBuf;

fn fail(msg: &str) -> ! {
    eprintln!("mobius-gen: {msg}");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: mobius-gen <file.mobius> [out-dir] [egui_mobius-root]");
        std::process::exit(2);
    };
    let input = PathBuf::from(input);
    let cli_out = args.next().map(PathBuf::from);
    let cli_root = args.next().map(PathBuf::from);

    let config = match mobius_codegen::load_project_config(&input) {
        Ok(config) => config.unwrap_or_default(),
        Err(e) => fail(&e.to_string()),
    };

    // Precedence: CLI > mobius.toml > defaults.
    let Some(out_dir) = cli_out.or(config.out) else {
        fail("no output dir: pass one, or set `out` in a mobius.toml next to the source");
    };
    let mobius_root = cli_root.or(config.mobius_root).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crate lives at <root>/crates/mobius_codegen")
            .to_path_buf()
    });
    let asset_root = config
        .asset_root
        .or_else(|| input.parent().map(std::path::Path::to_path_buf));

    let source = match std::fs::read_to_string(&input) {
        Ok(source) => source,
        Err(e) => fail(&format!("cannot read {}: {e}", input.display())),
    };
    // Compiler-style pre-flight: report every parse error, not just the
    // first (generate() itself re-parses; that costs microseconds).
    let parsed = mobius_lang::parse_with_recovery(&source);
    if !parsed.errors.is_empty() {
        for e in &parsed.errors {
            eprintln!("{}:{}: error: {}", input.display(), e.span, e.message);
        }
        fail(&format!("{} parse error(s)", parsed.errors.len()));
    }
    let options = mobius_codegen::Options {
        mobius_root,
        asset_root,
        crate_name: config.crate_name,
    };
    let generated = match mobius_codegen::generate(&source, &options) {
        Ok(generated) => generated,
        Err(e) => fail(&e.to_string()),
    };
    if let Err(e) = generated.write_to(&out_dir) {
        fail(&format!("cannot write {}: {e}", out_dir.display()));
    }
    for rel in generated.files.keys().chain(generated.assets.keys()) {
        println!("  {}", out_dir.join(rel).display());
    }
    println!("generated `{}` — try: cargo run", generated.crate_name);
}
