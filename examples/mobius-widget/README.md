# Mobius Reactive Widgets Demo

One can run the application by using the cargo run command from anywhere in the crate : 

```sh
cargo run -p mobius-widget
```

This example demonstrates how to use **`egui_mobius_reactive`** to create employing composition via the **`MobiusWidget`** trait to build reactive widgets in a GUI application built with **Egui** and **eframe**. The example includes two widgets:

1. **Counter Widget**: A simple counter that can be incremented and decremented.
2. **Logger Widget**: A text area that displays log messages from the counter widget.

The widgets are connected using **reactive state** (`Dynamic<T>`), allowing them to update and interact seamlessly.

---

## Why This Design Architecture?

The architecture of this example is built around the **`MobiusWidget` trait**, which serves as a "hook" for creating modular and reusable widgets in Egui applications. This design facilitates **clean composition** and **separation of concerns**, making it easy to build complex UIs by combining smaller, self-contained components.

### Key Traits in the Architecture

1. **`MobiusWidget`**:
   - The core trait that defines the interface for rendering widgets.
   - Provides a clean abstraction for integrating widgets into the Egui UI.

2. **`MobiusWidgetReactive`**:
   - Extends `MobiusWidget` to enable reactive state binding.
   - Widgets can dynamically bind to `Dynamic<T>` or other reactive types, allowing seamless updates when the state changes.

3. **`MobiusWidgetSlot`**:
   - Adds support for **slots**, enabling widgets to act as placeholders for dynamic content.
   - Useful for building layouts or containers that can host other widgets.

4. **`MobiusWidgetSignal`**:
   - Enables widgets to emit **signals** that other widgets or systems can respond to.
   - Facilitates event-driven communication between widgets, such as triggering actions or propagating state changes.


As an example, consider a **`MobiusWidget`** hook implementation for a geneatic **CounterWidget** that is composed of a two buttons and a text string displaying the count, as well as **updating** the state of another widget, which is a logger : 

```rust
use egui_mobius_reactive::*; 

pub struct CounterWidget {
    pub count : Dynamic<i32>,
    pub log   : Option<Dynamic<String>>,
}

impl Default for CounterWidget {
    fn default() -> Self {
        Self {
            count: Dynamic::new(0),
            log: None,
        }
    }
}

impl MobiusWidget for CounterWidget {
    fn render_widget(&self, ui: &mut egui::Ui) {
        use egui::Color32;

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Reactive Counter").color(Color32::GREEN).heading());
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.count.set(self.count.get() - 1);
                }
                ui.label(egui::RichText::new(format!("Count: {}", self.count.get())).color(Color32::GREEN));
                if ui.button("+").clicked() {
                    self.count.set(self.count.get() + 1);
                    if let Some(log) = &self.log {
                        log.set(format!("Incremented to {}", self.count.get()));
                    }
                }
            });
        });
    }

    fn widget_name(&self) -> &str {
        "CounterWidget"
    }
}

```

The one can decide whether to attach a **Signal<T>**, a **Slot<T>**, or a **`Dynamic<T>`** to add the properties
to the widget. In this case, since the widget is *reactive*, the
**`MobiusWidgetReactive`** trait hook is implemented for the 
CounterWidget. 

```rust
impl MobiusWidgetReactive for CounterWidget {
    fn with_dynamic(&mut self, state: Arc<dyn std::any::Any>) {
        if let Some(d) = state.downcast_ref::<Dynamic<i32>>() {
            self.count = d.clone();
        }
        if let Some(d) = state.downcast_ref::<Dynamic<String>>() {
            self.log = Some(d.clone());
        }
    }
}

```
### Benefits of This Architecture

- **Modularity**:
  - Each widget is self-contained and implements the necessary traits, making it easy to reuse and compose widgets in different contexts.

- **Reactive State Management**:
  - The `MobiusWidgetReactive` trait allows widgets to bind to reactive state (`Dynamic<T>`), ensuring that UI updates are automatically propagated when the state changes.

- **Dynamic Composition**:
  - Using dynamic dispatch (`Arc<dyn MobiusWidget>`), widgets can be composed and managed at runtime, enabling flexible and extensible UI designs.

- **Event-Driven Communication**:
  - The `MobiusWidgetSignal` trait provides a mechanism for widgets to emit and respond to signals, enabling decoupled interactions between components.

This architecture is particularly well-suited for building **scalable**, **maintainable**, and **reactive** Egui applications.

---

## Features

- **Reactive State Management**:
  - The `Dynamic<T>` type is used to manage the state of the widgets.
  - Changes to the state automatically propagate to the widgets.

- **Dynamic Dispatch**:
  - Widgets implement the `MobiusWidget` and `MobiusWidgetReactive` traits, enabling dynamic composition and runtime flexibility.

- **Widget Interactions**:
  - The counter widget logs its state changes (e.g., increments and decrements) to the logger widget.

---

## How It Works

1. **Counter Widget**:
   - Displays a counter value that can be incremented or decremented.
   - Uses a `Dynamic<i32>` to manage its state.

2. **Logger Widget**:
   - Displays log messages in a text area.
   - Uses a `Dynamic<String>` to manage the log text.

3. **Reactive State Binding**:
   - The `with_dynamic` method is used to bind reactive state (`Dynamic<T>`) to the widgets.
   - The counter widget updates the logger widget by modifying the shared `Dynamic<String>` state.

---

