# Reactive GUI with `masonry` + `egui_mobius_reactive`

This example demonstrates how the [`xilem-masonry`](https://docs.rs/masonry/latest/masonry/) crate—a foundational framework for GUI development—can be extended and enhanced with the modular, reactive capabilities of the [`egui_mobius_reactive`](https://crates.io/crates/egui_mobius_reactive) crate.

The `masonry` crate is designed to serve as the core of higher-level GUI architectures. The authors suggest building Elm-like, functional-reactive, or immediate-mode frameworks on top of it. This example is an exercise in doing just that—leveraging `egui_mobius_reactive` as a general-purpose reactive GUI crate to showcase its versatility beyond the `egui_mobius` ecosystem.

Future directions may include connecting `egui_mobius_reactive` to `bevy`, as well as exploring **more sophisticated examples** using `masonry`, particularly those involving *layout and styling*.

---

## Requirements

- Rust stable (1.70+)
- [`masonry`](https://crates.io/crates/masonry)
- [`egui_mobius_reactive`](https://crates.io/crates/egui_mobius_reactive)

---

## Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/saturn77/egui_mobius.git
   ```

2. Build and run the example:

   ```bash
   cargo run -p masonry-reactive
   ```

---

## Overview

### About `masonry`

The `masonry` crate, part of the [`xilem`](https://github.com/linebender/xilem) ecosystem, is a powerful and flexible framework for building modern, declarative GUIs in Rust. It provides:

- A **widget-based architecture** for creating user interfaces.
- A **global theming system** for consistent styling.
- Support for **reactive state** and **event-driven programming**.

### About `egui_mobius_reactive`

The `egui_mobius_reactive` crate introduces **reactive programming** to GUI development. It provides:

- `Dynamic<T>` and `Derived<T>` types for managing reactive state.
- A `SignalRegistry` for tracking and updating signals.
- A modular interface that integrates into other GUI frameworks like `masonry`.

In this example, these two libraries are brought together to simplify and enhance state-driven GUI development.

---

## Features of This Example

### 1. **Reactive State Management**

Reactive state is handled using `Dynamic<i32>` and `Derived<i32>` values:

- `click_count` tracks how many times the button has been pressed.
- `doubled` computes 2× the click count.
- `sum` combines both values.
- `message` shows a human-readable message.

Changes to these values automatically update the UI—no manual redrawing needed.

### 2. \*\*Integration with \*\***`masonry`**

- Uses `masonry` widgets like `Button`, `Label`, `Flex`, and `RootWidget`.
- Reactive values are displayed and updated via standard `masonry` layouts.
- The `AppDriver` trait decouples the UI from the state.

### 3. **Modular Architecture**

- `egui_mobius_reactive` remains independent of the GUI backend.
- You could reuse the same reactive logic with another renderer, like `bevy`.

---

## Code Highlights

### Reactive State in `AppState`

```rust
pub struct AppState {
    click_count : Dynamic<i32>,
    doubled     : Derived<i32>,
    sum         : Derived<i32>,
    message     : Dynamic<String>,
}

impl AppState {
    pub fn new() -> Self {
        let click_count = Dynamic::new(0);
        let doubled = click_count.doubled();
        let sum = click_count.clone() + doubled.clone();
        let message = Dynamic::new("Click the button!".to_string());

        let registry = SignalRegistry::default();
        registry.register_named_signal("click_count", Arc::new(click_count.clone()));
        registry.register_named_signal("doubled", Arc::new(doubled.clone()));
        registry.register_named_signal("sum", Arc::new(sum.clone()));
        registry.register_named_signal("message", Arc::new(message.clone()));

        Self {
            click_count,
            doubled,
            sum,
            message,
        }
    }
}
```

### Handling Actions in `AppDriver`

`masonry` decouples UI from state. The `AppDriver` trait (similar to `eframe::App`) defines how user actions update application state.

```rust
struct Driver {
    state: Arc<AppState>,
}

impl AppDriver for Driver {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: Action) {
        match action {
            Action::ButtonPressed(_) => {
                let count = self.state.click_count.get() + 1;
                self.state.click_count.set(count);
                std::thread::sleep(std::time::Duration::from_micros(200));

                let doubled = self.state.doubled.get();
                println!("Count: {}, Doubled: {}", count, doubled);
                self.state.message.set(format!("Count: {} × 2 = {}", count, doubled));

                let sum_string = format!("Derived Value `sum` = {}", self.state.sum.get());

                ctx.render_root().edit_root_widget(|mut root| {
                    let mut root = root.downcast::<RootWidget<Flex>>();
                    let mut flex = RootWidget::child_mut(&mut root);
                    Flex::clear(&mut flex);

                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);
                    Flex::add_child(&mut flex, Label::new(self.state.message.get())
                        .with_style(StyleProperty::FontSize(24.0))
                        .with_style(StyleProperty::FontWeight(FontWeight::BOLD)));
                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);
                    Flex::add_child(&mut flex, Label::new(sum_string)
                        .with_style(StyleProperty::FontSize(24.0))
                        .with_style(StyleProperty::FontWeight(FontWeight::BOLD)));
                    Flex::add_spacer(&mut flex, VERTICAL_WIDGET_SPACING);
                    Flex::add_child(&mut flex, Button::new("Click me!"));
                });
            }
            action => {
                eprintln!("Unexpected action {action:?}");
            }
        }
    }
}
```

### Main Application Entry Point

```rust
fn main() {
    let state = Arc::new(AppState::new());

    let label = Label::new(state.message.get())
        .with_style(StyleProperty::FontSize(32.0))
        .with_style(StyleProperty::FontWeight(FontWeight::BOLD));
    let button = Button::new("Click me!");

    let main_widget = Flex::column()
        .with_child(label)
        .with_spacer(40.0)
        .with_child(button);

    let window_size = LogicalSize::new(400.0, 400.0);
    let window_attributes = Window::default_attributes()
        .with_title("Reactive Counter")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    masonry::app::run(
        masonry::app::EventLoop::with_user_event(),
        window_attributes,
        RootWidget::new(main_widget),
        Driver {
            state: state.clone(),
        },
    )
    .unwrap();
}
```

---

## Next Steps

- Add layout experiments with `masonry`'s styling system.
- Create reusable reactive components.
- Explore integration with `bevy` or `dioxus`.

---

Feel free to fork, contribute, or open issues!

