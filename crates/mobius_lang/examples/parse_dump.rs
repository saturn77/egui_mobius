//! Parse a `.mobius` file and dump the AST — the front half of
//! `--emit ast`.
//!
//! ```shell
//! cargo run -p mobius_lang --example parse_dump -- crates/mobius_lang/examples/signal_bench.mobius
//! ```

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: parse_dump <file.mobius>");
    let source = std::fs::read_to_string(&path).expect("failed to read source file");
    match mobius_lang::parse(&source) {
        Ok((file, comments)) => {
            println!("{file:#?}");
            eprintln!(
                "parsed OK: {} app(s), {} comment(s)",
                file.apps.len(),
                comments.len()
            );
        }
        Err(error) => {
            eprintln!("{path}:{error}");
            std::process::exit(1);
        }
    }
}
