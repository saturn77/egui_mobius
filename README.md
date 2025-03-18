<div align="center">
<img width=260 height=200 src="./assets/mobius_strip.png"></img>

# egui_mobius  
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui_version](https://img.shields.io/badge/egui-0.31.1-blue)](https://github.com/emilk/egui)
[![egui_taffy](https://img.shields.io/badge/egui__taffy-0.7.0-purple)](https://github.com/Veykril/egui_taffy)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Latest Version](https://img.shields.io/badge/version-0.3.0--alpha.6-green.svg)
![Crates.io](https://img.shields.io/badge/crates.io-unreleased-orange.svg)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

egui_mobius is a layer 2 solution that transforms egui's immediate mode foundation into a complete application development platform. It bridges the gap between egui's efficient rendering and the architectural needs of production applications through a Qt-inspired signal-slot system, sophisticated state management, and clean separation of UI and business logic.

# Features

Inspired by production-grade GUI frameworks, egui_mobius addresses the key challenges of building maintainable Rust applications while preserving egui's performance and simplicity:

* **Enhanced State Management:**
  Thread-safe state persistence with automatic UI updates via Value<T>. Preserves widget state (sliders, radio buttons) between updates while maintaining proper ownership and thread safety through Mutex guards.

* **Thread-Aware Signal-Slot Architecture:**
  Unlike the Signal types in frameworks like Leptos or Dioxus, egui_mobius's slots maintain their own threads, enabling true hybrid synchronous/asynchronous operation. This approach allows slots to handle both immediate UI updates and long-running background tasks without blocking.

* **Flexible Dispatcher System:**
  Integral to egui_mobius is its dispatching system. The standard Dispatcher manages signal-slot connections through named channels and priority queues, with an AsyncDispatcher variant available for specialized async workloads using Tokio. This flexibility enables clean handling of both synchronous UI events and asynchronous background operations.

* **True Concurrent Processing:**
  Each slot's dedicated thread enables genuine parallel execution, not just asynchronous scheduling. Background tasks like clock updates run independently of the UI thread, with type-safe message passing ensuring thread-safe communication.

* **Structured Code Organization:**
  Clear separation of concerns through dedicated modules. Background operations (like clock generation) are cleanly extracted into standalone functions, improving maintainability and testability.

* **Event Traceability:**
  Type-safe message passing between UI and background threads enables clear tracking of event flow. The signal-slot architecture naturally supports adding custom logging and debugging capabilities.

* **Production-Focused Design:**
  Being built with real-world applications in mind, incorporating architectural patterns from mature GUI frameworks. While still evolving, the signal-slot system provides a foundation for managing complex UI state and background operations.

* **Taffy Layout Integration:**
  Full support for Taffy's powerful layout engine, enabling complex, responsive layouts with flexbox-style controls. Seamlessly integrates with egui's native layout system while providing additional capabilities for sophisticated UI designs.


## Versioning

egui_mobius follows semantic versioning and is currently in its alpha phase (0.3.0-alpha.5). This version represents:

- A mature architectural foundation with thread-aware slots
- Production-ready core features including type-safe messaging
- Ongoing API refinements based on real-world usage
- Full compatibility with egui 0.31.1

See [VERSIONING.md](VERSIONING.md) for our complete version strategy and compatibility matrix.

## Contributing  
* Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  


* This project is licensed under the MIT License.  

* For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
