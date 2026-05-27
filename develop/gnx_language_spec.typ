// GNX Language Specification
// Document version (SEMVER)
#let doc-version = "v0.1.0"

#let inset-size = 11pt

// Git version — passed in via:
//   typst compile --input githash="..." --input buildtime="..." file.typ
#let git-hash = sys.inputs.at("githash", default: "unknown")
#let build-time = sys.inputs.at("buildtime", default: datetime.today().display())

// Clickable TOC entries.
#show outline.entry: it => {
  link(it.element.location(), it)
}

#set page(
  paper: "us-letter",
  margin: 1in,
  header: [
    #grid(
      columns: (1fr, 1fr),
      align: (left, right),
      [_GNX_ Language Specification],
      [_Version_ - #doc-version]
    )
    #line(length: 100%, stroke: 0.5pt)
  ],
  footer: context [
    #grid(
      columns: (1fr, 2fr, 1fr),
      align: (left, center, right),
      [],
      [],
      [Page #counter(page).display()]
    )
  ]
)

#set text(
  font: "Libertinus Serif",
  size: 13pt
)

#set par(justify: true)
#set heading(numbering: "1.")

// Code block styling.
#show raw.where(block: true): it => {
  set text(font: "DejaVu Sans Mono", size: 9pt)
  block(
    fill: luma(245),
    inset: inset-size,
    radius: 4pt,
    width: 100%,
    it
  )
}

#show raw.where(block: false): it => {
  set text(font: "DejaVu Sans Mono", size: 10pt)
  box(
    fill: luma(240),
    inset: (x: 3pt, y: 0pt),
    outset: (y: 3pt),
    radius: 2pt,
    it
  )
}

// Invariant / contract callout.
#let invariant(body) = {
  block(
    fill: rgb("#fff3d6"),
    inset: inset-size,
    radius: 4pt,
    width: 100%,
    [#text(weight: "bold", size: 9pt, fill: rgb("#8a6d00"))[Invariant] #body]
  )
}

// Implementation-detail callout (host-specific behaviour).
#let host-note(body) = {
  block(
    fill: rgb("#e7f3ff"),
    inset: inset-size,
    radius: 4pt,
    width: 100%,
    [#text(weight: "bold", size: 9pt, fill: rgb("#0066cc"))[Host Note] #body]
  )
}

// ============================================================
// Title Page
// ============================================================

#align(center)[
  #text(size: 18pt, weight: "bold")[
    _GNX_ Language Specification
  ]

  #v(0.5em)

  #text(size: 14pt, fill: gray)[
    Graphica Node eXchange — A Declarative Format for Engineering Diagrams
  ]

  #v(1em)

  #text(size: 12pt)[
    *Prepared by* \ James Bonanno \ \<james\@atlantixeng.com\> \ \
    *Git Version:* #raw(git-hash) \
    *Last Updated:* #build-time
  ]

  #v(1.5in)

  #align(left)[
    #text(size: 14pt, weight: "bold")[Abstract]

    #v(0.5em)

    #text(size: 12pt)[
      #par(justify: true)[
        GNX (Graphica Node eXchange) is a *declarative, text-first description language* for two-dimensional engineering diagrams. A `.gnx` document describes everything a viewer needs to reproduce a drawing: the drafting sheet, named style classes, primitive shapes ("nodes"), and the connections between them ("wires"). It is the on-disk format for the `egui_grafica` canvas and the editor surface for `grafica-ide`.
      ]
      #par(justify: true)[
        This document specifies GNX, including: lexical structure (line-oriented tokens, four comment styles, double-quoted strings, signed decimal numbers); top-level grammar (the `canvas { ... }` root block); style classes (re-usable overlay and port templates referenced by nodes through a `: stylename` suffix, with inline overrides); the coordinate model and page-board declaration; and the round-trip invariant — `parse(pretty(scene)) ≡ scene` for every valid scene.
      ]
    ]
  ]
]

#pagebreak()

