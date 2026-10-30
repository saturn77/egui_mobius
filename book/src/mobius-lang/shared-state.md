# Shared state — cells, roles, and the wire

Everything on Path A is declared in an `interface`: typed cells with
defaults, and **roles** that say which side of the boundary may do what
to each cell. This chapter's example is real — it is the `ToolStyle`
interface from the ShutterFrame Properties pilot, and the generated Rust
shown below is what `mobius-gen` actually emits.

## Declaring the state

```mobius
interface ToolStyle {
    theme : Theme = Theme::CopperDark
    stroke_width : f32 = 2.0
    fill : bool = false
    text_size : f32 = 18.0

    role props (write theme, write stroke_width, write fill, write text_size)
    role host  (read theme,  read stroke_width,  read fill,  read text_size)
}
```

Each state field becomes one `Dynamic<T>` cell. The two roles are the
SystemVerilog-modport idea: the same bundle of wires seen from two
seats. `write` means *this role's hand is on the pen* (writing implies
reading); `read` means observe only. **Every cell has exactly one
writing role** — that is the one-writer discipline, and the checker
enforces it.

The generated state is ordinary Rust — no macros, no magic:

```rust
pub struct ToolStyle {
    pub theme: Dynamic<Theme>,
    pub stroke_width: Dynamic<f32>,
    pub fill: Dynamic<bool>,
    pub text_size: Dynamic<f32>,
}
```

## The wire: `--`

A widget connects to a cell with the wire operator, and the **direction
comes from the role**, not from the widget:

```mobius
citizen Properties (s : ToolStyle.props) {
    column {
        slider "Width" 1.0..12.0 -- s.stroke_width;   // props writes → two-way
        checkbox "Fill" -- s.fill;                     // props writes → two-way
    }
}

citizen Host (h : ToolStyle.host) {
    column {
        label -- h.stroke_width;     // host reads → display only
        checkbox "fill" -- h.fill;   // host reads → rendered DISABLED
    }
}
```

The same `checkbox -- …fill` line produces an editable checkbox on the
writing side and a *disabled* one on the reading side. You never say
which; the role already said it. (`<->` and `<-` exist as explicit
overrides, but in practice the wire covers almost everything.)

## What the wire generates

The write side is the book's canonical get → widget → set-on-change
loop ([Stored vs stateless panels](../patterns/stored-vs-stateless.md)),
emitted for you:

```rust
let mut value = s.stroke_width.get();
if ui.add(egui::Slider::new(&mut value, 1.0..=12.0).text("Width")).changed() {
    s.stroke_width.set(value);
}
```

The read side is just `get()` into a display. Nothing is ever passed
`&mut`; citizens hold `&ToolStyle` and the cells do the synchronization.

## Banks and loops

Fixed-size collections are array cells — one `Dynamic` per element —
with Verilog-style generate loops to wire them:

```mobius
labels : [string; 4] = {"capture 0", "capture 1", "capture 2", "capture 3"}
```

```mobius
for i in 0..4 {
    row {
        label -- t.labels[i];
        button "Open" -> t.open(i);
    }
}
```

The loop unrolls at generation; the index lands in both the cell access
(`t.labels[2].get()`) and, as the next chapter shows, in event payloads.
