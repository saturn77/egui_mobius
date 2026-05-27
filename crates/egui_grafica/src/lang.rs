//! The `.canvas` domain-specific language.
//!
//! The source file on disk is the authoritative artifact; the GUI is one
//! structured editor for it. A [`Scene`] round-trips between disk text and
//! the in-memory model with no lossy serialization step.
//!
//! ## Syntax — keyword-block
//!
//! Line-oriented: one statement per line, each led by a keyword; compound
//! things (`settings`, `node`, `text`, `wire`) are `{ }` blocks. Chosen for
//! autocomplete-friendliness — "what is valid at the cursor" is computable
//! from the enclosing block plus the line's leading keyword.
//!
//! ```text
//! canvas "QuadCluster" {
//!   settings {
//!     grid 20
//!     routing orthogonal
//!   }
//!   node power_board : rect {
//!     at 120 200
//!     size 120 220
//!     border solid 2 "#1F2937"
//!     fill "#DBEAFE" 0.9
//!     text {
//!       value "BUCK BOARD"
//!       anchor center
//!       font "Inter" 12
//!       bold off
//!       italic off
//!       color "#111827"
//!     }
//!     port out ch1 east 0.2
//!   }
//!   wire e1 power_board.ch1 -> sense_board.ch1 {
//!     routing orthogonal
//!     stroke "#2196F3" 2 solid
//!     arrow arrow none
//!   }
//! }
//! ```
//!
//! ## v1 scope
//!
//! - Node kinds: `rect`, `circle`, `ellipse`. `Path` / `Group` nodes are
//!   printed as `rect` (lossy) and not yet parseable.
//! - `Routing::Manual { mid_offset }` round-trips as `routing manual <n>`.
//! - `Scene::groups` is not yet expressed.
//! - `#` line comments: comments leading a top-level item (the file header,
//!   `settings`, a `node`, a `wire`) survive a round-trip via
//!   [`parse_document`] / [`pretty_document`]. Comments *inside* a block, and
//!   trailing comments before a closing `}`, are not yet preserved. The bare
//!   [`parse`] / [`pretty`] pair discards all comments.

use crate::model::{
    ArrowHead, Border, CanvasBackground, CanvasSettings, Edge, EdgeEnd, EdgeId, EdgeOverlay, Fill,
    GridStyle, GridUnits, LineStyle, Node, NodeId, NodeKind, Overlay, Port, PortAnchor, PortId,
    PortKind, Routing, Scene, Style, TextAnchor, TextLabel, Transform,
};

// =============================================================================
// Errors
// =============================================================================

/// A parse failure, with the 1-based source line it occurred on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

// =============================================================================
// Comment-preserving document
// =============================================================================

/// A parsed `.canvas` document: the [`Scene`] plus the comment blocks
/// authored against its top-level items.
///
/// `parse`/`pretty` deal in bare `Scene` and discard comments. A GUI that
/// wants a load → edit → save cycle to preserve authored comments uses
/// [`parse_document`] / [`pretty_document`] and carries the
/// `ParsedDocument` instead.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedDocument {
    pub scene: Scene,
    pub comments: Vec<CommentBlock>,
}

/// A run of consecutive comment lines anchored to one place in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentBlock {
    pub anchor: CommentAnchor,
    /// Comment lines, verbatim text after the `#` (without the `#` itself).
    pub lines: Vec<String>,
}

/// Where a [`CommentBlock`] sits. v1 anchors comments to top-level items
/// only; comments inside a block, and trailing comments before `}`, are
/// not preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentAnchor {
    /// Before the `canvas` keyword — the file header.
    Header,
    /// Immediately before the `settings` block.
    Settings,
    /// Immediately before the node with this id.
    Node(NodeId),
    /// Immediately before the wire with this id.
    Wire(EdgeId),
}

// =============================================================================
// Public API
// =============================================================================

/// Parse `.canvas` source into a [`Scene`], discarding comments.
pub fn parse(source: &str) -> Result<Scene, ParseError> {
    Ok(parse_document(source)?.scene)
}

/// Parse `.canvas` source into a [`ParsedDocument`] — a [`Scene`] plus the
/// comment blocks anchored to its top-level items.
pub fn parse_document(source: &str) -> Result<ParsedDocument, ParseError> {
    let tokens = lex(source)?;
    Parser::new(tokens).parse_document()
}

/// Emit canonical `.canvas` text for a [`Scene`]. Round-trip stable:
/// `parse(&pretty(&s))` reconstructs an equal `Scene`.
pub fn pretty(scene: &Scene) -> String {
    Printer::new(&[]).print(scene)
}

/// Emit canonical `.canvas` text for a [`ParsedDocument`], re-emitting each
/// comment block before the item it is anchored to. Round-trip stable:
/// `parse_document(&pretty_document(&d))` reconstructs an equal document.
pub fn pretty_document(doc: &ParsedDocument) -> String {
    Printer::new(&doc.comments).print(&doc.scene)
}

/// Attach accumulated comments to `anchor`, clearing the pending buffer.
fn attach(comments: &mut Vec<CommentBlock>, anchor: CommentAnchor, pending: &mut Vec<String>) {
    if !pending.is_empty() {
        comments.push(CommentBlock { anchor, lines: std::mem::take(pending) });
    }
}

// =============================================================================
// Lexer
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Str(String),
    Num(f64),
    LBrace,
    RBrace,
    Colon,
    Dot,
    Arrow,
    Newline,
    /// A `#` line comment — the verbatim text after the `#`.
    Comment(String),
}

#[derive(Debug, Clone)]
struct Lexed {
    tok: Tok,
    line: usize,
}

