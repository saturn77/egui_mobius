# Styled Buttons

```bash
cargo run -p styled_buttons
```

# Introduction

This example illustrate the use of egui_mobius_widgets. These
widgets have an emphasis on being stateful, in that internal
state is reflected into the display of the widget. 

The Signal! macro can be attached to these widgets, for sending
signals to the backend or core container in an application. 

# Egui Frame API

The widgets make heavy use of the egui Frame API for CSS like 
styling. Note that there is a ButtonStyle struct which is 
completely optional! So to use a stateful button, you are not
required to style it. 

The only required arguments are the id and name of the button. 

For example : 

```rust
button_start: StatefulButton::new(
    0,
    "Start Process",
    ButtonStyle {
        stroke_size          : Some(2),
        stroke_color         : Some(Color32::DARK_GREEN),
        hovered_color        : Some(Color32::DARK_BLUE),
        stroke_size_on_hover : Some(4), 
        corner_radius        : Some(5),
        inner_margin         : Some(Margin::same(8)),
    },
),
```

Here the primary CSS like properties of Frame are exposed as 
properties of the stateful button. 