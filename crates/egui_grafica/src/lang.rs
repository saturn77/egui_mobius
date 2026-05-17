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
//! - `Routing::Manual` prints as `orthogonal`.
//! - `Scene::groups` is not yet expressed.
//! - `#` line comments are accepted by the lexer but not preserved through a
//!   round-trip. Comment + formatting preservation is deferred.

use crate::model::{
    ArrowHead, Border, CanvasSettings, Edge, EdgeId, EdgeOverlay, Fill, GridStyle, GridUnits,
    LineStyle, Node, NodeId, NodeKind, Overlay, Port, PortAnchor, PortId, PortKind, Routing,
    Scene, TextAnchor, TextLabel, Transform,
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
// Public API
// =============================================================================

/// Parse `.canvas` source into a [`Scene`].
pub fn parse(source: &str) -> Result<Scene, ParseError> {
    let tokens = lex(source)?;
    Parser::new(tokens).parse_scene()
}

/// Emit canonical `.canvas` text for a [`Scene`]. Round-trip stable:
/// `parse(&pretty(&s))` reconstructs an equal `Scene`.
pub fn pretty(scene: &Scene) -> String {
    Printer::default().print(scene)
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
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
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

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Tok::Newline)) {
            self.pos += 1;
        }
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
            Some(Tok::Newline) => {
                self.skip_newlines();
                Ok(())
            }
            Some(Tok::RBrace) | None => Ok(()),
            _ => self.err("unexpected token at end of statement"),
        }
    }

    // ── Scene ────────────────────────────────────────────────────────────

    fn parse_scene(&mut self) -> Result<Scene, ParseError> {
        self.skip_newlines();
        self.expect_kw("canvas")?;
        let name = self.string()?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_newlines();

        let mut scene = Scene { name, ..Scene::default() };

        loop {
            match self.peek() {
                Some(Tok::RBrace) => {
                    self.pos += 1;
                    break;
                }
                Some(Tok::Ident(kw)) => match kw.as_str() {
                    "settings" => scene.settings = self.parse_settings()?,
                    "node" => scene.nodes.push(self.parse_node()?),
                    "wire" => scene.edges.push(self.parse_wire()?),
                    other => return self.err(format!("unexpected keyword '{other}'")),
                },
                _ => return self.err("expected 'settings', 'node', 'wire', or '}'"),
            }
            self.skip_newlines();
        }

        self.skip_newlines();
        if self.peek().is_some() {
            return self.err("unexpected content after canvas block");
        }
        Ok(scene)
    }

    // ── settings ─────────────────────────────────────────────────────────

    fn parse_settings(&mut self) -> Result<CanvasSettings, ParseError> {
        self.expect_kw("settings")?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_newlines();

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
                other => return self.err(format!("unknown setting '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(s)
    }

    // ── node ─────────────────────────────────────────────────────────────

    fn parse_node(&mut self) -> Result<Node, ParseError> {
        self.expect_kw("node")?;
        let id = self.ident()?;
        self.expect(&Tok::Colon, "':'")?;
        let kind = self.parse_node_kind()?;
        self.expect(&Tok::LBrace, "'{'")?;
        self.skip_newlines();

        let mut node = Node {
            id: NodeId(id),
            kind,
            transform: Transform::default(),
            overlay: Overlay::default(),
            ports: Vec::new(),
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
                "port" => node.ports.push(self.parse_port()?),
                other => return self.err(format!("unknown node property '{other}'")),
            }
            self.end_statement()?;
        }
        self.pos += 1; // consume RBrace
        Ok(node)
    }

    fn parse_node_kind(&mut self) -> Result<NodeKind, ParseError> {
        let k = self.ident()?;
        match k.as_str() {
            "rect" => Ok(NodeKind::Rect),
            "circle" => Ok(NodeKind::Circle),
            "ellipse" => Ok(NodeKind::Ellipse),
            other => self.err(format!("unknown node kind '{other}' (expected rect/circle/ellipse)")),
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
        self.skip_newlines();

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
        self.skip_newlines();

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

    fn parse_endpoint(&mut self) -> Result<(NodeId, PortId), ParseError> {
        let node = self.ident()?;
        self.expect(&Tok::Dot, "'.'")?;
        let port = self.ident()?;
        Ok((NodeId(node), PortId(port)))
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
            other => self.err(format!("unknown routing '{other}'")),
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

#[derive(Default)]
struct Printer {
    out: String,
}

impl Printer {
    fn print(mut self, scene: &Scene) -> String {
        self.line(0, &format!("canvas {} {{", quote(&scene.name)));
        self.print_settings(&scene.settings);
        for node in &scene.nodes {
            self.out.push('\n');
            self.print_node(node);
        }
        for edge in &scene.edges {
            self.out.push('\n');
            self.print_wire(edge);
        }
        self.line(0, "}");
        self.out
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
        self.line(2, &format!("routing {}", routing_kw(&s.default_routing)));
        if let Some(p) = &s.paper_size {
            self.line(2, &format!("paper {}", quote(p)));
        }
        if let Some(o) = &s.paper_orientation {
            self.line(2, &format!("orientation {}", quote(o)));
        }
        self.line(1, "}");
    }

    fn print_node(&mut self, node: &Node) {
        self.line(1, &format!("node {} : {} {{", node.id.0, node_kind_kw(&node.kind)));
        let t = &node.transform;
        self.line(2, &format!("at {} {}", num(t.position.0), num(t.position.1)));
        self.line(2, &format!("size {} {}", num(t.size.0), num(t.size.1)));
        self.line(2, &format!("rotation {}", num(t.rotation)));

        let b = &node.overlay.border;
        self.line(2, &format!("border {} {} {}", line_style_kw(b.style), num(b.width), quote(&b.color)));
        let f = &node.overlay.fill;
        self.line(2, &format!("fill {} {}", quote(&f.color), num(f.alpha)));

        if let Some(text) = &node.overlay.text {
            self.line(2, "text {");
            self.line(3, &format!("value {}", quote(&text.value)));
            self.line(3, &format!("anchor {}", text_anchor_kw(text.anchor)));
            self.line(3, &format!("font {} {}", quote(&text.font_family), num(text.font_size)));
            self.line(3, &format!("bold {}", onoff(text.bold)));
            self.line(3, &format!("italic {}", onoff(text.italic)));
            self.line(3, &format!("color {}", quote(&text.color)));
            self.line(2, "}");
        }

        for port in &node.ports {
            self.line(2, &self.port_line(port));
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
        self.line(
            1,
            &format!(
                "wire {} {}.{} -> {}.{} {{",
                edge.id.0, edge.from.0.0, edge.from.1.0, edge.to.0.0, edge.to.1.0
            ),
        );
        self.line(2, &format!("routing {}", routing_kw(&edge.routing)));
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

fn routing_kw(r: &Routing) -> &'static str {
    match r {
        Routing::Orthogonal => "orthogonal",
        Routing::Bezier => "bezier",
        Routing::Straight => "straight",
        // v1: Manual is not yet expressible — printed as orthogonal.
        Routing::Manual(_) => "orthogonal",
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
                },
            ],
            edges: vec![Edge {
                id: EdgeId("e1".into()),
                from: (NodeId("a".into()), PortId("p1".into())),
                to: (NodeId("b".into()), PortId("q1".into())),
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
}
