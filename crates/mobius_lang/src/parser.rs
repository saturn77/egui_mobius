//! Recursive-descent parser for mobius-lang.
//!
//! Grammar (v0, the baseline subset):
//!
//! ```text
//! file      := app*
//! app       := 'app' IDENT '{' item* '}'
//! item      := value | interface | citizen | section
//! value     := 'let' IDENT ':' type ['=' expr] ';'
//! interface := 'interface' IDENT '{' field* modport* '}'
//! field     := IDENT ':' ('signal' type | type ['=' expr])
//! modport   := 'modport' IDENT '(' verb IDENT (',' verb IDENT)* ')'
//! verb      := 'out' | 'in' | 'emit' | 'drain'
//! citizen   := 'citizen' IDENT '(' [port (',' port)*] ')' '{' widget* '}'
//! port      := IDENT ':' type
//! widget    := IDENT '{' widget* '}'
//!            | IDENT [STR] [expr '..' expr] [binding] ';'
//! binding   := '<->' path | '<-' path | '->' expr
//! section   := '@' IDENT '{' stmt* '}'
//! stmt      := 'let' IDENT (':' type ['=' expr] | '=' path '(' args ')') ';'
//!            | 'bind' STR '=' path ';'
//!            | IDENT '(' dirargs ')' ';'
//! type      := path ['<' type (',' type)* '>']
//! path      := IDENT (('.' | '::') IDENT)*
//! expr      := INT | FLOAT | STR | 'true' | 'false' | array | path ['(' expr,* ')']
//! ```

use crate::ast::*;
use crate::error::{ParseError, Span};
use crate::lexer::{Comment, LexOutput, Token, TokenKind, lex};

