# Versioning Guide for egui_mobius

## Version Strategy

egui_mobius follows [Semantic Versioning](https://semver.org/) with the following structure:
`MAJOR.MINOR.PATCH[-PRERELEASE]`

### Current Phase: 0.3.0-alpha.15

We are currently in the alpha phase of our first public release. This version represents:
- A mature architectural foundation with Signal/Slot pattern
- Production-ready core features and async support
- Ongoing API refinements and ergonomic improvements
- Full compatibility with egui 0.31.1 and egui_taffy 0.7.0

### Version Progression

1. **Alpha Phase (0.3.0-alpha.x)**
   - Current stage
   - API may undergo refinements
   - Core architecture is stable
   - Gathering community feedback

2. **Stable Release (0.3.0)**
   - First stable public release
   - API considered stable
   - Production-ready features
   - Comprehensive documentation

3. **Patch Updates (0.3.x)**
   - Bug fixes
   - Documentation improvements
   - Performance optimizations
   - No breaking changes

4. **Minor Versions (0.4.0+)**
   - New features
   - Backwards compatible changes
   - Enhanced capabilities
   - Additional examples

5. **Major Version (1.0.0)**
   - Production-proven API
   - Complete feature set
   - Comprehensive testing
   - Multiple production use cases

## Compatibility Matrix

### egui/eframe Compatibility

| egui_mobius Version | egui/eframe Version | Rust Edition | Status      |
|-------------------|-------------------|--------------|-------------|
| 0.3.0-alpha.7     | 0.31.1            | 2024         | Current     |
| 0.3.0-alpha.6     | 0.31.1            | 2024         | Deprecated  |
| 0.3.0-alpha.5     | 0.31.1            | 2024         | Deprecated  |
| 0.3.0-alpha.4     | 0.31.1            | 2024         | Deprecated  |
| 0.3.0-alpha.3     | 0.30.0            | 2024         | Deprecated  |
| 0.3.0-alpha.2     | 0.30.0            | 2024         | Deprecated  |
| 0.2.0-alpha        | 0.24.0             | 2024         | Deprecated  |
| 0.1.0              | 0.22.0             | 2021         | Deprecated  |

### Crate Dependencies

All egui_mobius crates maintain version parity:
- egui_mobius: 0.3.0-alpha.7
- egui_mobius_widgets: 0.3.0-alpha.7
- egui_mobius_macros: 0.3.0-alpha.7 (Signal/Slot pattern only)

## Breaking Changes Policy

### During Alpha (0.3.0-alpha.x)
- Breaking changes possible with minor version bumps
- All changes documented in CHANGELOG.md
- Migration guides provided for significant changes

### After Stable (0.3.0+)
- Breaking changes only in major versions
- Deprecation notices in minor versions
- Minimum 2 minor versions notice before removal

## Version Support

- Latest stable version: Full support
- Previous minor version: Bug fixes only
- Older versions: Security fixes only

## Release Process

1. **Pre-release Checklist**
   - All tests passing
   - Examples updated
   - Documentation current
   - CHANGELOG.md updated
   - Version numbers synchronized

2. **Release Steps**
   - Update version numbers
   - Create git tag
   - Publish to crates.io
   - Update documentation

3. **Post-release**
   - Verify crates.io publication
   - Update badges
   - Announce release
