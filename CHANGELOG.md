# Changelog

## [0.3.0-alpha.31] - 2025-04-25

### Added
- Added new `egui_mobius_components` crate, a collection of reusable UI components for the egui_mobius framework
- Added EventLogger component:
  - Terminal-like widget for logging events with different severity levels
  - Supports rich text formatting with customizable colors
  - Thread-safe implementation compatible with the egui_mobius signal/slot architecture
  - Support for categorizing logs by sender type and custom log styles
  - Built-in timestamping and filtering capabilities
  - Complete with examples that demonstrate usage in multi-threaded environments
- Added `logger_component` example that demonstrates the use of the EventLogger component

### Changed
- Enhanced the workspace structure to include the new components crate
- Updated the prelude pattern across all crates for more consistent imports

### Fixed
- Fixed release tools to correctly handle workspace inheritance in Cargo.toml files
- Updated formatting in platform.banner module for clippy compliance

## [0.3.0-alpha.30] - 2025-04-23

### Added
-- Added new feature, ReactiveWidgetRef, with Weak<T> references that facilitates using
components in a more retained-style manner or modular composition. This will cut down 
on the use of Arc and clone inside of a reactive widget.
-- Added example "reactive_slider" to demonstrate the use of ReactiveWidgetRef.
### Changed

### Fixed

## [0.3.0-alpha.29] - 2025-04-13

### Added
-- New MobiusWidget trait, along with MobiusWidgetReactive, MobiusWidgetSlot, MobiusWidgetSignal 
allowing for dynamic composition of widgets in an ecapsulated context providing greater flexibiity
to egui_mobius overall. Each widget can have a Dynamic<T>, Signal<T>, or Slot<T> attached to it. 
-- Added an example "mobius-widget" to the examples to illustrate the use of this new composition
style 

### Changed

### Fixed

## [0.3.0-alpha.28] - 2025-04-05

### Added
-- Example `masonry-reactive` demonstrating egui_mobius_reactive integrating with `masonry` from `xilem`

-- Added appropriate README.md in the example; highlighting the versatility of `egui_mobius_reactive`
### Changed

### Fixed

## [0.3.0-alpha.27] - 2025-04-01

### Added
-- Added reactive_math for easier handling of math operation involving Derived<T> and Dynamic<T>

### Changed
-- Updated the examples for reactive and clock_reactive

### Fixed

## [0.3.0-alpha.26] - 2025-03-29

### Added

### Changed
- Modified readme.md to reflect the evolving focus on egui_mobius_reactive and the async runtime
- Updated the getting started description
- Added links to the new egui_mobius_template repository
### Fixed

## [0.3.0-alpha.25] - 2025-03-27

### Added
- Added new MobiusReactive runtime, enabling an async runtime for egui_mobius, runtime.rs
- Added start_async method on Slot type, updated in slot.rs
### Changed
- Changed the clock_reactive example to be fully reactive + async, by making use of new MobiusRuntime

### Fixed

## [0.3.0-alpha.24] - 2025-03-26

### Added

### Changed
- Updated docstrings in lib.rs for egui_mobius_reactive, better description and example
### Fixed
- Clippy lint issues in types.rs & styled_buttons.rs 

## [0.3.0-alpha.23] - 2025-03-23

### Added
- Added Dynamic<T> to reactive system - replacing the `Value<T>` type that was in the reactive system to avoid confusion with the `Value` type in egui_mobius crate
- Updated all downstream tests and examples to use `Dynamic<T>`
- Added `From<Derived<T>>` implementation for `Dynamic<T>` to allow conversion from `Derived<T>` to `Dynamic<T>`

### Changed
- Updated `ValueExt::on_change` doctests to include proper imports and delays to ensure callback execution.

### Fixed


## [0.3.0-alpha.22] - 2025-03-23

### Added
- Created clock_reactive example, which is the same functionality of clock_async
- Features of clock_reactive include smaller code base and reactive state management
- Provides counterpoint to the clock_async example

### Changed

### Fixed

## [0.3.0-alpha.21] - 2025-03-22

### Added
- Added `Debug` implementation for `Value<T>` where `T` implements `Debug`.
- Added missing `cargo doc`-style comments to `Value` and `Derived` modules, including examples for methods like `on_change`, `new`, `get`, and `set`.
- Added comprehensive tests for `Value` and `Derived` to ensure thread safety and callback functionality.
- Added `From<Derived<T>>` implementation for `Value<T>` to allow conversion from `Derived<T>` to `Value<T>`.

### Changed
- Updated `ValueExt::on_change` doctests to include proper imports and delays to ensure callback execution.