fn lex(src: &str) -> Result<Vec<Lexed>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut line = 1usize;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\r' => i += 1,
            '\n' => {
                out.push(Lexed { tok: Tok::Newline, line });
                line += 1;
                i += 1;
            }
            '#' => {
                // `# …` — original DSL comment form, kept for
                // back-compat. Body runs to end of line.
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                out.push(Lexed { tok: Tok::Comment(text), line });
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Rust-style line comments: `//`, `///`, `//!`. The
                // extra slash / bang is stripped here — all three
                // forms collapse to a plain comment with the same
                // payload, so the pretty-printer can emit a single
                // canonical form on save.
                i += 2;
                if i < chars.len() && (chars[i] == '/' || chars[i] == '!') {
                    i += 1;
                }
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                out.push(Lexed { tok: Tok::Comment(text), line });
            }
            '{' => {
                out.push(Lexed { tok: Tok::LBrace, line });
                i += 1;
            }
            '}' => {
                out.push(Lexed { tok: Tok::RBrace, line });
                i += 1;
            }
            ':' => {
                out.push(Lexed { tok: Tok::Colon, line });
                i += 1;
            }
            '.' => {
                out.push(Lexed { tok: Tok::Dot, line });
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                loop {
                    if i >= chars.len() {
                        return Err(ParseError { line, message: "unterminated string".into() });
                    }
                    match chars[i] {
                        '"' => {
                            i += 1;
                            break;
                        }
                        '\\' => {
                            i += 1;
                            if i >= chars.len() {
                                return Err(ParseError { line, message: "unterminated escape".into() });
                            }
                            match chars[i] {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                '"' => s.push('"'),
                                '\\' => s.push('\\'),
                                other => s.push(other),
                            }
                            i += 1;
                        }
                        ch => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                out.push(Lexed { tok: Tok::Str(s), line });
            }
            '-' if i + 1 < chars.len() && chars[i + 1] == '>' => {
                out.push(Lexed { tok: Tok::Arrow, line });
                i += 2;
            }
            '-' | '0'..='9' => {
                let start = i;
                if chars[i] == '-' {
                    i += 1;
                }
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n: f64 = text
                    .parse()
                    .map_err(|_| ParseError { line, message: format!("invalid number '{text}'") })?;
                out.push(Lexed { tok: Tok::Num(n), line });
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                out.push(Lexed { tok: Tok::Ident(text), line });
            }
            other => {
                return Err(ParseError {
                    line,
                    message: format!("unexpected character '{other}'"),
                });
            }
        }
    }

    Ok(out)
}

// =============================================================================
// Parser
// =============================================================================

struct Parser {
    toks: Vec<Lexed>,
    pos: usize,
}

