//! `mobius-gen <file.mobius> <out-dir> [egui_mobius-root]` — generate a
//! compilable cargo project from a `.mobius` source file. The third
//! argument overrides where the generated manifest path-depends on
//! `egui_citizen` / `egui_mobius_reactive`; it defaults to the checkout
//! this binary was built from.

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(out_dir)) = (args.next(), args.next()) else {
        eprintln!("usage: mobius-gen <file.mobius> <out-dir> [egui_mobius-root]");
        std::process::exit(2);
    };
    let mobius_root = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crate lives at <root>/crates/mobius_codegen")
            .to_path_buf()
    });

    let source = match std::fs::read_to_string(&input) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("mobius-gen: cannot read {input}: {e}");
            std::process::exit(1);
        }
    };
    let asset_root = std::path::Path::new(&input)
        .parent()
        .map(std::path::Path::to_path_buf);
    let generated = match mobius_codegen::generate(
        &source,
        &mobius_codegen::Options { mobius_root, asset_root },
    ) {
        Ok(generated) => generated,
        Err(e) => {
            eprintln!("mobius-gen: {e}");
            std::process::exit(1);
        }
    };
    let out_dir = PathBuf::from(out_dir);
    if let Err(e) = generated.write_to(&out_dir) {
        eprintln!("mobius-gen: cannot write {}: {e}", out_dir.display());
        std::process::exit(1);
    }
    for rel in generated.files.keys() {
        println!("  {}", out_dir.join(rel).display());
    }
    println!("generated `{}` — try: cargo run", generated.crate_name);
}