### Fixed
- Fixed doctest failures in `Value` and `Derived` caused by missing imports and incorrect module paths.

## [0.3.0-alpha.20] - 2025-03-22

### Added
- Added `Default` implementation for `ReactiveList<T>` to simplify initialization.
- Introduced `type Subscribers` alias to improve readability and reduce type complexity in the reactive system.
- Added Clippy fixes for redundant closures and complex type definitions.
- Enhanced `ReactiveValue` trait with improved examples and documentation for `subscribe` and `as_any` methods.
- Added comprehensive tests for `ReactiveList` and `Value` to ensure thread safety and callback functionality.

### Changed
- Refactored `ReactiveList` to use `type Subscribers` for managing callback subscribers, improving code clarity.
- Simplified `subscribe` method in `ReactiveValue` by removing redundant closures.
- Improved documentation for `ReactiveList` and `Value` with clearer examples and explanations.
- Updated `on_change` and `subscribe` methods to ensure proper thread-safe callback execution.

### Fixed
- Fixed Clippy warnings related to redundant closures and complex type definitions.
- Fixed `Default` implementation for `ReactiveList<T>` to ensure proper trait bounds (`Clone + Send + Sync + 'static`) are enforced.
- Resolved doctest failures in `ReactiveList` and `Value` caused by missing imports and incorrect module paths.
- Fixed potential deadlock issues in `ReactiveList` by ensuring proper lock handling in `notify_subscribers`.
  
## [0.3.0-alpha.19] - 2025-03-22

### Added
- Added comprehensive `Debug` implementation for `Value<T>` where `T` implements `Debug`.
- Added missing `cargo doc`-style comments to `Value` and `Derived` modules, including examples for methods like `on_change`, `new`, `get`, and `set`.

### Changed
- Updated `ValueExt::on_change` doctests to include proper imports and delays to ensure callback execution.
- Replaced `Into<Value<T>>` implementation for `Derived<T>` with `From<Derived<T>>` for better adherence to Rust best practices.
- Improved documentation for `ValueExt` and `Derived` to clarify usage and provide working examples.

### Fixed
- Fixed unresolved import issues in `ValueExt` doctests by correcting the module paths.
- Fixed assertion failures in `ValueExt::on_change` doctests by adding appropriate delays to ensure callback execution.
- Fixed doctest failures caused by missing imports and incorrect module paths.

## [0.3.0-alpha.18] - 2025-03-21

### Added
- Added new reactive example showcasing:
  - Thread-safe state sharing pattern
  - Comprehensive AsyncDispatcher usage

### Changed
- Change toml files for examples so they can be run from the root crate
- Updated reactive example to use `Value::set()` for proper change notification
- Updated reactive example documentation with clearer usage instructions

### Fixed
- Fixed ownership issues in reactive system doc tests

## [0.3.0-alpha.17] - 2025-03-21
### Fixed
- Fixed logo image not displaying on crates.io by using absolute GitHub URL

## [0.3.0-alpha.16] - 2025-03-21

### Added

### Changed
- Improved reactive system documentation with clearer examples

### Fixed
- Fixed doctest issues in egui_mobius_reactive
- Fixed reactive update timing in examples

## [0.3.0-alpha.15] - 2025-03-20

### Added
- New `egui_mobius_reactive` crate for thread-safe reactive state management
- Comprehensive documentation for reactive system

### Changed
- Updated reactive example to use `Value::set()` for proper change notification
- Improved example documentation with clearer usage instructions
- Removed duplicate reactive.rs from core crate (now in egui_mobius_reactive)
- Added comprehensive module-level documentation for factory.rs
- Updated lib.rs documentation to reflect reactive system move to egui_mobius_reactive crate
- Enhanced core documentation with real-world examples:
  - Added thread-safe state sharing pattern with signal-slot example
  - Added comprehensive AsyncDispatcher usage pattern

### Fixed
- Fixed doubled value not updating in reactive example by using proper update method

## [0.3.0-alpha.14] - 2025-03-20

### Added
- example "reactive":
  - Reactive example showcasing context and method chaining

### Changed
- Renamed `ReactiveCtx` to `SignalRegistry` for better clarity
- Added `Default` implementation for `SignalRegistry`
- Enhanced reactive example documentation with detailed threaded runtime info

### Fixed
- Fixed ownership issues in reactive system doc tests

## [0.3.0-alpha.13] - 2025-03-20

### Added
- example "reactive":
  - Reactive example showcasing context and method chaining

### Changed

### Fixed

## [0.3.0-alpha.12] - 2025-03-20

### Added
- example/clock_async: 
  - Buffer size control in control panel
  - VecDeque for log storage (replaces Vec)
  - LoggerPanel::new with buffer size parameter