/// Parse a `.mobius` source string, returning the AST and preserved comments.
pub fn parse(source: &str) -> Result<(SourceFile, Vec<Comment>), ParseError> {
    let LexOutput { tokens, comments } = lex(source)?;
    let file = Parser::new(tokens).file()?;
    Ok((file, comments))
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError::new(message, self.peek().span)
    }

    fn expect(&mut self, kind: &TokenKind, what: &str) -> Result<Token, ParseError> {
        if &self.peek().kind == kind {
            Ok(self.bump())
        } else {
            Err(self.error(format!("expected {what}, found {:?}", self.peek().kind)))
        }
    }

    fn ident(&mut self, what: &str) -> Result<(String, Span), ParseError> {
        match self.peek().kind.clone() {
            TokenKind::Ident(name) => {
                let span = self.peek().span;
                self.bump();
                Ok((name, span))
            }
            other => Err(self.error(format!("expected {what}, found {other:?}"))),
        }
    }

    /// True if the next token is the identifier `word` (contextual keyword).
    fn at_word(&self, word: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(name) if name == word)
    }

    fn eat_word(&mut self, word: &str) -> bool {
        if self.at_word(word) {
            self.bump();
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------------
    // File / app
    // ------------------------------------------------------------------

    fn file(&mut self) -> Result<SourceFile, ParseError> {
        let mut apps = Vec::new();
        while !self.at_eof() {
            if self.at_word("app") {
                apps.push(self.app()?);
            } else {
                return Err(self.error("expected `app` at top level"));
            }
        }
        Ok(SourceFile { apps })
    }

    fn app(&mut self) -> Result<App, ParseError> {
        let start = self.peek().span;
        self.bump(); // `app`
        let (name, _) = self.ident("application name")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut items = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            items.push(self.app_item()?);
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        Ok(App {
            name,
            items,
            span: start.to(end),
        })
    }

    fn app_item(&mut self) -> Result<AppItem, ParseError> {
        if self.at_word("let") {
            Ok(AppItem::Value(self.value_decl()?))
        } else if self.at_word("enum") {
            Ok(AppItem::Enum(self.enum_decl()?))
        } else if self.at_word("interface") {
            Ok(AppItem::Interface(self.interface()?))
        } else if self.at_word("citizen") {
            Ok(AppItem::Citizen(self.citizen()?))
        } else if matches!(self.peek().kind, TokenKind::At) {
            Ok(AppItem::Section(self.section()?))
        } else {
            Err(self.error(
                "expected `let`, `enum`, `interface`, `citizen`, or a `@section` inside `app`",
            ))
        }
    }

    /// `enum Name { A, B, C }` — comma-separated variants, trailing comma ok.
    fn enum_decl(&mut self) -> Result<EnumDecl, ParseError> {
        let start = self.peek().span;
        self.bump(); // `enum`
        let (name, _) = self.ident("enum name")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut variants = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            let (variant, _) = self.ident("variant name")?;
            variants.push(variant);
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
            } else {
                break;
            }
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        if variants.is_empty() {
            return Err(self.error("enum must have at least one variant"));
        }
        Ok(EnumDecl {
            name,
            variants,
            span: start.to(end),
        })
    }

    // ------------------------------------------------------------------
    // Values & types
    // ------------------------------------------------------------------

    /// `let name : type [= expr] ;` — caller has seen `let`.
    fn value_decl(&mut self) -> Result<ValueDecl, ParseError> {
        let start = self.peek().span;
        self.bump(); // `let`
        let (name, _) = self.ident("value name")?;
        self.expect(&TokenKind::Colon, "`:`")?;
        let ty = self.type_ref()?;
        let default = if matches!(self.peek().kind, TokenKind::Eq) {
            self.bump();
            Some(self.expr()?)
        } else {
            None
        };
        let end = self.expect(&TokenKind::Semi, "`;`")?.span;
        Ok(ValueDecl {
            name,
            ty,
            default,
            span: start.to(end),
        })
    }

    fn type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let path = self.path()?;
        let start = path.span;
        let mut params = Vec::new();
        let mut end = path.span;
        if matches!(self.peek().kind, TokenKind::Lt) {
            self.bump();
            loop {
                params.push(self.type_ref()?);
                match self.peek().kind {
                    TokenKind::Comma => {
                        self.bump();
                    }
                    TokenKind::Gt => break,
                    _ => return Err(self.error("expected `,` or `>` in type parameters")),
                }
            }
            end = self.expect(&TokenKind::Gt, "`>`")?.span;
        }
        Ok(TypeRef {
            path,
            params,
            span: start.to(end),
        })
    }

    fn path(&mut self) -> Result<Path, ParseError> {
        let (first, start) = self.ident("a name")?;
        let mut segments = vec![first];
        let mut end = start;
        while matches!(self.peek().kind, TokenKind::Dot | TokenKind::ColonColon) {
            self.bump();
            let (segment, span) = self.ident("a path segment")?;
            segments.push(segment);
            end = span;
        }
        Ok(Path {
            segments,
            span: start.to(end),
        })
    }

    // ------------------------------------------------------------------
    // Interfaces
    // ------------------------------------------------------------------

    fn interface(&mut self) -> Result<Interface, ParseError> {
        let start = self.peek().span;
        self.bump(); // `interface`
        let (name, _) = self.ident("interface name")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut fields = Vec::new();
        let mut modports = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            if self.at_word("modport") {
                modports.push(self.modport()?);
            } else if modports.is_empty() {
                fields.push(self.field()?);
            } else {
                return Err(self.error("fields must precede modports in an interface"));
            }
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        Ok(Interface {
            name,
            fields,
            modports,
            span: start.to(end),
        })
    }

    /// `name : type [= expr]` or `name : signal Type` — newline-terminated
    /// (no semicolon), matching the interface examples in the spec.
    fn field(&mut self) -> Result<Field, ParseError> {
        let (name, start) = self.ident("field name")?;
        self.expect(&TokenKind::Colon, "`:`")?;
        if self.eat_word("signal") {
            let ty = self.type_ref()?;
            let span = start.to(ty.span);
            Ok(Field {
                name,
                kind: FieldKind::Signal { ty },
                span,
            })
        } else if matches!(self.peek().kind, TokenKind::LBracket) {
            // `[type; N]` array field
            self.bump(); // `[`
            let elem = self.type_ref()?;
            self.expect(&TokenKind::Semi, "`;` in array type")?;
            let len = match self.peek().kind {
                TokenKind::Int(value) if value > 0 => {
                    self.bump();
                    value as usize
                }
                _ => return Err(self.error("array length must be a positive integer")),
            };
            let mut end = self.expect(&TokenKind::RBracket, "`]`")?.span;
            let default = if matches!(self.peek().kind, TokenKind::Eq) {
                self.bump();
                let (array_default, default_end) = self.array_default()?;
                end = default_end;
                Some(array_default)
            } else {
                None
            };
            Ok(Field {
                name,
                kind: FieldKind::Array { elem, len, default },
                span: start.to(end),
            })
        } else {
            let ty = self.type_ref()?;
            let mut end = ty.span;
            let default = if matches!(self.peek().kind, TokenKind::Eq) {
                self.bump();
                let expr = self.expr()?;
                end = expr.span();
                Some(expr)
            } else {
                None
            };
            Ok(Field {
                name,
                kind: FieldKind::State { ty, default },
                span: start.to(end),
            })
        }
    }

    fn modport(&mut self) -> Result<Modport, ParseError> {
        let start = self.peek().span;
        self.bump(); // `modport`
        let (name, _) = self.ident("modport name")?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut entries = Vec::new();
        loop {
            let verb_span = self.peek().span;
            let verb = if self.eat_word("out") {
                Verb::Out
            } else if self.eat_word("in") {
                Verb::In
            } else if self.eat_word("emit") {
                Verb::Emit
            } else if self.eat_word("drain") {
                Verb::Drain
            } else {
                return Err(self.error("expected modport verb `out`, `in`, `emit`, or `drain`"));
            };
            let (field, field_span) = self.ident("field name")?;
            entries.push(ModportEntry {
                verb,
                field,
                span: verb_span.to(field_span),
            });
            match self.peek().kind {
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::RParen => break,
                _ => return Err(self.error("expected `,` or `)` in modport")),
            }
        }
        let end = self.expect(&TokenKind::RParen, "`)`")?.span;
        Ok(Modport {
            name,
            entries,
            span: start.to(end),
        })
    }

    // ------------------------------------------------------------------
    // Citizens from primitives
    // ------------------------------------------------------------------

    fn citizen(&mut self) -> Result<CitizenDecl, ParseError> {
        let start = self.peek().span;
        self.bump(); // `citizen`
        let (name, _) = self.ident("citizen name")?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut ports = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RParen) {
            let (port_name, port_start) = self.ident("port name")?;
            self.expect(&TokenKind::Colon, "`:`")?;
            let ty = self.type_ref()?;
            let span = port_start.to(ty.span);
            ports.push(PortParam {
                name: port_name,
                ty,
                span,
            });
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect(&TokenKind::RParen, "`)`")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            body.push(self.widget()?);
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        Ok(CitizenDecl {
            name,
            ports,
            body,
            span: start.to(end),
        })
    }

    fn widget(&mut self) -> Result<WidgetNode, ParseError> {
        if self.at_word("for") {
            return self.for_loop();
        }
        let (kind, start) = self.ident("widget or container name")?;
        if matches!(self.peek().kind, TokenKind::LBrace) {
            self.bump();
            let mut children = Vec::new();
            while !matches!(self.peek().kind, TokenKind::RBrace) {
                children.push(self.widget()?);
            }
            let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
            return Ok(WidgetNode::Container {
                kind,
                children,
                span: start.to(end),
            });
        }

        let label = match self.peek().kind.clone() {
            TokenKind::Str(text) => {
                self.bump();
                Some(text)
            }
            _ => None,
        };

        let range = if matches!(self.peek().kind, TokenKind::Float(_) | TokenKind::Int(_)) {
            let lo = self.expr()?;
            self.expect(&TokenKind::DotDot, "`..` in range")?;
            let hi = self.expr()?;
            Some((lo, hi))
        } else {
            None
        };

        let binding = match self.peek().kind {
            TokenKind::ArrowBoth => {
                self.bump();
                let (path, index) = self.indexed_path()?;
                Binding::TwoWay { path, index }
            }
            TokenKind::ArrowLeft => {
                self.bump();
                let (path, index) = self.indexed_path()?;
                Binding::Read { path, index }
            }
            TokenKind::ArrowRight => {
                self.bump();
                Binding::Event(self.expr()?)
            }
            _ => Binding::None,
        };

        let end = self.expect(&TokenKind::Semi, "`;` after widget")?.span;
        Ok(WidgetNode::Primitive(Box::new(Primitive {
            kind,
            label,
            range,
            binding,
            span: start.to(end),
        })))
    }

    /// `for VAR in LO..HI { widget* }`
    fn for_loop(&mut self) -> Result<WidgetNode, ParseError> {
        let start = self.peek().span;
        self.bump(); // `for`
        let (var, _) = self.ident("loop variable")?;
        if !self.eat_word("in") {
            return Err(self.error("expected `in` after loop variable"));
        }
        let lo = self.int_literal("loop start")?;
        self.expect(&TokenKind::DotDot, "`..` in loop range")?;
        let hi = self.int_literal("loop end")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            body.push(self.widget()?);
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        Ok(WidgetNode::For {
            var,
            lo,
            hi,
            body,
            span: start.to(end),
        })
    }

    fn int_literal(&mut self, what: &str) -> Result<i64, ParseError> {
        match self.peek().kind {
            TokenKind::Int(value) => {
                self.bump();
                Ok(value)
            }
            _ => Err(self.error(format!("expected an integer for {what}"))),
        }
    }

    /// A binding path with an optional `[index]`: `b.selected` or
    /// `b.selected[i]`. The index is an integer literal or a loop variable.
    fn indexed_path(&mut self) -> Result<(Path, Option<Expr>), ParseError> {
        let path = self.path()?;
        let index = if matches!(self.peek().kind, TokenKind::LBracket) {
            self.bump();
            let expr = self.expr()?;
            self.expect(&TokenKind::RBracket, "`]`")?;
            Some(expr)
        } else {
            None
        };
        Ok((path, index))
    }

    /// `{expr}` (broadcast) or `{e0, e1, ...}` (per-element).
    fn array_default(&mut self) -> Result<(ArrayDefault, Span), ParseError> {
        self.expect(&TokenKind::LBrace, "`{` array initializer")?;
        let mut items = vec![self.expr()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.bump();
            items.push(self.expr()?);
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        let default = if items.len() == 1 {
            ArrayDefault::Broadcast(items.into_iter().next().unwrap())
        } else {
            ArrayDefault::Elements(items)
        };
        Ok((default, end))
    }

    // ------------------------------------------------------------------
    // Sections
    // ------------------------------------------------------------------

    fn section(&mut self) -> Result<Section, ParseError> {
        let start = self.expect(&TokenKind::At, "`@`")?.span;
        let (name, _) = self.ident("section name")?;
        self.expect(&TokenKind::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            stmts.push(self.stmt()?);
        }
        let end = self.expect(&TokenKind::RBrace, "`}`")?.span;
        Ok(Section {
            name,
            stmts,
            span: start.to(end),
        })
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.at_word("let") {
            let start = self.peek().span;
            // Distinguish `let x : type = ...;` (value) from
            // `let x = Type(...);` (instance) by the token after the name.
            let checkpoint = self.pos;
            self.bump(); // `let`
            let (name, _) = self.ident("binding name")?;
            match self.peek().kind {
                TokenKind::Colon => {
                    self.pos = checkpoint;
                    Ok(Stmt::Value(self.value_decl()?))
                }
                TokenKind::Eq => {
                    self.bump();
                    let ty = self.path()?;
                    self.expect(&TokenKind::LParen, "`(`")?;
                    let mut args = Vec::new();
                    while !matches!(self.peek().kind, TokenKind::RParen) {
                        let (arg_name, arg_start) = self.ident("argument name")?;
                        self.expect(&TokenKind::Eq, "`=`")?;
                        let value = self.expr()?;
                        let span = arg_start.to(value.span());
                        args.push(NamedArg {
                            name: arg_name,
                            value,
                            span,
                        });
                        if matches!(self.peek().kind, TokenKind::Comma) {
                            self.bump();
                        }
                    }
                    self.expect(&TokenKind::RParen, "`)`")?;
                    let end = self.expect(&TokenKind::Semi, "`;`")?.span;
                    Ok(Stmt::Instance {
                        name,
                        ty,
                        args,
                        span: start.to(end),
                    })
                }
                _ => Err(self.error("expected `:` or `=` after `let name`")),
            }
        } else if self.at_word("bind") {
            let start = self.peek().span;
            self.bump(); // `bind`
            let handler = match self.peek().kind.clone() {
                TokenKind::Str(text) => {
                    self.bump();
                    text
                }
                _ => return Err(self.error("expected handler name string after `bind`")),
            };
            self.expect(&TokenKind::Eq, "`=`")?;
            let view = self.path()?;
            let end = self.expect(&TokenKind::Semi, "`;`")?.span;
            Ok(Stmt::Bind {
                handler,
                view,
                span: start.to(end),
            })
        } else {
            // Directive: `dock(plot, region = center);`
            let (name, start) = self.ident("directive name")?;
            self.expect(&TokenKind::LParen, "`(`")?;
            let mut args = Vec::new();
            while !matches!(self.peek().kind, TokenKind::RParen) {
                // `ident = expr` is named; anything else is positional.
                let named = matches!(self.peek().kind, TokenKind::Ident(_))
                    && matches!(
                        self.tokens.get(self.pos + 1).map(|t| &t.kind),
                        Some(TokenKind::Eq)
                    );
                if named {
                    let (arg_name, arg_start) = self.ident("argument name")?;
                    self.bump(); // `=`
                    let value = self.expr()?;
                    let span = arg_start.to(value.span());
                    args.push(DirectiveArg::Named(NamedArg {
                        name: arg_name,
                        value,
                        span,
                    }));
                } else {
                    args.push(DirectiveArg::Positional(self.expr()?));
                }
                if matches!(self.peek().kind, TokenKind::Comma) {
                    self.bump();
                }
            }
            self.expect(&TokenKind::RParen, "`)`")?;
            let end = self.expect(&TokenKind::Semi, "`;`")?.span;
            Ok(Stmt::Directive {
                name,
                args,
                span: start.to(end),
            })
        }
    }

    // ------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------

    fn expr(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek().kind.clone() {
            TokenKind::Int(value) => {
                self.bump();
                Ok(Expr::Int(value, span))
            }
            TokenKind::Float(value) => {
                self.bump();
                Ok(Expr::Float(value, span))
            }
            TokenKind::Str(value) => {
                self.bump();
                Ok(Expr::Str(value, span))
            }
            TokenKind::LBracket => {
                self.bump();
                let mut items = Vec::new();
                while !matches!(self.peek().kind, TokenKind::RBracket) {
                    items.push(self.expr()?);
                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.bump();
                    }
                }
                let end = self.expect(&TokenKind::RBracket, "`]`")?.span;
                Ok(Expr::Array(items, span.to(end)))
            }
            TokenKind::Ident(word) if word == "true" || word == "false" => {
                self.bump();
                Ok(Expr::Bool(word == "true", span))
            }
            TokenKind::Ident(_) => {
                let path = self.path()?;
                if matches!(self.peek().kind, TokenKind::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    while !matches!(self.peek().kind, TokenKind::RParen) {
                        args.push(self.expr()?);
                        if matches!(self.peek().kind, TokenKind::Comma) {
                            self.bump();
                        }
                    }
                    let end = self.expect(&TokenKind::RParen, "`)`")?.span;
                    let span = path.span.to(end);
                    Ok(Expr::Call { path, args, span })
                } else {
                    Ok(Expr::Path(path))
                }
            }
            other => Err(self.error(format!("expected an expression, found {other:?}"))),
        }
    }
}
