# Changelog

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