// Table of contents.
#outline(
  title: "Table of Contents",
  indent: auto
)

#pagebreak()

// Document History (unnumbered, not in TOC).
#heading(level: 1, numbering: none, outlined: false)[Document History]

#table(
  columns: (auto, 1fr, auto, auto),
  align: (center, left, center, center),
  table.header(
    [*Version*], [*Description*], [*Date*], [*Author*]
  ),
  [0.1.0], [Initial spec — canvas, settings, style, node, wire, four comment forms, page board declaration, round-trip invariant.], [27 May 2026], [JB]
)

#pagebreak()

// Scope (unnumbered, not in TOC).
#heading(level: 1, numbering: none, outlined: false)[Scope]

This document defines the GNX language for *v0.1*. It covers:

- File format (`.gnx`) and its lexical structure.
- Top-level grammar: the `canvas` root, the `settings`, `style`, `node`, and `wire` block syntaxes.
- Style auto-extraction and inline-override semantics.
- The coordinate model and the engineering-drawing page board declaration.
- The round-trip invariant any conformant implementation must preserve.

It does *not* cover:

- The visual rendering pipeline of `egui_grafica` (CPU painter and wgpu retained-mode).
- The IDE shell (`grafica-ide`) — Inspector behaviour, dock layout, editor sync.
- Behavioural or computational semantics of node-graph evaluation. GNX describes geometry and styling, not "what the nodes do."

#heading(level: 1, numbering: none, outlined: false)[Purpose]

The purpose of this document is to:

- Define the syntax and semantics of the GNX language.
- Establish the contract a conformant parser and pretty-printer must honour.
- Pin down the lossy and lossless cases of the round-trip pipeline.
- Provide a worked example a maintainer can read top-to-bottom and reproduce.
- Guide downstream implementations (alternative editors, exporters, tooling).

#pagebreak()

// ============================================================
// SECTION 1: MOTIVATION
// ============================================================

= Motivation <motivation>

The canvas world has two dominant authoring modes:

+ *Visual editors with opaque formats* — Visio, draw.io, Lucid. The drawing lives inside the tool; the file is a binary or XML payload that is hostile to diff, code review, and pipeline generation.

+ *Pure-text languages with no editor surface* — TikZ, mermaid, PlantUML. Excellent for diff, but hostile to the geometric authoring real diagrams need.

GNX takes a third path: a *dual-surface format* that is simultaneously a hand-readable text file and the canonical save form of a direct-manipulation canvas. The editor and the file are not coupled through serialisation tricks — they share a single grammar, parsed once and pretty-printed back the same way.

The result:

+ *Diffable* — every diagram change is a readable text diff. No "the binary changed" merge conflicts.

+ *Scriptable* — `.gnx` is generable from anywhere that can write a string. Templates, fixture builders, codegen targets, ad-hoc shell pipelines.

+ *Reviewable* — pull requests on diagram-bearing documentation no longer require opening a tool.

+ *Authorable in either direction* — type the DSL and watch the canvas update, or draw on the canvas and watch the DSL fall out.

GNX is intentionally small. It describes diagrams, not behaviour; styling, not simulation; geometry, not the semantics of what the nodes _do_. A node-graph compute model layered on top — or a circuit, or a state machine — is a downstream concern.

#pagebreak()

// ============================================================
// SECTION 2: DESIGN PRINCIPLES
// ============================================================

= Design Principles <principles>

+ *Round-trip first* — every legal document survives `parse → pretty → parse` unchanged. Style auto-extraction and comment preservation honour this invariant.

+ *Style classes over property repetition* — when two or more nodes share the same overlay and ports, they reference a named style. The pretty-printer factors styles out automatically; the parser merges them back in on load.

+ *Inline overrides* — any field declared inside a node body wins over the style. A single deviant fill or port placement does not force a new style.

+ *Comments as document structure* — comments anchor to the following item (file header, settings, node, wire). They survive round-trip and surface on the canvas wherever the host renders them.