impl Parser {
    fn new(toks: Vec<Lexed>) -> Self {
        Self { toks, pos: 0 }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|l| &l.tok)
    }

    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or_else(|| self.toks.last())
            .map(|l| l.line)
            .unwrap_or(1)
    }

    fn err<T>(&self, msg: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError { line: self.line(), message: msg.into() })
    }

    /// Skip newlines and comments. Used inside blocks, where comments are not
    /// preserved (v1 keeps only the comments leading top-level items).
    fn skip_trivia(&mut self) {
        while matches!(self.peek(), Some(Tok::Newline) | Some(Tok::Comment(_))) {
            self.pos += 1;
        }
    }

    fn peek_comment(&self) -> Option<String> {
        match self.peek() {
            Some(Tok::Comment(c)) => Some(c.clone()),
            _ => None,
        }
    }

    fn at_newline(&self) -> bool {
        matches!(self.peek(), Some(Tok::Newline))
    }

    fn at_rbrace(&self) -> bool {
        matches!(self.peek(), Some(Tok::RBrace))
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Tok::Ident(s)) => {
                let s = s.clone();
                self.pos += 1;
                Ok(s)
            }
            _ => self.err("expected identifier"),
        }
    }

    fn string(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Tok::Str(s)) => {
                let s = s.clone();
                self.pos += 1;
                Ok(s)
            }
            _ => self.err("expected string"),
        }
    }

    fn number(&mut self) -> Result<f32, ParseError> {
        match self.peek() {
            Some(Tok::Num(n)) => {
                let n = *n as f32;
                self.pos += 1;
                Ok(n)
            }
            _ => self.err("expected number"),
        }
    }

    fn expect(&mut self, tok: &Tok, what: &str) -> Result<(), ParseError> {
        if self.peek() == Some(tok) {
            self.pos += 1;
            Ok(())
        } else {
            self.err(format!("expected {what}"))
        }
    }

    fn expect_kw(&mut self, kw: &str) -> Result<(), ParseError> {
        match self.peek() {
            Some(Tok::Ident(s)) if s == kw => {
                self.pos += 1;
                Ok(())
            }
            _ => self.err(format!("expected keyword '{kw}'")),
        }
    }

    /// After a statement: the next token must end the line (Newline) or close
    /// the block (RBrace). Catches trailing garbage.
    fn end_statement(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(Tok::Newline) | Some(Tok::Comment(_)) => {
                self.skip_trivia();
                Ok(())
            }
            Some(Tok::RBrace) | None => Ok(()),
            _ => self.err("unexpected token at end of statement"),
        }
    }

    // ── Document ─────────────────────────────────────────────────────────

    fn parse_document(&mut self) -> Result<ParsedDocument, ParseError> {
        let mut comments: Vec<CommentBlock> = Vec::new();

        // Header: comments appearing before the `canvas` keyword.
        let mut header: Vec<String> = Vec::new();
        loop {
            if let Some(c) = self.peek_comment() {
                header.push(c);
                self.pos += 1;
            } else if self.at_newline() {
                self.pos += 1;
            } else {
                break;
            }
        }
        if !header.is_empty() {
            comments.push(CommentBlock { anchor: CommentAnchor::Header, lines: header });
        }

        self.expect_kw("canvas")?;
        let name = self.string()?;
        self.expect(&Tok::LBrace, "'{'")?;

        let mut scene = Scene { name, ..Scene::default() };
        // Comments seen since the last item — attached to the next one parsed.
        let mut pending: Vec<String> = Vec::new();

        loop {
            // Collect leading trivia; comments accumulate into `pending`.
            loop {
                if let Some(c) = self.peek_comment() {
                    pending.push(c);
                    self.pos += 1;
                } else if self.at_newline() {
                    self.pos += 1;
                } else {
                    break;
                }
            }

            if self.at_rbrace() {
                self.pos += 1;
                break;
            }

            match self.peek() {
                Some(Tok::Ident(kw)) => match kw.as_str() {
                    "settings" => {
                        scene.settings = self.parse_settings()?;
                        attach(&mut comments, CommentAnchor::Settings, &mut pending);
                    }
                    "style" => {
                        let style = self.parse_style_block()?;
                        scene.styles.push(style);
                        // Style blocks aren't yet a CommentAnchor target.
                        pending.clear();
                    }
                    "node" => {
                        let node = self.parse_node_with_styles(&scene.styles)?;
                        let anchor = CommentAnchor::Node(node.id.clone());
                        scene.nodes.push(node);
                        attach(&mut comments, anchor, &mut pending);
                    }
                    "wire" => {
                        let edge = self.parse_wire()?;
                        let anchor = CommentAnchor::Wire(edge.id.clone());
                        scene.edges.push(edge);
                        attach(&mut comments, anchor, &mut pending);
                    }
                    other => return self.err(format!("unexpected keyword '{other}'")),
                },
                _ => return self.err("expected 'settings', 'style', 'node', 'wire', or '}'"),
            }
        }

        self.skip_trivia();
        if self.peek().is_some() {
            return self.err("unexpected content after canvas block");
        }
        Ok(ParsedDocument { scene, comments })
    }

    // ── settings ─────────────────────────────────────────────────────────

    fn parse_settings(&mut self) -> Result<CanvasSettings, ParseError> {
        self.expect_kw("settings")?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_trivia();

        let mut s = CanvasSettings::default();
        while !matches!(self.peek(), Some(Tok::RBrace)) {
            let key = self.ident()?;
            match key.as_str() {
                "grid" => s.grid_spacing = self.number()?,
                "grid_style" => s.grid_style = self.parse_grid_style()?,
                "dot_size" => s.dot_size = self.number()?,
                "units" => s.grid_units = self.parse_units()?,
                "snap" => s.snap_to_grid = self.parse_onoff()?,
                "show_grid" => s.show_grid = self.parse_onoff()?,
                "routing" => s.default_routing = self.parse_routing()?,
                "paper" => s.paper_size = Some(self.string()?),
                "orientation" => s.paper_orientation = Some(self.string()?),
                "background" => s.background = self.parse_background()?,
                other => return self.err(format!("unknown setting '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(s)
    }

    // ── node ─────────────────────────────────────────────────────────────

    fn parse_node_with_styles(&mut self, styles: &[Style]) -> Result<Node, ParseError> {
        self.expect_kw("node")?;
        let id = self.ident()?;
        self.expect(&Tok::Colon, "':'")?;
        let kind = self.parse_node_kind()?;

        // Optional second colon → style class reference. The style
        // pre-seeds the node's overlay + ports; inline fields then
        // override.
        let mut style_ref: Option<String> = None;
        if matches!(self.peek(), Some(Tok::Colon)) {
            self.pos += 1;
            let name = self.ident()?;
            style_ref = Some(name);
        }

        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_trivia();

        // Seed the node from the named style, if any.
        let mut overlay = Overlay::default();
        let mut ports: Vec<Port> = Vec::new();
        if let Some(name) = &style_ref {
            let Some(style) = styles.iter().find(|s| &s.name == name) else {
                return self.err(format!("unknown style '{name}'"));
            };
            if let Some(b) = &style.border {
                overlay.border = b.clone();
            }
            if let Some(f) = &style.fill {
                overlay.fill = f.clone();
            }
            if let Some(t) = &style.text {
                overlay.text = Some(t.clone());
            }
            ports = style.ports.clone();
        }

        let mut node = Node {
            id: NodeId(id),
            kind,
            transform: Transform::default(),
            overlay,
            ports,
            style_ref,
        };

        while !matches!(self.peek(), Some(Tok::RBrace)) {
            let key = self.ident()?;
            match key.as_str() {
                "at" => node.transform.position = (self.number()?, self.number()?),
                "size" => node.transform.size = (self.number()?, self.number()?),
                "rotation" => node.transform.rotation = self.number()?,
                "border" => node.overlay.border = self.parse_border()?,
                "fill" => node.overlay.fill = self.parse_fill()?,
                "text" => node.overlay.text = Some(self.parse_text()?),
                "port" => {
                    // Inline ports override style ports with the same id.
                    let p = self.parse_port()?;
                    if let Some(slot) = node.ports.iter_mut().find(|sp| sp.id == p.id) {
                        *slot = p;
                    } else {
                        node.ports.push(p);
                    }
                }
                other => return self.err(format!("unknown node property '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(node)
    }

    /// `style NAME { ... }` block — every inner key is optional, mirrors
    /// the node-field keys but without `at`/`size`/`rotation`.
    fn parse_style_block(&mut self) -> Result<Style, ParseError> {
        self.expect_kw("style")?;
        let name = self.ident()?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_trivia();

        let mut style = Style {
            name,
            border: None,
            fill: None,
            text: None,
            ports: Vec::new(),
        };
        while !matches!(self.peek(), Some(Tok::RBrace)) {
            let key = self.ident()?;
            match key.as_str() {
                "border" => style.border = Some(self.parse_border()?),
                "fill" => style.fill = Some(self.parse_fill()?),
                "text" => style.text = Some(self.parse_text()?),
                "port" => style.ports.push(self.parse_port()?),
                other => return self.err(format!("unknown style property '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(style)
    }

    fn parse_node_kind(&mut self) -> Result<NodeKind, ParseError> {
        let k = self.ident()?;
        match k.as_str() {
            "rect" => Ok(NodeKind::Rect),
            "circle" => Ok(NodeKind::Circle),
            "ellipse" => Ok(NodeKind::Ellipse),
            "parallelogram" => Ok(NodeKind::Parallelogram),
            other => self.err(format!(
                "unknown node kind '{other}' (expected rect/circle/ellipse/parallelogram)"
            )),
        }
    }

    fn parse_border(&mut self) -> Result<Border, ParseError> {
        let style = self.parse_line_style()?;
        let width = self.number()?;
        let color = self.string()?;
        Ok(Border { color, width, style })
    }

    fn parse_fill(&mut self) -> Result<Fill, ParseError> {
        let color = self.string()?;
        let alpha = self.number()?;
        Ok(Fill { color, alpha })
    }

    fn parse_text(&mut self) -> Result<TextLabel, ParseError> {
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_trivia();

        let mut t = TextLabel {
            value: String::new(),
            anchor: TextAnchor::Center,
            font_family: String::new(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: "#000000".to_string(),
        };
        while !matches!(self.peek(), Some(Tok::RBrace)) {
            let key = self.ident()?;
            match key.as_str() {
                "value" => t.value = self.string()?,
                "anchor" => t.anchor = self.parse_text_anchor()?,
                "font" => {
                    t.font_family = self.string()?;
                    t.font_size = self.number()?;
                }
                "bold" => t.bold = self.parse_onoff()?,
                "italic" => t.italic = self.parse_onoff()?,
                "color" => t.color = self.string()?,
                other => return self.err(format!("unknown text property '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(t)
    }

    fn parse_port(&mut self) -> Result<Port, ParseError> {
        let kind = self.parse_port_kind()?;
        let name = self.ident()?;
        let anchor = self.parse_port_anchor()?;
        let data_type = if matches!(self.peek(), Some(Tok::Ident(k)) if k == "type") {
            self.pos += 1;
            Some(self.string()?)
        } else {
            None
        };
        Ok(Port { id: PortId(name.clone()), name, kind, anchor, data_type })
    }

    fn parse_port_kind(&mut self) -> Result<PortKind, ParseError> {
        let k = self.ident()?;
        match k.as_str() {
            "in" => Ok(PortKind::In),
            "out" => Ok(PortKind::Out),
            "bidir" => Ok(PortKind::Bidir),
            "untyped" => Ok(PortKind::Untyped),
            other => self.err(format!("unknown port kind '{other}'")),
        }
    }

    fn parse_port_anchor(&mut self) -> Result<PortAnchor, ParseError> {
        let side = self.ident()?;
        match side.as_str() {
            "north" => Ok(PortAnchor::North(self.number()?)),
            "south" => Ok(PortAnchor::South(self.number()?)),
            "east" => Ok(PortAnchor::East(self.number()?)),
            "west" => Ok(PortAnchor::West(self.number()?)),
            "free" => Ok(PortAnchor::Free(self.number()?, self.number()?)),
            other => self.err(format!("unknown port anchor '{other}'")),
        }
    }

    // ── wire ─────────────────────────────────────────────────────────────

    fn parse_wire(&mut self) -> Result<Edge, ParseError> {
        self.expect_kw("wire")?;
        let id = self.ident()?;
        let from = self.parse_endpoint()?;
        self.expect(&Tok::Arrow, "'->'")?;
        let to = self.parse_endpoint()?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_trivia();

        let mut edge = Edge {
            id: EdgeId(id),
            from,
            to,
            routing: Routing::default(),
            overlay: EdgeOverlay::default(),
        };

        while !matches!(self.peek(), Some(Tok::RBrace)) {
            let key = self.ident()?;
            match key.as_str() {
                "routing" => edge.routing = self.parse_routing()?,
                "stroke" => {
                    edge.overlay.color = self.string()?;
                    edge.overlay.width = self.number()?;
                    edge.overlay.line_style = self.parse_line_style()?;
                }
                "arrow" => {
                    edge.overlay.arrow_head = self.parse_arrow_head()?;
                    edge.overlay.arrow_tail = self.parse_arrow_head()?;
                }
                "label" => edge.overlay.label = Some(self.string()?),
                other => return self.err(format!("unknown wire property '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(edge)
    }

    fn parse_endpoint(&mut self) -> Result<EdgeEnd, ParseError> {
        let node = self.ident()?;
        self.expect(&Tok::Dot, "'.'")?;
        let port = self.ident()?;
        Ok(EdgeEnd::Port(NodeId(node), PortId(port)))
    }

    // ── small enums ──────────────────────────────────────────────────────

    fn parse_onoff(&mut self) -> Result<bool, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "on" => Ok(true),
            "off" => Ok(false),
            other => self.err(format!("expected 'on' or 'off', found '{other}'")),
        }
    }

    fn parse_line_style(&mut self) -> Result<LineStyle, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "solid" => Ok(LineStyle::Solid),
            "dashed" => Ok(LineStyle::Dashed),
            "dotted" => Ok(LineStyle::Dotted),
            other => self.err(format!("unknown line style '{other}'")),
        }
    }

    fn parse_routing(&mut self) -> Result<Routing, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "orthogonal" => Ok(Routing::Orthogonal),
            "bezier" => Ok(Routing::Bezier),
            "straight" => Ok(Routing::Straight),
            "manual" => {
                let mut waypoints = Vec::new();
                while matches!(self.peek(), Some(Tok::Num(_))) {
                    let x = self.number()?;
                    let y = self.number()?;
                    waypoints.push((x, y));
                }
                Ok(Routing::Manual { waypoints })
            }
            other => self.err(format!("unknown routing '{other}'")),
        }
    }

    fn parse_background(&mut self) -> Result<CanvasBackground, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "light" => Ok(CanvasBackground::Light),
            "slate" => Ok(CanvasBackground::Slate),
            "charcoal" => Ok(CanvasBackground::Charcoal),
            "dark" => Ok(CanvasBackground::Dark),
            other => self.err(format!("unknown background '{other}'")),
        }
    }

    fn parse_grid_style(&mut self) -> Result<GridStyle, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "lines" => Ok(GridStyle::Lines),
            "dots" => Ok(GridStyle::Dots),
            other => self.err(format!("unknown grid style '{other}'")),
        }
    }

    fn parse_units(&mut self) -> Result<GridUnits, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "pixels" => Ok(GridUnits::Pixels),
            "mils" => Ok(GridUnits::Mils),
            "mm" => Ok(GridUnits::Millimeters),
            "inches" => Ok(GridUnits::Inches),
            other => self.err(format!("unknown units '{other}'")),
        }
    }

    fn parse_text_anchor(&mut self) -> Result<TextAnchor, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "center" => Ok(TextAnchor::Center),
            "top_center" => Ok(TextAnchor::TopCenter),
            "bottom_center" => Ok(TextAnchor::BottomCenter),
            "left" => Ok(TextAnchor::Left),
            "right" => Ok(TextAnchor::Right),
            "top_left" => Ok(TextAnchor::TopLeft),
            "top_right" => Ok(TextAnchor::TopRight),
            "bottom_left" => Ok(TextAnchor::BottomLeft),
            "bottom_right" => Ok(TextAnchor::BottomRight),
            other => self.err(format!("unknown text anchor '{other}'")),
        }
    }

    fn parse_arrow_head(&mut self) -> Result<ArrowHead, ParseError> {
        let v = self.ident()?;
        match v.as_str() {
            "none" => Ok(ArrowHead::None),
            "arrow" => Ok(ArrowHead::Arrow),
            "triangle" => Ok(ArrowHead::Triangle),
            "diamond" => Ok(ArrowHead::Diamond),
            "circle" => Ok(ArrowHead::Circle),
            other => self.err(format!("unknown arrowhead '{other}'")),
        }
    }
}

// =============================================================================
// Pretty-printer
// =============================================================================

struct Printer<'a> {
    out: String,
    /// Comment blocks to re-emit before their anchored items. Empty slice
    /// for the comment-less `pretty`.
    comments: &'a [CommentBlock],
}

impl<'a> Printer<'a> {
    fn new(comments: &'a [CommentBlock]) -> Self {
        Self { out: String::new(), comments }
    }

    fn print(mut self, scene: &Scene) -> String {
        self.emit_comments(&CommentAnchor::Header, 0);
        self.line(0, &format!("canvas {} {{", quote(&scene.name)));
        self.emit_comments(&CommentAnchor::Settings, 1);
        self.print_settings(&scene.settings);

        // Style extraction — group nodes by their (overlay, ports)
        // signature; every group with 2+ members is factored into a
        // named style and referenced from each member. Single
        // occurrences inline as before, so a one-off node doesn't
        // grow a wrapper.
        let style_table = build_style_table(scene);
        for style in &style_table.styles {
            self.out.push('\n');
            self.print_style(style);
        }

        for (idx, node) in scene.nodes.iter().enumerate() {
            self.out.push('\n');
            self.emit_comments(&CommentAnchor::Node(node.id.clone()), 1);
            self.print_node(node, style_table.node_style.get(&idx).cloned());
        }
        for edge in &scene.edges {
            // Dangling (free-ended) wires aren't representable in the
            // DSL yet — they're an in-memory-only construct, so skip
            // them on save. Re-loading drops them silently.
            if edge.from.is_free() || edge.to.is_free() {
                continue;
            }
            self.out.push('\n');
            self.emit_comments(&CommentAnchor::Wire(edge.id.clone()), 1);
            self.print_wire(edge);
        }
        self.line(0, "}");
        self.out
    }

    fn print_style(&mut self, style: &Style) {
        self.line(1, &format!("style {} {{", style.name));
        if let Some(b) = &style.border {
            self.line(
                2,
                &format!("border {} {} {}", line_style_kw(b.style), num(b.width), quote(&b.color)),
            );
        }
        if let Some(f) = &style.fill {
            self.line(2, &format!("fill {} {}", quote(&f.color), num(f.alpha)));
        }
        if let Some(text) = &style.text {
            self.print_text_block(2, text);
        }
        for port in &style.ports {
            self.line(2, &self.port_line(port));
        }
        self.line(1, "}");
    }

    fn print_text_block(&mut self, depth: usize, text: &TextLabel) {
        self.line(depth, "text {");
        self.line(depth + 1, &format!("value {}", quote(&text.value)));
        self.line(depth + 1, &format!("anchor {}", text_anchor_kw(text.anchor)));
        self.line(
            depth + 1,
            &format!("font {} {}", quote(&text.font_family), num(text.font_size)),
        );
        self.line(depth + 1, &format!("bold {}", onoff(text.bold)));
        self.line(depth + 1, &format!("italic {}", onoff(text.italic)));
        self.line(depth + 1, &format!("color {}", quote(&text.color)));
        self.line(depth, "}");
    }

    /// Emit every comment block anchored at `anchor`, as `#`-prefixed lines.
    fn emit_comments(&mut self, anchor: &CommentAnchor, depth: usize) {
        let blocks = self.comments;
        for block in blocks {
            if &block.anchor == anchor {
                for text in &block.lines {
                    self.line(depth, &format!("//{text}"));
                }
            }
        }
    }

    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn print_settings(&mut self, s: &CanvasSettings) {
        self.line(1, "settings {");
        self.line(2, &format!("grid {}", num(s.grid_spacing)));
        self.line(2, &format!("grid_style {}", grid_style_kw(s.grid_style)));
        self.line(2, &format!("dot_size {}", num(s.dot_size)));
        self.line(2, &format!("units {}", units_kw(s.grid_units)));
        self.line(2, &format!("snap {}", onoff(s.snap_to_grid)));
        self.line(2, &format!("show_grid {}", onoff(s.show_grid)));
        self.line(2, &format!("routing {}", routing_text(&s.default_routing)));
        self.line(2, &format!("background {}", background_kw(s.background)));
        if let Some(p) = &s.paper_size {
            self.line(2, &format!("paper {}", quote(p)));
        }
        if let Some(o) = &s.paper_orientation {
            self.line(2, &format!("orientation {}", quote(o)));
        }
        self.line(1, "}");
    }

    fn print_node(&mut self, node: &Node, applied_style: Option<Style>) {
        let header = match &applied_style {
            Some(s) => format!("node {} : {} : {} {{", node.id.0, node_kind_kw(&node.kind), s.name),
            None => format!("node {} : {} {{", node.id.0, node_kind_kw(&node.kind)),
        };
        self.line(1, &header);
        let t = &node.transform;
        self.line(2, &format!("at {} {}", num(t.position.0), num(t.position.1)));
        self.line(2, &format!("size {} {}", num(t.size.0), num(t.size.1)));
        self.line(2, &format!("rotation {}", num(t.rotation)));

        // Skip overlay + ports fields when they match the applied style.
        // Anything that differs gets emitted as an inline override.
        let style_border = applied_style.as_ref().and_then(|s| s.border.clone());
        let style_fill = applied_style.as_ref().and_then(|s| s.fill.clone());
        let style_text = applied_style.as_ref().and_then(|s| s.text.clone());
        let style_ports: Vec<Port> = applied_style
            .as_ref()
            .map(|s| s.ports.clone())
            .unwrap_or_default();

        let b = &node.overlay.border;
        if style_border.as_ref() != Some(b) {
            self.line(
                2,
                &format!("border {} {} {}", line_style_kw(b.style), num(b.width), quote(&b.color)),
            );
        }
        let f = &node.overlay.fill;
        if style_fill.as_ref() != Some(f) {
            self.line(2, &format!("fill {} {}", quote(&f.color), num(f.alpha)));
        }

        if let Some(text) = &node.overlay.text {
            if style_text.as_ref() != Some(text) {
                self.print_text_block(2, text);
            }
        }

        for port in &node.ports {
            // Only emit ports that differ from (or aren't in) the style.
            let in_style = style_ports.iter().any(|sp| sp == port);
            if !in_style {
                self.line(2, &self.port_line(port));
            }
        }
        self.line(1, "}");
    }

    fn port_line(&self, port: &Port) -> String {
        let anchor = match port.anchor {
            PortAnchor::North(t) => format!("north {}", num(t)),
            PortAnchor::South(t) => format!("south {}", num(t)),
            PortAnchor::East(t) => format!("east {}", num(t)),
            PortAnchor::West(t) => format!("west {}", num(t)),
            PortAnchor::Free(x, y) => format!("free {} {}", num(x), num(y)),
        };
        let mut s = format!("port {} {} {}", port_kind_kw(port.kind), port.name, anchor);
        if let Some(dt) = &port.data_type {
            s.push_str(&format!(" type {}", quote(dt)));
        }
        s
    }

    fn print_wire(&mut self, edge: &Edge) {
        // The caller filters out free-ended edges; this guard is just
        // defense-in-depth so the field accesses below stay valid.
        let (EdgeEnd::Port(fn_id, fp_id), EdgeEnd::Port(tn_id, tp_id)) =
            (&edge.from, &edge.to)
        else {
            return;
        };
        self.line(
            1,
            &format!(
                "wire {} {}.{} -> {}.{} {{",
                edge.id.0, fn_id.0, fp_id.0, tn_id.0, tp_id.0
            ),
        );
        self.line(2, &format!("routing {}", routing_text(&edge.routing)));
        let o = &edge.overlay;
        self.line(2, &format!("stroke {} {} {}", quote(&o.color), num(o.width), line_style_kw(o.line_style)));
        self.line(2, &format!("arrow {} {}", arrow_head_kw(o.arrow_head), arrow_head_kw(o.arrow_tail)));
        if let Some(label) = &o.label {
            self.line(2, &format!("label {}", quote(label)));
        }
        self.line(1, "}");
    }
}

// =============================================================================
// Formatting helpers
// =============================================================================

/// Format a float without a trailing `.0` for integral values. Rust's `{}`
/// for `f32` emits the shortest string that round-trips to the same `f32`.
/// Result of style harvesting — the list of styles to emit, and a
/// per-node index → resolved style mapping for inline omission.
struct StyleTable {
    styles: Vec<Style>,
    node_style: std::collections::HashMap<usize, Style>,
}

/// Walk the scene's nodes and harvest reusable styles. Two policies
/// govern the result:
///
/// 1. If a node already carries a `style_ref` and the scene's
///    `styles` list contains that name with a matching signature,
///    use it as-is.
/// 2. Otherwise group nodes by their `(overlay, ports)` signature;
///    any group with 2+ members becomes an auto-style named
///    `s0`, `s1`, ... Single occurrences emit inline (no style).
fn build_style_table(scene: &Scene) -> StyleTable {
    // Use Vec linear scan rather than HashMap because Border/Fill
    // contain f32 (no Hash). The N here is the node count, kept
    // tiny in practice.
    let mut groups: Vec<(Overlay, Vec<Port>, Vec<usize>)> = Vec::new();
    for (idx, node) in scene.nodes.iter().enumerate() {
        if let Some(slot) = groups
            .iter_mut()
            .find(|(o, p, _)| o == &node.overlay && p == &node.ports)
        {
            slot.2.push(idx);
        } else {
            groups.push((node.overlay.clone(), node.ports.clone(), vec![idx]));
        }
    }

    let mut styles: Vec<Style> = Vec::new();
    let mut node_style: std::collections::HashMap<usize, Style> = std::collections::HashMap::new();
    let mut auto_counter = 0;

    for (overlay, ports, indices) in groups {
        if indices.len() < 2 {
            continue;
        }
        // Pick a name: first node with a style_ref wins; else mint sNN.
        let name = indices
            .iter()
            .find_map(|i| scene.nodes[*i].style_ref.clone())
            .unwrap_or_else(|| {
                let n = format!("s{auto_counter}");
                auto_counter += 1;
                n
            });
        let style = Style {
            name: name.clone(),
            border: Some(overlay.border.clone()),
            fill: Some(overlay.fill.clone()),
            text: overlay.text.clone(),
            ports: ports.clone(),
        };
        styles.push(style.clone());
        for i in indices {
            node_style.insert(i, style.clone());
        }
    }

    // Carry through any scene-level styles the parser stored that
    // weren't referenced by a node — preserves user-authored styles
    // across a round-trip even when nothing currently uses them.
    for s in &scene.styles {
        if !styles.iter().any(|x| x.name == s.name) {
            styles.push(s.clone());
        }
    }

    // Honour every node's *explicit* `style_ref`, even for
    // singletons that auto-extraction skipped. Two things to
    // settle here:
    //
    //   1. The named style must exist in the output `styles` list
    //      so the parser can find it on reload. We've already
    //      copied `scene.styles` over above, so any user-set ref
    //      with a registered style is covered.
    //   2. The node needs a matching entry in `node_style` so
    //      `print_node` knows to emit the `: stylename` annotation
    //      and omit fields equal to the style.
    //
    // Without this pass, a programmatic placement (symbol-instance
    // with unique label text → unique overlay → singleton group)
    // loses its style ref on save, and recovery sees a stylesless
    // node and can't tell it was an instance.
    for (idx, node) in scene.nodes.iter().enumerate() {
        if node_style.contains_key(&idx) {
            continue;
        }
        if let Some(name) = &node.style_ref
            && let Some(style) = styles.iter().find(|s| &s.name == name)
        {
            node_style.insert(idx, style.clone());
        }
    }

    StyleTable { styles, node_style }
}

fn num(v: f32) -> String {
    if v.is_finite() && v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            other => o.push(other),
        }
    }
    o.push('"');
    o
}

fn onoff(b: bool) -> &'static str {
    if b { "on" } else { "off" }
}

fn node_kind_kw(k: &NodeKind) -> &'static str {
    match k {
        NodeKind::Rect => "rect",
        NodeKind::Circle => "circle",
        NodeKind::Ellipse => "ellipse",
        NodeKind::Parallelogram => "parallelogram",
        // v1: Path/Group are not yet expressible — printed lossily as rect.
        NodeKind::Path(_) | NodeKind::Group(_) => "rect",
    }
}

fn line_style_kw(s: LineStyle) -> &'static str {
    match s {
        LineStyle::Solid => "solid",
        LineStyle::Dashed => "dashed",
        LineStyle::Dotted => "dotted",
    }
}

