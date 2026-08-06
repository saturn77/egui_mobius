//! Hand-written lexer for mobius-lang.
//!
//! Produces a flat token stream with spans. `//` comments are collected into
//! a side list (with spans) rather than discarded, so a future serializer can
//! round-trip source with comments intact.

use crate::error::{ParseError, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    ColonColon,
    Eq,
    Dot,
    DotDot,
    At,
    Lt,
    Gt,
    Plus,
    Minus,
    Star,
    /// `<->` two-way binding.
    ArrowBoth,
    /// `->` event binding.
    ArrowRight,
    /// `<-` read-only binding.
    ArrowLeft,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// A `//` comment, preserved for round-tripping.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub text: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

pub fn lex(source: &str) -> Result<LexOutput, ParseError> {
    Lexer::new(source).run()
}

struct Lexer<'s> {
    src: &'s [u8],
    pos: usize,
    line: u32,
    column: u32,
    tokens: Vec<Token>,
    comments: Vec<Comment>,
}

impl<'s> Lexer<'s> {
    fn new(source: &'s str) -> Self {
        Self {
            src: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn peek(&self) -> u8 {
        *self.src.get(self.pos).unwrap_or(&0)
    }

    fn peek_at(&self, offset: usize) -> u8 {
        *self.src.get(self.pos + offset).unwrap_or(&0)
    }

    fn bump(&mut self) -> u8 {
        let byte = self.peek();
        self.pos += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        byte
    }

    fn here(&self) -> Span {
        Span::new(self.pos, self.pos, self.line, self.column)
    }

    fn span_from(&self, start: Span) -> Span {
        Span::new(start.start, self.pos, start.line, start.column)
    }

    fn push(&mut self, kind: TokenKind, start: Span) {
        let span = self.span_from(start);
        self.tokens.push(Token { kind, span });
    }

    fn run(mut self) -> Result<LexOutput, ParseError> {
        loop {
            self.skip_whitespace_and_comments();
            let start = self.here();
            let byte = self.peek();
            if byte == 0 {
                self.push(TokenKind::Eof, start);
                return Ok(LexOutput {
                    tokens: self.tokens,
                    comments: self.comments,
                });
            }
            match byte {
                b'{' => {
                    self.bump();
                    self.push(TokenKind::LBrace, start);
                }
                b'}' => {
                    self.bump();
                    self.push(TokenKind::RBrace, start);
                }
                b'(' => {
                    self.bump();
                    self.push(TokenKind::LParen, start);
                }
                b')' => {
                    self.bump();
                    self.push(TokenKind::RParen, start);
                }
                b'[' => {
                    self.bump();
                    self.push(TokenKind::LBracket, start);
                }
                b']' => {
                    self.bump();
                    self.push(TokenKind::RBracket, start);
                }
                b',' => {
                    self.bump();
                    self.push(TokenKind::Comma, start);
                }
                b';' => {
                    self.bump();
                    self.push(TokenKind::Semi, start);
                }
                b'=' => {
                    self.bump();
                    self.push(TokenKind::Eq, start);
                }
                b'@' => {
                    self.bump();
                    self.push(TokenKind::At, start);
                }
                b'>' => {
                    self.bump();
                    self.push(TokenKind::Gt, start);
                }
                b':' => {
                    self.bump();
                    if self.peek() == b':' {
                        self.bump();
                        self.push(TokenKind::ColonColon, start);
                    } else {
                        self.push(TokenKind::Colon, start);
                    }
                }
                b'.' => {
                    self.bump();
                    if self.peek() == b'.' {
                        self.bump();
                        self.push(TokenKind::DotDot, start);
                    } else {
                        self.push(TokenKind::Dot, start);
                    }
                }
                b'<' => {
                    self.bump();
                    if self.peek() == b'-' {
                        self.bump();
                        if self.peek() == b'>' {
                            self.bump();
                            self.push(TokenKind::ArrowBoth, start);
                        } else {
                            self.push(TokenKind::ArrowLeft, start);
                        }
                    } else {
                        self.push(TokenKind::Lt, start);
                    }
                }
                b'-' => {
                    self.bump();
                    if self.peek() == b'>' {
                        self.bump();
                        self.push(TokenKind::ArrowRight, start);
                    } else {
                        // Subtraction / unary minus; negatives are formed in
                        // the parser, so the lexer never makes a signed number.
                        self.push(TokenKind::Minus, start);
                    }
                }
                b'+' => {
                    self.bump();
                    self.push(TokenKind::Plus, start);
                }
                b'*' => {
                    self.bump();
                    self.push(TokenKind::Star, start);
                }
                b'"' => self.string(start)?,
                b'0'..=b'9' => self.number(start)?,
                b'_' | b'a'..=b'z' | b'A'..=b'Z' => self.ident(start),
                other => {
                    return Err(ParseError::new(
                        format!("unexpected character `{}`", other as char),
                        self.span_from(start),
                    ));
                }
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.peek().is_ascii_whitespace() {
                self.bump();
            }
            if self.peek() == b'/' && self.peek_at(1) == b'/' {
                let start = self.here();
                let text_start = self.pos;
                while self.peek() != b'\n' && self.peek() != 0 {
                    self.bump();
                }
                let text = String::from_utf8_lossy(&self.src[text_start..self.pos]).into_owned();
                let span = self.span_from(start);
                self.comments.push(Comment { text, span });
            } else {
                return;
            }
        }
    }

    fn ident(&mut self, start: Span) {
        let begin = self.pos;
        while matches!(self.peek(), b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') {
            self.bump();
        }
        let text = String::from_utf8_lossy(&self.src[begin..self.pos]).into_owned();
        self.push(TokenKind::Ident(text), start);
    }

    fn string(&mut self, start: Span) -> Result<(), ParseError> {
        self.bump(); // opening quote
        let mut value = String::new();
        loop {
            match self.peek() {
                0 | b'\n' => {
                    return Err(ParseError::new(
                        "unterminated string literal",
                        self.span_from(start),
                    ));
                }
                b'"' => {
                    self.bump();
                    self.push(TokenKind::Str(value), start);
                    return Ok(());
                }
                b'\\' => {
                    self.bump();
                    match self.bump() {
                        b'"' => value.push('"'),
                        b'\\' => value.push('\\'),
                        b'n' => value.push('\n'),
                        other => {
                            return Err(ParseError::new(
                                format!("unsupported escape `\\{}`", other as char),
                                self.span_from(start),
                            ));
                        }
                    }
                }
                _ => {
                    let byte_start = self.pos;
                    while !matches!(self.peek(), 0 | b'\n' | b'"' | b'\\') {
                        self.bump();
                    }
                    value.push_str(&String::from_utf8_lossy(&self.src[byte_start..self.pos]));
                }
            }
        }
    }

    /// Lex an integer or float. Stops before `..` so ranges like `0.0..10.0`
    /// lex as `Float DotDot Float`.
    fn number(&mut self, start: Span) -> Result<(), ParseError> {
        let begin = self.pos;
        while self.peek().is_ascii_digit() {
            self.bump();
        }
        let mut is_float = false;
        if self.peek() == b'.' && self.peek_at(1) != b'.' && self.peek_at(1).is_ascii_digit() {
            is_float = true;
            self.bump(); // '.'
            while self.peek().is_ascii_digit() {
                self.bump();
            }
        }
        let text = String::from_utf8_lossy(&self.src[begin..self.pos]).into_owned();
        if is_float {
            let value: f64 = text
                .parse()
                .map_err(|_| ParseError::new("invalid float literal", self.span_from(start)))?;
            self.push(TokenKind::Float(value), start);
        } else {
            let value: i64 = text
                .parse()
                .map_err(|_| ParseError::new("invalid integer literal", self.span_from(start)))?;
            self.push(TokenKind::Int(value), start);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source)
            .unwrap()
            .tokens
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn range_lexes_as_float_dotdot_float() {
        assert_eq!(
            kinds("0.0..10.0"),
            vec![
                TokenKind::Float(0.0),
                TokenKind::DotDot,
                TokenKind::Float(10.0),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn arrows_disambiguate() {
        assert_eq!(
            kinds("<-> -> <- < >"),
            vec![
                TokenKind::ArrowBoth,
                TokenKind::ArrowRight,
                TokenKind::ArrowLeft,
                TokenKind::Lt,
                TokenKind::Gt,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn generics_and_paths() {
        assert_eq!(
            kinds("Vec<f32> BenchCmd::Apply bench.enabled"),
            vec![
                TokenKind::Ident("Vec".into()),
                TokenKind::Lt,
                TokenKind::Ident("f32".into()),
                TokenKind::Gt,
                TokenKind::Ident("BenchCmd".into()),
                TokenKind::ColonColon,
                TokenKind::Ident("Apply".into()),
                TokenKind::Ident("bench".into()),
                TokenKind::Dot,
                TokenKind::Ident("enabled".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn comments_are_preserved_with_spans() {
        let out = lex("// hello\nlet x : f32 = 1.0; // tail\n").unwrap();
        assert_eq!(out.comments.len(), 2);
        assert_eq!(out.comments[0].text, "// hello");
        assert_eq!(out.comments[0].span.line, 1);
        assert_eq!(out.comments[1].span.line, 2);
    }

    #[test]
    fn errors_carry_line_and_column() {
        let err = lex("let x = ^").unwrap_err();
        assert_eq!(err.span.line, 1);
        assert_eq!(err.span.column, 9);
    }
}
