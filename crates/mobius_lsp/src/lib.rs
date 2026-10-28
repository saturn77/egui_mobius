//! # mobius_lsp
//!
//! Diagnostics for `.mobius` sources, reusing `mobius_lang`'s parser —
//! the same front end that feeds runtime interpretation
//! (`mobius_lang_host`) and project generation (`mobius_codegen`).
//!
//! **Tier 1 only (parse errors).** Tier 2 — `lower()`'s semantic
//! diagnostics (one-writer check, unknown fields) — is deliberately not
//! wired yet: `lower()` hardcodes a closed directive/widget vocabulary
//! (`lower.rs` rejects "unknown directive"), while `mobius_codegen`
//! treats those as open sets (`top()`, `about()`, `wordmark`, …).
//! Wiring it today would mark valid codegen sources as broken. The
//! vocabularies must be reconciled first — see mobius_codegen/ROADMAP.md.

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Byte-offset → LSP position (UTF-16 columns, per the protocol default).
pub struct LineIndex {
    /// Byte offset of each line start; `[0]` is always 0.
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn position(&self, text: &str, offset: usize) -> Position {
        let mut offset = offset.min(text.len());
        while offset > 0 && !text.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let start = self.line_starts[line];
        let character = text[start..offset].encode_utf16().count() as u32;
        Position {
            line: line as u32,
            character,
        }
    }

    fn range(&self, text: &str, span: mobius_lang::Span) -> Range {
        let start = self.position(text, span.start);
        // A zero-width or inverted span still needs a visible squiggle:
        // extend at least one column past the start.
        let end = if span.end > span.start {
            self.position(text, span.end)
        } else {
            Position {
                line: start.line,
                character: start.character + 1,
            }
        };
        Range { start, end }
    }
}

/// Diagnostics for one document's full text. The recovering parser
/// reports *every* error, not just the first — a typo at the top of the
/// file no longer blanks out diagnostics below it.
pub fn diagnostics(text: &str) -> Vec<Diagnostic> {
    let index = LineIndex::new(text);
    mobius_lang::parse_with_recovery(text)
        .errors
        .into_iter()
        .map(|e| Diagnostic {
            range: index.range(text, e.span),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("mobius".to_owned()),
            message: e.message,
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_source_has_no_diagnostics() {
        let source = include_str!("../../../examples/mobius/ribbon/ribbon.mobius");
        assert!(diagnostics(source).is_empty());
    }

    #[test]
    fn parse_error_maps_to_a_ranged_diagnostic() {
        // Missing `;` after the widget.
        let source = "app A {\n    citizen C (p : I.v) {\n        button \"x\" -> p.s(E::V)\n    }\n}\n";
        let diags = diagnostics(source);
        assert_eq!(diags.len(), 1);
        let d = &diags[0];
        assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
        // The parser stops somewhere in the citizen body — the range must
        // land on a real line of the source, not line 0.
        assert!(d.range.start.line >= 2, "range: {:?}", d.range);
    }

    #[test]
    fn multiple_errors_all_reported() {
        // An error in the citizen body AND one in @wiring — recovery means
        // the second is no longer hidden behind the first.
        let source = "app A {\n    citizen C (p : I.v) {\n        button \"x\" <-> p.s\n    }\n    @wiring {\n        let i = I()\n        let c = C(p = i.v);\n    }\n}\n";
        let diags = diagnostics(source);
        assert!(diags.len() >= 2, "got {} diagnostics: {diags:?}", diags.len());
    }

    #[test]
    fn utf16_columns() {
        // '🎨' is 4 UTF-8 bytes but 2 UTF-16 units; the byte offset after
        // it must map to character 3 (quote + two surrogate halves).
        let text = "\"🎨\" x";
        let index = LineIndex::new(text);
        let after_emoji_quote = 1 + 4 + 1; // `"` + emoji + `"`
        let pos = index.position(text, after_emoji_quote);
        assert_eq!((pos.line, pos.character), (0, 4));
    }

    #[test]
    fn multiline_positions() {
        let text = "ab\ncd\nef";
        let index = LineIndex::new(text);
        assert_eq!(index.position(text, 0), Position { line: 0, character: 0 });
        assert_eq!(index.position(text, 4), Position { line: 1, character: 1 });
        assert_eq!(index.position(text, 99), Position { line: 2, character: 2 });
    }
}