fn routing_text(r: &Routing) -> String {
    match r {
        Routing::Orthogonal => "orthogonal".to_string(),
        Routing::Bezier => "bezier".to_string(),
        Routing::Straight => "straight".to_string(),
        Routing::Manual { waypoints } => {
            let mut s = String::from("manual");
            for (x, y) in waypoints {
                s.push_str(&format!(" {} {}", num(*x), num(*y)));
            }
            s
        }
    }
}

fn background_kw(b: CanvasBackground) -> &'static str {
    match b {
        CanvasBackground::Light => "light",
        CanvasBackground::Slate => "slate",
        CanvasBackground::Charcoal => "charcoal",
        CanvasBackground::Dark => "dark",
    }
}

fn grid_style_kw(s: GridStyle) -> &'static str {
    match s {
        GridStyle::Lines => "lines",
        GridStyle::Dots => "dots",
    }
}

fn units_kw(u: GridUnits) -> &'static str {
    match u {
        GridUnits::Pixels => "pixels",
        GridUnits::Mils => "mils",
        GridUnits::Millimeters => "mm",
        GridUnits::Inches => "inches",
    }
}

fn port_kind_kw(k: PortKind) -> &'static str {
    match k {
        PortKind::In => "in",
        PortKind::Out => "out",
        PortKind::Bidir => "bidir",
        PortKind::Untyped => "untyped",
    }
}

