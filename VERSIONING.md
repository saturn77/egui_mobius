# Versioning Guide for egui_mobius

## Version Strategy

`egui_mobius` follows [Semantic Versioning](https://semver.org/) with the following structure:
`MAJOR.MINOR.PATCH[-PRERELEASE]`

### Current Phase: 0.3.0-alpha.29

We are currently in the **alpha phase** of our first public release. This version represents:
- An evolving, yet relatively **mature architectural foundation** with the `MobiusWidget` trait and related traits (`MobiusWidgetReactive`, `MobiusWidgetSlot`, `MobiusWidgetSignal`).
- **Production-ready core features** with async support and reactive state management.
- **Ongoing API refinements** to improve ergonomics and modularity.
- **Full compatibility** with `egui 0.31.1` and `egui_taffy 0.7.0`.

---

## Version Progression

### 1. **Alpha Phase (0.3.0-alpha.x)**
   - **Current stage** of development.
   - API may undergo refinements and breaking changes.
   - Core architecture is stable, focusing on gathering community feedback.
   - New examples like `reactive-widget-async` demonstrate advanced use cases.

### 2. **Stable Release (0.3.0)**
   - First **stable public release**.
   - API considered stable and production-ready.
   - Comprehensive documentation and examples.
   - Focus on performance optimizations and bug fixes.

### 3. **Patch Updates (0.3.x)**
   - Bug fixes and minor improvements.
   - Documentation updates and additional examples.
   - No breaking changes.

### 4. **Minor Versions (0.4.0+)**
   - Introduction of new features and enhancements.
   - Backwards-compatible changes.
   - Expanded support for advanced use cases (e.g., dynamic layouts, async workflows).

### 5. **Major Version (1.0.0)**
   - Production-proven API with a complete feature set.
   - Comprehensive testing and multiple production use cases.
   - Long-term support and stability guarantees.

---

## Compatibility Matrix

### egui/eframe Compatibility

| egui_mobius Version | egui/eframe Version | Rust Edition | Status      |
|---------------------|---------------------|--------------|-------------|
| 0.3.0-alpha.30      | 0.31.1             | 2024         | Current     |
| 0.3.0-alpha.29      | 0.31.1             | 2024         | Deprecated  |
| 0.3.0-alpha.28      | 0.31.1             | 2024         | Deprecated  |
| 0.3.0-alpha.27      | 0.30.0             | 2024         | Deprecated  |
| 0.2.0-alpha         | 0.24.0             | 2021         | Deprecated  |
| 0.1.0               | 0.22.0             | 2021         | Deprecated  |

### Crate Dependencies

All `egui_mobius` crates maintain version parity:
- `egui_mobius`: 0.3.0-alpha.30
- `egui_mobius_widgets`: 0.3.0-alpha.30
- `egui_mobius_macros`: 0.3.0-alpha.30 (Signal/Slot pattern only)

---

## Breaking Changes Policy

### During Alpha (0.3.0-alpha.x)
- Breaking changes are possible with **minor version bumps**.
- All changes are documented in `CHANGELOG.md`.
- Migration guides are provided for significant changes.

### After Stable (0.3.0+)
- Breaking changes are introduced only in **major versions**.
- Deprecation notices are provided in **minor versions**.
- Minimum **two minor versions notice** before removal of deprecated features.

---

## Version Support

| Version             | Support Level       |
|---------------------|---------------------|
| Latest stable       | Full support        |
| Previous minor      | Bug fixes only      |
| Older versions      | Security fixes only |

---

## Release Process

### 1. **Pre-release Checklist**
   - All tests passing.
   - Examples updated (e.g., `reactive-widget-async`).
   - Documentation current and reviewed.
   - `CHANGELOG.md` updated with all changes.
   - Version numbers synchronized across crates.

### 2. **Release Steps**
   - Update version numbers in `Cargo.toml`.
   - Create a git tag for the release.
   - Publish crates to `crates.io`.
   - Update documentation and examples.

### 3. **Post-release**
   - Verify the release on `crates.io`.
   - Update badges and links in the repository.
   - Announce the release on community channels.

---

## New Features in 0.3.0-alpha.30

### 1. **`MobiusWidget` Architecture**
   - Introduced the `MobiusWidget` trait for modular and reusable widget design.
   - Added support for reactive state binding with `MobiusWidgetReactive`.
   - Enabled dynamic layouts with `MobiusWidgetSlot`.
   - Event-driven communication via `MobiusWidgetSignal`.

### 2. **`reactive-widget-async` Example**
   - Demonstrates the use of `MobiusWidget` traits in a tabbed interface.
   - Features a `TerminalWidget` with reactive log updates and dynamic color schemes.
   - Integrates with `MobiusRuntime` for async workflows.

### 3. **Improved Async Support**
   - Enhanced integration with `tokio` for runtime management.
   - Reactive state updates in async contexts.

---

## Future Roadmap

### Short-term Goals (0.3.x)
- Expand examples to cover advanced use cases (e.g., dynamic layouts, async workflows).
- Improve performance and reduce runtime overhead.
- Gather community feedback to refine the API.

### Medium-term Goals (0.4.x)
- Add support for more complex widgets and layouts.
- Introduce a plugin system for extending functionality.
- Provide integration with external libraries (e.g., database, networking).

### Long-term Goals (1.0.0)
- Achieve production-grade stability and performance.
- Ensure compatibility with future versions of `egui` and `eframe`.
- Build a thriving community around `egui_mobius`.

---

## Community Involvement

We encourage the community to contribute by:
- Reporting bugs and submitting feature requests.
- Providing feedback on the API and examples.
- Contributing to documentation and examples.

Join the discussion on our [GitHub Discussions](https://github.com/your-repo/egui_mobius/discussions) or [Discord](https://discord.gg/your-invite-link).

---

## Conclusion

The `egui_mobius` library is evolving rapidly, with a focus on modularity, reactivity, and async support. The `0.3.0-alpha.30` release represents a significant milestone, laying the foundation for a stable and production-ready library. We look forward to your feedback and contributions as we move toward the first stable release.