### Changed

### Fixed

## [0.3.0-alpha.11] - 2025-03-19

### Added
- New subscriber example showcasing signal-slot pattern with real-time data visualization
- IDE settings to .gitignore

### Changed
- Removed edition2024 feature flag (now stabilized in Rust 1.85)
- Improved code quality in subscriber example:
  - Added Default implementations
  - Fixed clippy warnings
  - Removed unused imports

## [0.3.0-alpha.10] - 2025-03-18

### Changed
- Renamed "Green Event" button to "Custom Event 2"
- Unified Custom Event buttons to use same logging color
- Added RUN/STOP button logging color selection

## [0.3.0-alpha.9] - 2025-03-18

### Changed
- Consolidated dispatching functionality in `dispatching` module
- Removed old Dispatcher implementation from factory.rs
- Enhanced documentation for signal-slot creation

## [0.3.0-alpha.8] - 2025-03-18

### Added
- Text color customization for StyledButton
- Default implementation for StyledButton

### Changed
- Simplified widget test framework by removing egui_kittest dependency
- Removed custom winit crate in favor of standard implementation

### Removed
- Integration test files
- Custom winit crate and egui_kittest dependencies

## [0.3.0-alpha.7] - 2025-03-18

### Added
- New ButtonColors struct to separate button appearance from log colors
- Unified RUN/STOP log color in LogColors
- Default implementation for StatefulButton

### Changed
- Improved UI responsiveness with better lock management
- Enhanced button layout and padding
- Separated button appearance colors from log colors
- Removed deprecated derive macros in favor of Signal/Slot pattern

### Fixed
- Lock contention issues in clock_async example
- Button color synchronization between UI and logs

### Removed
- Deprecated derive macros and associated code
- `macros.rs` module

## [0.3.0-alpha.6] - 2025-03-18

### Changed
- Simplified Signal/Slot API by removing channel number requirement
- Improved ergonomics of create_signal_slot function
- Removed deprecated as_command_derive dependency
- Updated all examples to use new Signal/Slot API

## [0.3.0-alpha.5] - 2025-03-18

### Changed
- Formally deprecated `AsCommand` macro with warning messages
- Enhanced documentation for Signal/Slot pattern as the recommended approach
- Removed deprecated `as_command_derive` crate

## [0.3.0-alpha.4] - 2025-03-18

### Changed
- Updated to egui 0.31.1 and egui_plot 0.31.0
- Standardized workspace dependencies for egui ecosystem crates
- Deprecated `AsCommand` macro in favor of Signal/Slot pattern with AsyncDispatcher

### Fixed
- Version mismatches between egui and egui_taffy
- Color imports in realtime_plot example

## [0.3.0-alpha.3] - 2025-03-17

### Added
- New clock_async example showcasing:
  - Thread-aware slot system with background clock generation
  - Comprehensive event logging with proper thread separation
  - Taffy layout integration for responsive UI
  - Interactive controls (slider and combo box)

### Changed
- Updated to egui 0.30.0
- Added Taffy layout support
- Improved README with detailed example documentation

### Fixed
- Logger panel layout in clock_async example
- Version mismatch in realtime_plot example

## [0.3.0-alpha.2] - 2025-03-16

### Added
- Initial support for async operations
- Enhanced thread safety features

### Changed
- Improved signal-slot system
- Better error handling

All notable changes to egui_mobius will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0-alpha.1] - 2025-03-16

### Added
- Thread-aware slot system with true hybrid sync/async operation
- Type-safe message passing between UI and background threads
- Qt-inspired signal-slot architecture
- Value<T> for thread-safe state management
- Support for background operations with clean thread separation
- AsyncDispatcher with Tokio runtime integration
- Comprehensive example suite demonstrating core features

### Changed
- Evolved to a complete application development platform
- Enhanced state management with automatic UI updates
- Improved thread safety and message ordering

### Deprecated
- Legacy examples moved to `deprecated` branch

## [0.2.0-alpha] - Previous Development
- Internal development version
- Added initial async support
- Enhanced dispatcher system

## [0.1.0] - Initial Development
- First internal development version
- Basic signal-slot implementation
- Initial UI integration with egui

## Upcoming Features (Planned)
- Enhanced logging and debugging capabilities
- Additional widget templates
- Performance optimizations
- Extended documentation and tutorials

## Version Strategy
- **0.3.x**: Initial public releases
  - 0.3.0-alpha.x: Pre-release versions
  - 0.3.0: First stable release
  - 0.3.x: Bug fixes and minor improvements
- **0.4.0+**: Feature additions
- **1.0.0**: Production-ready release