+ *One physical unit base per document* — `units` is declared once in `settings`. Every coordinate inside the document is in those units. The renderer scales the page board (intrinsically defined in inches) to match.

+ *Forgiving lexer, strict parser* — whitespace, statement separators, and comment forms are flexible. Keywords, ordering inside a block, and enum spellings are not.

+ *No host-specific extensions* — a `.gnx` document does not depend on `egui_grafica` to be meaningful. Any conformant parser can reproduce the diagram.

#pagebreak()

// ============================================================
// SECTION 3: FILE TYPES
// ============================================================

= File Types <file-types>

#table(
  columns: (auto, 1fr, 2fr),
  align: (center, left, left),
  table.header(
    [*Extension*], [*Purpose*], [*Contents*]
  ),
  [`.gnx`], [Canvas / Diagram], [Settings, optional styles, nodes, wires],
)

There is exactly one file type. Reusable style libraries are not split into a separate file in the v0.1 spec — every `style` block lives in the document that uses it. A multi-document style import mechanism is a v0.2 candidate.

#pagebreak()

// ============================================================
// SECTION 4: LEXICAL STRUCTURE
// ============================================================

= Lexical Structure <lex>

== Encoding <lex-encoding>

`.gnx` files are UTF-8. The lexer reads `char`s; identifiers and keywords are ASCII; string literals carry arbitrary Unicode.

== Whitespace <lex-whitespace>

Spaces, tabs, and carriage returns are insignificant except as token separators. Newlines act as *statement terminators* inside a block — one statement per line is the canonical form, but a `;` is also accepted at the parser level.

== Comments <lex-comments>

Four forms are recognised; all four collapse to the same comment payload and survive the round-trip via anchored comment blocks.

#table(
  columns: (auto, auto, 1fr),
  align: (left, left, left),
  table.header(
    [*Form*], [*Origin*], [*Conventional Use*]
  ),
  [`// …`], [Rust line], [Ordinary inline comment],
  [`/// …`], [Rust outer doc], [Attached to the *following* item],
  [`//! …`], [Rust inner doc], [Attached to the *enclosing* block (typically the file header)],
  [`# …`], [Legacy GNX], [Back-compat; treated as `//` on parse, emitted as `//` on save],
)

A comment runs from the marker to end-of-line. There are no block comments. The pretty-printer emits the `//` canonical form; the `///` / `//!` distinctions are accepted on read but not preserved on write.

== Identifiers <lex-ids>

```ebnf
identifier := [A-Za-z_] [A-Za-z0-9_]*
```

Identifiers are case-sensitive. Node IDs, port IDs, and style names share the same lexical class.

== Numbers <lex-numbers>

```ebnf
number := [-+]? [0-9]+ ( '.' [0-9]+ )?
```

All numbers are interpreted as `f32` (IEEE 754 single precision). No exponent form, no hex form. Leading sign is permitted.

== Strings <lex-strings>

Double-quoted, with C-style escapes:

```ebnf
string := '"' ( escape | non-quote-char )* '"'
escape := '\' ( 'n' | 't' | '"' | '\' )
```

Strings carry arbitrary Unicode bytes between the quotes. Newlines inside a string must be escaped as `\n`.

== Punctuation <lex-punctuation>

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Token*], [*Role*]
  ),
  [`{` `}`], [Block delimiters],
  [`:`], [Type / style annotation (`node a : rect : my_style`) and port reference (`a.p1`)],
  [`.`], [Port reference inside a wire (`a.p1 -> b.p2`)],
  [`->`], [Wire direction],
)

== Keywords <lex-keywords>

