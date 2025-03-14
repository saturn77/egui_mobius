<div align="center">
<img width=260 height=200 src="./assets/mobius_strip.png"></img>

# egui_mobius  
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui_version](https://img.shields.io/badge/egui-0.31-blue)](https://github.com/emilk/egui)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Latest Version](https://img.shields.io/badge/version-0.3.0alpha-green.svg)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)

</div>

egui_mobius is a reactive UI and messaging framework for Rust, built on top of egui and eframe. It provides a signal–slot (dispatcher) architecture and reactive state management via its Value<T> type, enabling decoupled communication between UI components and backend logic.

# Features

Inspired by modern UI frameworks and event-driven architectures, egui_mobius helps you build interactive, responsive UIs with minimal boilerplate.

* Reactive State Management: 
Use Value<T> to automatically update your UI when state changes.

* Signal–Slot Architecture:
Connect events to handlers (slots) using a generic, typed dispatcher that supports named channels.

* Dispatcher:
Send events across named channels to decouple producers and consumers. Reuse the same mechanism throughout your application.

* Modular Design:
Separate your UI, backend, event, and state logic into reusable modules.

* Integration with eframe:
Leverage egui’s immediate-mode rendering for fast and portable applications.

## Contributing  
* Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  


* This project is licensed under the MIT License.  

* For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
