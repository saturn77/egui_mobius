//! Parse and lower a `.mobius` file, then print the IR netlist — the
//! `--emit ir` stage of the pipeline.
//!
//! ```shell
//! cargo run -p mobius_lang --example emit_ir -- crates/mobius_lang/examples/signal_bench.mobius
//! ```

use mobius_lang::Registry;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: emit_ir <file.mobius>");
    let source = std::fs::read_to_string(&path).expect("failed to read source file");

    // Stand-in for the host binary's registrations; the real registry is
    // built in Rust with typed constructors and port tables.
    let registry = Registry::new()
        .citizen("PlotPanel")
        .citizen("QuillEditor")
        .citizen("LensLogger")
        .handler("bench_worker")
        .event("BenchCmd");

    let (file, _comments) = match mobius_lang::parse(&source) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{path}:{error}");
            std::process::exit(1);
        }
    };

    let mut failed = false;
    for app in &file.apps {
        match mobius_lang::lower(app, &registry) {
            Ok(ir) => print!("{}", ir.emit_text()),
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{path}:{diagnostic}");
                }
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}