Reserved at the top of their respective block contexts. They are not reserved globally — a node ID `style` is legal (though confusing).

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Keyword*], [*Context*]
  ),
  [`canvas`], [File root],
  [`settings`], [Inside `canvas`],
  [`style`], [Inside `canvas`],
  [`node`], [Inside `canvas`],
  [`wire`], [Inside `canvas`],
  [`text`], [Inside `node`, `style`],
  [`port`], [Inside `node`, `style`],
  [`at`, `size`, `rotation`], [Node transform],
  [`border`, `fill`], [Node / style overlay],
  [`routing`, `stroke`, `arrow`, `label`], [Inside `wire`],
  [`grid`, `grid_style`, `dot_size`, `units`, `snap`, `show_grid`, `paper`, `orientation`, `background`], [Inside `settings`],
  [`value`, `anchor`, `font`, `bold`, `italic`, `color`], [Inside `text`],
)

#pagebreak()

// ============================================================
// SECTION 5: TOP-LEVEL GRAMMAR
// ============================================================

= Top-Level Grammar <toplevel>

```ebnf
document    := comment* "canvas" string "{" canvas_body "}" ;
canvas_body := comment* settings_block
               ( comment* style_block
               | comment* node_block
               | comment* wire_block )* ;
```

The order is:

+ *Exactly one* `settings { … }` block — required, first item after the canvas opening brace.
+ *Zero or more* `style { … }` blocks — appear before nodes by convention; the parser accepts them anywhere after `settings`.
+ *Zero or more* `node { … }` blocks.
+ *Zero or more* `wire { … }` blocks.

A `.gnx` document is anchored by its `canvas "…" { }` root. The string is a user-facing document name and may be empty.

#pagebreak()

// ============================================================
// SECTION 6: SETTINGS
// ============================================================

= Settings Block <settings>

```ebnf
settings_block := "settings" "{" settings_field* "}" ;
settings_field := "grid" number
                | "grid_style" ( "lines" | "dots" )
                | "dot_size" number
                | "units" ( "pixels" | "mils" | "millimeters" | "inches" )
                | "snap" onoff
                | "show_grid" onoff
                | "routing" ( "orthogonal" | "bezier" | "straight" )
                | "paper" string
                | "orientation" string
                | "background" ( "light" | "slate" | "charcoal" | "dark" ) ;
onoff := "on" | "off" | "true" | "false" ;
```

== Field Semantics <settings-fields>

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Field*], [*Meaning*]
  ),
  [`grid`], [Grid step in *world units*.],
  [`grid_style`], [`lines` draws faint axis-aligned rulings; `dots` draws a marker at every intersection.],
  [`dot_size`], [World-unit diameter of dot markers when `grid_style dots`. Ignored for `lines`.],
  [`units`], [World-unit interpretation. Drives page-board scaling: 1 inch = 1, 1000, 25.4, or 96 world units for `inches`, `mils`, `millimeters`, `pixels` respectively.],
  [`snap`], [When `on`, placements and resizes snap to the nearest grid intersection.],
  [`show_grid`], [Renderer flag — the grid still exists for snapping when `off`.],
  [`routing`], [Default routing for new wires.],
  [`paper`], [Named paper size — `"Letter"`, `"Legal"`, `"Tabloid"`, `"A5"`, `"A4"`, `"A3"`, `"ANSI C"`, `"ANSI D"`. Omit to disable the page board.],
  [`orientation`], [`"portrait"` or `"landscape"`. Only meaningful with a `paper`.],
  [`background`], [Canvas tone.],
)

#pagebreak()

// ============================================================
// SECTION 7: STYLE
// ============================================================

= Style Block <style>

```ebnf
style_block := "style" identifier "{" style_field* "}" ;
style_field := border_field | fill_field | text_block | port_field ;
```

A style carries any subset of the overlay surface (border, fill, text) plus a list of ports. Every field is optional — a style that contains only `fill` and `port` lines is perfectly legal.

== Inheritance Semantics <style-inheritance>

When a node references a style with `node x : rect : my_style { … }`:

+ The named style is looked up in the document's style table.
+ Its fields *pre-seed* the node — `border`, `fill`, `text` become the node's overlay starting state; the style's `port` list is copied to the node verbatim.
+ The node body is then parsed top-to-bottom. Each inline field *overrides* the style's value.
+ Inline `port` declarations whose ID matches a style port replace that port in place; new IDs append.

