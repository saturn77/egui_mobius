# egui_mobius_macros

🚀 **Procedural Macros for egui_mobius** 🚀  
This crate provides powerful **procedural macros** to simplify event handling in the `egui_mobius` framework.  

## **Overview**
`egui_mobius_macros` provides the `#[derive(EventMacro)]` macro, which **automatically implements key traits** and utilities for event enums in `egui_mobius`.  

### **Key Features:**
✅ **Auto-generates `event_name()`** – Get the event type as a string.  
✅ **Implements `Debug` and `Clone`** – No manual trait implementations needed.  
✅ **Supports all enum styles** – Works with unit, tuple, and struct-like variants.  

---

## **Installation**
Add `egui_mobius_macros` to your `Cargo.toml`:

```toml
[dependencies]
egui_mobius_macros = { path = "../egui_mobius_macros" }
```

Or, when published on **crates.io**:
```toml
[dependencies]
egui_mobius_macros = "0.1"
```

---

## **Usage**
Simply derive `EventMacro` on an enum:

```rust
use egui_mobius_macros::EventMacro;

#[derive(EventMacro, PartialEq, Clone)]
enum MyEvent {
    RefreshUI,
    UpdateData(String),
    Custom(i32),
}

fn main() {
    let event = MyEvent::UpdateData("Hello!".to_string());

    println!("Event name: {}", event.event_name()); // Outputs: UpdateData
}
```

### **What Happens Under the Hood?**
- **`event_name()`** method is auto-generated:
  ```rust
  impl MyEvent {
      pub fn event_name(&self) -> &'static str {
          match self {
              MyEvent::RefreshUI => "RefreshUI",
              MyEvent::UpdateData(..) => "UpdateData",
              MyEvent::Custom(..) => "Custom",
          }
      }
  }
  ```
- **`Debug` implementation:**  
  ```rust
  println!("{:?}", event); // Outputs: "UpdateData event"
  ```

---

## **Supported Enum Variants**
This macro works with all Rust enum styles:

#### ✅ **Unit Variants**
```rust
#[derive(EventMacro, Clone)]
enum SimpleEvent {
    Start,
    Stop,
}
```

#### ✅ **Tuple Variants**
```rust
#[derive(EventMacro, Clone)]
enum TupleEvent {
    Resize(u32, u32),
    UpdateData(String),
}
```

#### ✅ **Struct-like Variants**
```rust
#[derive(EventMacro, Clone)]
enum StructEvent {
    Custom { id: u32, name: String },
}
```

---

## **Advanced Features (Planned)**
🚀 **Trait-Based Event Dispatching** – Automatic conversion to `dyn EventTrait`.  
🚀 **`From<T>` Implementations** – Simplified event conversions.  
🚀 **Integration with `Slot<T>`** – Automatic event-slot binding.  

---

## **Contributing**
Contributions are welcome! Feel free to:
- Open an issue 💡
- Submit a pull request 🚀
- Improve documentation 📚

---

## **License**
This project is licensed under the **MIT License**.

---

### 🎉 **Happy coding with `egui_mobius`!** 🚀

