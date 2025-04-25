# Versioning Guide for egui_mobius

## Version Strategy

`egui_mobius` follows [Semantic Versioning](https://semver.org/) with the following structure:
`MAJOR.MINOR.PATCH[-PRERELEASE]`

## Development Phases

### 1. **Alpha Phase (0.3.0-alpha.x)**
   - Initial public releases with **core architecture** in place
   - API may undergo refinements and breaking changes
   - Focus on gathering community feedback and testing
   - Each alpha release adds new features or significant improvements

### 2. **Stable Release (0.3.0)**
   - First **stable public release**
   - API considered stable and production-ready
   - Comprehensive documentation and examples
   - Focus on performance optimizations and bug fixes

### 3. **Patch Updates (0.3.x)**
   - Bug fixes and minor improvements
   - Documentation updates and additional examples
   - No breaking changes

### 4. **Minor Versions (0.4.0+)**
   - Introduction of new features and enhancements
   - Backwards-compatible changes
   - Expanded support for advanced use cases (e.g., dynamic layouts, async workflows)

### 5. **Major Version (1.0.0)**
   - Production-proven API with a complete feature set
   - Comprehensive testing and multiple production use cases
   - Long-term support and stability guarantees

---

## Dependencies & Compatibility

### egui/eframe Compatibility

`egui_mobius` is always tested against specific versions of `egui` and related crates. For the specific compatibility matrix of each release, refer to the repository README or the release notes.

### Workspace Structure

The `egui_mobius` ecosystem consists of multiple crates that maintain version parity:
- `egui_mobius`: Core signal-slot framework
- `egui_mobius_widgets`: Reusable UI widgets
- `egui_mobius_reactive`: Thread-safe reactive state management
- `egui_mobius_components`: Higher-level UI components

All versions within the workspace are updated simultaneously to ensure compatibility.

---

## Breaking Changes Policy

### During Alpha Phase
- Breaking changes are possible with **minor version increments**
- All changes are documented in `CHANGELOG.md`
- Migration guides are provided for significant changes

### After Stable Release
- Breaking changes are introduced only in **major versions**
- Deprecation notices are provided in **minor versions**
- Minimum **two minor versions notice** before removal of deprecated features

---

## Version Support

| Version Phase     | Support Level       |
|-------------------|---------------------|
| Latest alpha/stable | Full support      |
| Previous minor    | Bug fixes only      |
| Older versions    | Security fixes only |

---

## Release Process

### 1. **Pre-release Checklist**
   - All tests passing
   - Examples updated and working
   - Documentation current and reviewed
   - `CHANGELOG.md` updated with all changes
   - Version numbers synchronized across crates

### 2. **Release Steps**
   - Update version numbers in workspace `Cargo.toml`
   - Create a git tag for the release
   - Publish crates to `crates.io`
   - Update documentation website

### 3. **Post-release**
   - Verify the release on `crates.io`
   - Update badges and links in the repository
   - Announce the release on community channels

---

## Feature Development

New features are developed according to these principles:

1. **Modularity**: Components should be reusable and composable
2. **Type Safety**: Leverage Rust's type system for correctness
3. **Thread Safety**: All components must work correctly in multi-threaded environments
4. **Documentation**: New features include comprehensive examples and documentation
5. **Testing**: Unit and integration tests cover new functionality

---

## Future Roadmap

For the current development roadmap, refer to:
- [GitHub Projects](https://github.com/your-org/egui_mobius/projects)
- [Milestone tracking](https://github.com/your-org/egui_mobius/milestones)
- Release discussions in the repository

The high-level roadmap progresses through these stages:
1. Core architectural components (signals, slots, dispatchers)
2. Reactive state management
3. Component library and patterns
4. Advanced layouts and UI patterns
5. Production optimizations and polish

---

## Community Involvement

We encourage the community to contribute by:
- Reporting bugs and submitting feature requests
- Providing feedback on the API and examples
- Contributing to documentation and examples

Join the discussion on our [GitHub Discussions](https://github.com/your-org/egui_mobius/discussions).

---

## Version Information

For the current version and latest changes, see:
- [CHANGELOG.md](./CHANGELOG.md) - Detailed listing of all changes by version
- [Cargo.toml](./Cargo.toml) - Current version number in the workspace
- [Release Tags](https://github.com/your-org/egui_mobius/releases) - Official releases with notes