A style with the same content but a different name is a different style. Names are compared as exact ASCII strings.

== Auto-extraction Policy <style-extraction>

When a scene is pretty-printed:

+ Every node's `(overlay, ports)` tuple is hashed against the other nodes.
+ Any tuple shared by *two or more nodes* is factored into a `style` block named after the first node that carried an explicit style reference, or `s0`, `s1`, … if none did.
+ Each member node emits `node id : kind : stylename { at … size … rotation … }` and *omits* every field equal to the style.
+ Nodes whose tuple is unique inline the full field set as before.

The printer also preserves any `style` block parsed but unreferenced by current nodes (a library style the user authored, say), so a round-trip never loses a declaration.

#invariant[
`parse(pretty(parse(text)))` is structurally equal to `parse(text)` for every valid GNX text — auto-extraction does not change the document's meaning.
]

#pagebreak()

// ============================================================
// SECTION 8: NODE
// ============================================================

= Node Block <node>

```ebnf
node_block := "node" identifier ":" node_kind ( ":" identifier )? "{" node_field* "}" ;
node_kind  := "rect" | "circle" | "ellipse" | "parallelogram" ;
node_field := "at" number number
            | "size" number number
            | "rotation" number
            | border_field | fill_field | text_block | port_field ;
```

== Transform Fields <node-transform>

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Field*], [*Description*]
  ),
  [`at x y`], [World-unit top-left of the node's bounding box.],
  [`size w h`], [World-unit width and height. Resize handles operate on this.],
  [`rotation deg`], [Counter-clockwise rotation around the bounding-box centre, in degrees.],
)

== Overlay Fields <node-overlay>

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Field*], [*Description*]
  ),
  [`border style width "#color"`], [Outline: `solid`, `dashed`, or `dotted`; width in world units; `#RRGGBB` colour.],
  [`fill "#color" alpha`], [Body fill: colour and alpha in `[0.0, 1.0]`.],
  [`text { … }`], [Optional label block (see Section 9).],
  [`port kind name anchor [args] [type "string"]`], [One port per line (see Section 10).],
)

== Kinds <node-kinds>

#table(
  columns: (auto, 1fr, 1fr),
  align: (left, left, left),
  table.header(
    [*Kind*], [*Visible contour*], [*Bbox interpretation*]
  ),
  [`rect`], [Axis-aligned rectangle.], [Direct.],
  [`circle`], [Ellipse inscribed in the bbox (square bbox → circle).], [Radial projection.],
  [`ellipse`], [Ellipse inscribed in the bbox.], [Radial projection.],
  [`parallelogram`], [Right-leaning parallelogram. Top edge inset by `h × 0.25`.], [Slanted-edge port projection.],
)

#invariant[
Port positions are computed against the *visible contour*, not the bounding box. A `circle` node's `east 0.5` port sits on the curve at the 3-o'clock point, not on the bbox corner.
]

#pagebreak()

// ============================================================
// SECTION 9: TEXT
// ============================================================

= Text Block <text>

```ebnf
text_block := "text" "{" text_field* "}" ;
text_field := "value" string
            | "anchor" text_anchor
            | "font" string number
            | "bold" onoff
            | "italic" onoff
            | "color" string ;
text_anchor := "center" | "top_center" | "bottom_center"
             | "left" | "right"
             | "top_left" | "top_right" | "bottom_left" | "bottom_right" ;
```

`font ""` means "use the host's default proportional family." `value` may contain `\n` for multi-line labels.

#pagebreak()

// ============================================================
// SECTION 10: PORT
// ============================================================

= Port Block <port>

```ebnf
port_field := "port" port_kind identifier port_anchor ( "type" string )? ;
port_kind  := "in" | "out" | "bidir" | "untyped" ;
port_anchor := "north" number
             | "south" number
             | "east"  number
             | "west"  number
             | "free"  number number ;
```

