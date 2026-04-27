<div align="center">
<img width=260 height=200 src="https://raw.githubusercontent.com/saturn77/egui_mobius/master/assets/mobius_strip.png"></img>

# egui_mobius
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui](https://img.shields.io/badge/egui-0.33-blue)](https://github.com/emilk/egui)
[![egui_dock](https://img.shields.io/badge/egui__dock-0.18-purple)](https://github.com/Adanos020/egui_dock)
[![Crates.io](https://img.shields.io/crates/v/egui_mobius.svg)](https://crates.io/crates/egui_mobius)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)
[![Book](https://img.shields.io/badge/📖_book-egui--citizen-orange)](https://saturn77.github.io/egui_mobius/)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

`egui_mobius` is a workspace of coordinated crates forming a *framework*
for building graphical user interfaces based on egui. 

The focus is the **egui_citizen** framework — first-class dock panel lifecycle, persistent identity, reactive state,
and central message dispatch. The
[book](https://saturn77.github.io/egui_mobius/) covers it in depth.

Underneath, a broader stack provides the building blocks: reactive
primitives (`Dynamic<T>`, `Derived<T>`), an async runtime, and
signal-slot architecture. These are part of **egui_mobius** and **egui_mobius_reactive** crates. 


| Crate | Role |
|-------|------|
| **`egui_citizen`** | First-class citizen pattern — dock panel lifecycle, identity, message dispatch. **Recommended framework.** |
| `egui_mobius_reactive` | Thread-safe reactive primitives: `Dynamic<T>`, `Derived<T>`, `SignalRegistry`. |
| `egui_mobius` | Core signal-slot and dispatching system. |
| `egui_mobius_widgets` | Stateful widget toolkit for retained-mode-style composition. |
| `egui_mobius_components` | Higher-level UI components (event logger, etc.). |




## Getting Started

Read the [egui-citizen book](https://saturn77.github.io/egui_mobius/) for
the concept-by-concept guide. Or jump straight to a runnable example:

```bash
# Citizen pattern
cargo run -p citizen_dock          # citizen + egui_dock, three panels
cargo run -p citizen_fetch         # backend threading with HTTP fetch
cargo run -p getting_started       # smallest possible citizen example

# Reactive / async / signal-slot
cargo run -p clock_reactive        # modern reactive UI with minimal boilerplate
cargo run -p clock_async           # thread-aware async with clean UI feedback
cargo run -p reactive_slider       # ReactiveWidgetRef composition
cargo run -p logger_component      # event logger for sophisticated tracking
```

For a full project skeleton, see the [template repository](https://github.com/saturn77/egui_mobius_template).

## Contributing

- Contributions welcome! Fork the repository, create a feature branch, and submit a pull request.
- Licensed under the MIT License.
- For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
