# Changelog

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