`north 0.5` means "midpoint of the top edge"; `east 0.0` is the top-right corner-ish; `west 1.0` is the bottom-left corner-ish (parametric along the face). `free fx fy` is normalised body-local coordinates — `free 0.5 0.5` is dead-centre regardless of shape.

The optional `type "string"` is reserved for typed connection validation in node-graph mode. Block-diagram diagrams leave it absent.

#pagebreak()

// ============================================================
// SECTION 11: WIRE
// ============================================================

= Wire Block <wire>

```ebnf
wire_block := "wire" identifier endpoint "->" endpoint "{" wire_field* "}" ;
endpoint   := identifier "." identifier ;
wire_field := "routing" wire_routing
            | "stroke" string number line_style
            | "arrow" arrow_head arrow_head
            | "label" string ;

wire_routing := "orthogonal" | "bezier" | "straight" | manual_routing ;
manual_routing := "manual" "[" ( number number )* "]" ;

line_style := "solid" | "dashed" | "dotted" ;
arrow_head := "none" | "arrow" | "triangle" | "diamond" | "circle" ;
```

== Endpoints <wire-endpoints>

`a.p1 -> b.p2` is a directed connection from node `a`'s port `p1` to node `b`'s port `p2`. The direction influences arrow placement and the routing algorithm's exit logic.

#host-note[
Free-floating endpoints (dangling wires) exist as an in-memory editor state but are *not* representable in `.gnx`. Saving a scene with free ends silently drops them.
]

== Routing <wire-routing>

#table(
  columns: (auto, 1fr),
  align: (left, left),
  table.header(
    [*Routing*], [*Behaviour*]
  ),
  [`orthogonal`], [Auto-routed; one bend, axis-aligned, port-direction-aware stubs.],
  [`bezier`], [Auto-routed; port-direction-aware control handles.],
  [`straight`], [Direct line between endpoints.],
  [`manual[ x0 y0 x1 y1 … ]`], [Hand-laid waypoints in world units. The router connects endpoints through each waypoint in order.],
)

#pagebreak()

// ============================================================
// SECTION 12: COORDINATE MODEL
// ============================================================

= Coordinate Model <coords>

GNX has *one coordinate system*: world units, axis-aligned, y-down. Every position and size in a `.gnx` file is in these units. There is no transform stack and no nested coordinate frames in v0.1.

The `units` setting is purely cosmetic — it controls the suffix in the inspector and the scaling of the engineering-drawing page board. Editing a file from `mils` to `mm` does *not* rescale numbers. If a sheet authored in mils is reinterpreted as mm, every shape is 25.4× larger.

A future v0.2 may add a `world_unit` declaration that the parser uses to physically rescale on unit change.

#pagebreak()

// ============================================================
// SECTION 13: PAGE BOARD
// ============================================================

= Page Board <page>

When `settings.paper` is set, the renderer draws an engineering-drawing sheet anchored at world origin:

- *Sheet* — paper-sized rectangle at `(0, 0)`. Outlined only; content sits on top.
- *Frame* — drawing border inset by 0.5 inch on every side.
- *Zone markers* — `A`–`H` along the horizontal frame edges, `1`–`6` along the vertical, bottom-up numbering (cell `A1` is bottom-left).
- *Title block* — optional bottom-right block carrying TITLE, COMPANY, DWG NO + REV, DATE + SHEET.

#host-note[
The page board is a host-level feature. The `.gnx` declaration is the `paper` and `orientation` settings. Title-block field content is currently a host-side editor concern; it does not round-trip through `.gnx` in v0.1.
]

#pagebreak()

// ============================================================
// SECTION 14: ROUND-TRIP INVARIANT
// ============================================================

= Round-Trip Invariant <roundtrip>

For every valid `Scene` value the implementation produces, the following must hold:

```rust
parse(pretty_document(&doc)) == doc.scene
```

Equality compares every field: nodes, edges, settings, style-extracted overlays, port lists, comment anchors. The reference implementation's test suite enforces this with a fixture covering all node kinds, all routing variants, comments at every anchor, and shared-style auto-extraction.