fn text_anchor_kw(a: TextAnchor) -> &'static str {
    match a {
        TextAnchor::Center => "center",
        TextAnchor::TopCenter => "top_center",
        TextAnchor::BottomCenter => "bottom_center",
        TextAnchor::Left => "left",
        TextAnchor::Right => "right",
        TextAnchor::TopLeft => "top_left",
        TextAnchor::TopRight => "top_right",
        TextAnchor::BottomLeft => "bottom_left",
        TextAnchor::BottomRight => "bottom_right",
    }
}

fn arrow_head_kw(a: ArrowHead) -> &'static str {
    match a {
        ArrowHead::None => "none",
        ArrowHead::Arrow => "arrow",
        ArrowHead::Triangle => "triangle",
        ArrowHead::Diamond => "diamond",
        ArrowHead::Circle => "circle",
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scene() -> Scene {
        Scene {
            name: "Test Sheet".to_string(),
            settings: CanvasSettings {
                grid_spacing: 25.0,
                snap_to_grid: false,
                show_grid: true,
                grid_style: GridStyle::Dots,
                dot_size: 3.0,
                grid_units: GridUnits::Millimeters,
                paper_size: Some("A4".to_string()),
                paper_orientation: Some("portrait".to_string()),
                default_routing: Routing::Bezier,
                background: CanvasBackground::Slate,
                title_block: None,
                port_marker_size: 4.0,
                port_marker_style: crate::model::PortMarkerStyle::Disc,
            },
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    kind: NodeKind::Rect,
                    transform: Transform {
                        position: (10.0, 20.0),
                        size: (100.0, 60.0),
                        rotation: 0.0,
                    },
                    overlay: Overlay {
                        border: Border {
                            color: "#111111".into(),
                            width: 2.0,
                            style: LineStyle::Dashed,
                        },
                        fill: Fill { color: "#EEEEEE".into(), alpha: 0.8 },
                        text: Some(TextLabel {
                            value: "Hello\nWorld".into(),
                            anchor: TextAnchor::Center,
                            font_family: "Inter".into(),
                            font_size: 14.0,
                            bold: true,
                            italic: false,
                            color: "#000000".into(),
                        }),
                    },
                    ports: vec![
                        Port {
                            id: PortId("p1".into()),
                            name: "p1".into(),
                            kind: PortKind::Out,
                            anchor: PortAnchor::East(0.5),
                            data_type: Some("sig".into()),
                        },
                        Port {
                            id: PortId("p2".into()),
                            name: "p2".into(),
                            kind: PortKind::In,
                            anchor: PortAnchor::West(0.25),
                            data_type: None,
                        },
                    ],
                    style_ref: None,
                },
                Node {
                    id: NodeId("b".into()),
                    kind: NodeKind::Circle,
                    transform: Transform {
                        position: (300.0, 40.0),
                        size: (80.0, 80.0),
                        rotation: 0.0,
                    },
                    overlay: Overlay::default(),
                    ports: vec![Port {
                        id: PortId("q1".into()),
                        name: "q1".into(),
                        kind: PortKind::In,
                        anchor: PortAnchor::Free(0.1, 0.2),
                        data_type: None,
                    }],
                    style_ref: None,
                },
            ],
            edges: vec![Edge {
                id: EdgeId("e1".into()),
                from: EdgeEnd::Port(NodeId("a".into()), PortId("p1".into())),
                to: EdgeEnd::Port(NodeId("b".into()), PortId("q1".into())),
                routing: Routing::Orthogonal,
                overlay: EdgeOverlay {
                    color: "#FF0000".into(),
                    width: 1.5,
                    line_style: LineStyle::Solid,
                    arrow_head: ArrowHead::Triangle,
                    arrow_tail: ArrowHead::None,
                    label: Some("net1".into()),
                },
            }],
            groups: vec![],
            styles: vec![],
        }
    }

    #[test]
    fn roundtrip_pretty_then_parse() {
        let scene = sample_scene();
        let text = pretty(&scene);
        let reparsed = parse(&text).expect("parse must succeed");
        assert_eq!(reparsed, scene, "scene must survive pretty -> parse\n--- text ---\n{text}");
    }

    #[test]
    fn pretty_is_idempotent() {
        let scene = sample_scene();
        let once = pretty(&scene);
        let twice = pretty(&parse(&once).unwrap());
        assert_eq!(once, twice);
    }

    #[test]
    fn shared_overlay_and_ports_factor_into_a_named_style() {
        // Three rect nodes with identical overlay + ports — the
        // pretty-printer should emit a single `style s0 { ... }`
        // block and three lean `node X : rect : s0 { ... }` rows.
        let make = |id: &str, x: f32, y: f32| Node {
            id: NodeId(id.into()),
            kind: NodeKind::Rect,
            transform: Transform { position: (x, y), size: (80.0, 50.0), rotation: 0.0 },
            overlay: Overlay {
                border: Border { color: "#1F2937".into(), width: 2.0, style: LineStyle::Solid },
                fill: Fill { color: "#DBEAFE".into(), alpha: 0.9 },
                text: None,
            },
            ports: vec![Port {
                id: PortId("n".into()),
                name: "n".into(),
                kind: PortKind::Untyped,
                anchor: PortAnchor::North(0.5),
                data_type: None,
            }],
            style_ref: None,
        };
        let scene = Scene {
            nodes: vec![make("a", 0.0, 0.0), make("b", 100.0, 0.0), make("c", 200.0, 0.0)],
            ..Default::default()
        };
        let text = pretty(&scene);
        assert!(text.contains("style s0 {"), "expected style block in:\n{text}");
        assert!(text.contains("node a : rect : s0"), "expected style ref:\n{text}");
        // The border field appears exactly once — inside the style
        // block, not in any of the three node bodies.
        let border_count = text.matches("border solid").count();
        assert_eq!(border_count, 1, "border should appear only in the style:\n{text}");
        // Round-trip preserves the scene exactly.
        let reparsed = parse(&text).unwrap();
        assert_eq!(reparsed.nodes.len(), 3);
        for n in &reparsed.nodes {
            assert_eq!(n.overlay.border.color, "#1F2937");
            assert_eq!(n.style_ref.as_deref(), Some("s0"));
        }
    }

    #[test]
    fn inline_field_overrides_the_named_style() {
        // Two nodes share style "default"; the second overrides fill.
        // Parse must preserve the override on `b` while leaving `a`
        // with the style's value.
        let dsl = r##"
canvas "" {
  settings { grid 10 }
  style default {
    border solid 2 "#1F2937"
    fill "#DBEAFE" 0.9
  }
  node a : rect : default {
    at 0 0
    size 80 50
    rotation 0
  }
  node b : rect : default {
    at 100 0
    size 80 50
    rotation 0
    fill "#FF0000" 1.0
  }
}
"##;
        let scene = parse(dsl).expect("parse");
        assert_eq!(scene.nodes[0].overlay.fill.color, "#DBEAFE");
        assert_eq!(scene.nodes[1].overlay.fill.color, "#FF0000");
        assert_eq!(scene.nodes[0].overlay.border.color, "#1F2937");
        assert_eq!(scene.nodes[1].overlay.border.color, "#1F2937");
    }

    #[test]
    fn parse_minimal_scene() {
        let text = "canvas \"Empty\" {\n  settings {\n    grid 10\n  }\n}\n";
        let scene = parse(text).expect("parse");
        assert_eq!(scene.name, "Empty");
        assert_eq!(scene.settings.grid_spacing, 10.0);
        assert!(scene.nodes.is_empty());
    }

    #[test]
    fn comments_are_skipped() {
        let text = "# a comment\ncanvas \"C\" {\n  # another\n  settings { grid 5 }\n}\n";
        let scene = parse(text).expect("parse");
        assert_eq!(scene.settings.grid_spacing, 5.0);
    }

    #[test]
    fn rust_style_comments_lex_alongside_hash() {
        // All four comment forms must lex equivalently.
        let text = "\
// outer
//! inner
/// doc
# hash
canvas \"C\" {
  // before settings
  settings { grid 5 }
}
";
        let scene = parse(text).expect("parse");
        assert_eq!(scene.settings.grid_spacing, 5.0);
    }

    #[test]
    fn pretty_emits_double_slash_for_comments() {
        // A document loaded with `#` comments should re-emit using
        // the new `//` canonical form on save.
        let src = "# hdr\ncanvas \"C\" {\n  settings { grid 1 }\n}\n";
        let doc = parse_document(src).expect("parse");
        let out = pretty_document(&doc);
        assert!(out.contains("// hdr"), "expected // form in:\n{out}");
        assert!(!out.contains("# hdr"));
    }

    #[test]
    fn error_reports_line_number() {
        let text = "canvas \"C\" {\n  settings {\n    grid bogus\n  }\n}\n";
        let err = parse(text).expect_err("should fail");
        assert_eq!(err.line, 3);
    }

    #[test]
    fn unknown_keyword_is_rejected() {
        let text = "canvas \"C\" {\n  banana foo\n}\n";
        assert!(parse(text).is_err());
    }

    #[test]
    fn document_round_trip_preserves_comments() {
        let src = "\
# file header
# second header line
canvas \"C\" {
  # about the settings
  settings {
    grid 10
  }

  # the alpha node
  node a : rect {
    at 0 0
    size 10 10
    rotation 0
    border solid 1 \"#000000\"
    fill \"#FFFFFF\" 1
  }
}
";
        let doc = parse_document(src).expect("parse");
        let printed = pretty_document(&doc);
        let reparsed = parse_document(&printed).expect("reparse");
        assert_eq!(reparsed, doc, "document must survive pretty_document -> parse_document");
        assert!(printed.contains("// file header"));
        assert!(printed.contains("// about the settings"));
        assert!(printed.contains("// the alpha node"));
    }

    #[test]
    fn comments_anchor_to_the_right_items() {
        let src = "# hdr\ncanvas \"C\" {\n  settings { grid 1 }\n  # for a\n  node a : rect { at 0 0\n size 1 1\n rotation 0\n border solid 1 \"#000000\"\n fill \"#FFFFFF\" 1 }\n}\n";
        let doc = parse_document(src).expect("parse");
        let header = doc.comments.iter().find(|b| b.anchor == CommentAnchor::Header);
        assert_eq!(header.map(|b| b.lines.as_slice()), Some(&[" hdr".to_string()][..]));
        let node_a = doc
            .comments
            .iter()
            .find(|b| b.anchor == CommentAnchor::Node(NodeId("a".into())));
        assert_eq!(node_a.map(|b| b.lines.as_slice()), Some(&[" for a".to_string()][..]));
    }

    #[test]
    fn plain_pretty_drops_comments() {
        let doc = parse_document("# gone\ncanvas \"C\" {\n  settings { grid 1 }\n}\n").unwrap();
        assert!(!pretty(&doc.scene).contains('#'));
    }
}
