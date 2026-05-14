//! The `.canvas` domain-specific language.
//!
//! The source file on disk is the authoritative artifact, and the GUI
//! is one structured editor for it. The same `Scene` round-trips
//! between disk text and in-memory model with no lossy serialization
//! step.
//!
//! Planned modules (not yet implemented):
//!
//! - `lexer` — `logos`-based tokenizer for the `.canvas` grammar.
//! - `parser` — hand-written recursive-descent parser yielding a
//!   [`crate::model::Scene`] (or a thin AST that lowers to one).
//! - `pretty` — pretty-printer that emits canonical `.canvas` text
//!   from a `Scene`. Roundtrip-stable.
//! - `error` — `thiserror`-based diagnostics with source spans.
//!
//! The roundtrip property is the load-bearing invariant:
//! `parse(pretty(parse(text)?)?)? == parse(text)?` for every valid input.

use crate::model::Scene;

/// Errors produced while parsing `.canvas` source. Stub — to be replaced
/// with a `thiserror`-driven enum once the parser exists.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parse `.canvas` source into a [`Scene`]. Not yet implemented.
pub fn parse(_source: &str) -> Result<Scene, ParseError> {
    Err(ParseError {
        message: "egui_grafica::lang::parse is not yet implemented".to_string(),
    })
}

/// Emit canonical `.canvas` text from a [`Scene`]. Not yet implemented.
pub fn pretty(_scene: &Scene) -> String {
    String::new()
}