The only documented loss-of-information cases are:

+ *Comment style* — `//`, `///`, `//!`, and `#` all collapse to `//` on emit.
+ *Free-ended wires* — dropped on save.
+ *Title-block fields* — host-only in v0.1.

#invariant[
A conformant pretty-printer *must not* mutate scene semantics. It may re-order auto-extracted styles, rename auto-extracted style identifiers across runs, or re-format whitespace — but `parse` on its output must reproduce the input `Scene`.
]

#pagebreak()

// ============================================================
// SECTION 15: FILE LAYOUT EXAMPLE
// ============================================================

= File Layout Example <example>

A minimal but representative document:

```gnx
//! grafica-ide system diagram
//! Version 1, James Bonanno

canvas "Block Diagram" {
  settings {
    grid 10
    grid_style lines
    dot_size 2
    units mils
    snap on
    show_grid on
    routing orthogonal
    background slate
  }

  // Shared widget style — auto-extracted by the printer
  // when two or more nodes share it.
  style s0 {
    border solid 2 "#1F2937"
    fill "#DBEAFE" 0.9
    text {
      value "Text"
      anchor center
      font "" 12
      bold off
      italic off
      color "#111827"
    }
    port untyped n north 0.5
    port untyped e east  0.5
    port untyped s south 0.5
    port untyped w west  0.5
  }

  /// The system's primary input stage.
  node alpha : rect : s0 {
    at 850 175
    size 80 50
    rotation 0
    text {
      value "Alpha"
      anchor center
      font "" 12
      bold off
      italic off
      color "#111827"
    }
  }

  node beta : circle : s0 {
    at 1040 240
    size 80 80
    rotation 0
  }

  wire w0 alpha.e -> beta.w {
    routing bezier
    stroke "#374151" 1.5 solid
    arrow arrow none
  }
}
```

This document:

- Carries inner-doc header comments anchored to the canvas block.
- Declares one shared style with full overlay and four quadrant ports.
- Inlines a per-node `text` override on `alpha` (its label differs from the style's "Text" default).
- Connects `alpha.e` to `beta.w` with a bezier wire.

#pagebreak()

// ============================================================
// SECTION 16: CONFORMANCE
// ============================================================

= Conformance <conformance>

== Required Behaviour <conformance-must>

A conformant GNX implementation *must*:

+ Accept every legal document under this specification.
+ Reject documents that violate the grammar with a useful, line-numbered error.
+ Preserve every field of a scene round-trip through `parse → pretty → parse`.
+ Recognise all four comment forms on read.
+ Compute port positions on the visible contour, not the bbox, for `circle`, `ellipse`, and `parallelogram` nodes.

== Permitted Behaviour <conformance-may>

A conformant implementation *may*:

- Emit any comment form on write (the reference implementation emits `//`).
- Reorder nodes, wires, or styles within their declaration sections, provided the round-trip identity holds.
- Auto-extract styles from shared overlays on write.
- Reject extensions not listed in this document (forward compatibility is not guaranteed in v0.1).

#pagebreak()

// ============================================================
// SECTION 17: REFERENCE IMPLEMENTATION
// ============================================================

= Reference Implementation <impl>

The reference parser and pretty-printer live in `crates/egui_grafica/src/lang.rs` in the `saturn77/egui_mobius` repository. The model types referenced throughout this spec — `Scene`, `CanvasSettings`, `Style`, `Node`, `Edge`, `Port`, `Overlay` — are defined in `crates/egui_grafica/src/model.rs`.

The syntect grammar for editor highlighting lives at `crates/egui_quill/syntaxes/Graphica.sublime-syntax`.

This document lives at `develop/gnx_language_spec.md` (markdown) and `develop/gnx_language_spec.typ` (Typst source for the PDF build). Build the PDF with:

```bash
cd develop
./build_gnx_spec.sh
```

The build script stamps the current git short hash and build timestamp into the title page so a printed copy is traceable to a specific revision.
