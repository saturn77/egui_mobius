# Changelog

## [0.3.0-alpha.4] - 2025-03-18

### Added

### Changed

### Fixed

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

### Changed

### Fixed

## [0.3.0-alpha.2] - 2025-03-16

### Added

### Changed

### Fixed

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
