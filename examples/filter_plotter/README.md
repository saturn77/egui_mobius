# Filter Plotter

A citizen-pattern tutorial app demonstrating IIR filtering with real-time plotting.

Three panels (Plot / Settings / Terminal) wired into `egui_dock` via a `TabViewer`, with `egui_citizen::Dispatcher` as the message hub between the settings panel and an in-process IIR filter backend.

## Features

- **Plot Panel**: Real-time visualization of filter response
- **Settings Panel**: Configure filter parameters and generate traces
- **Logger Panel**: View application events and messages
- Citizen pattern for clean panel lifecycle management
- In-process IIR filter backend

## Building and Running

### Native (Desktop)

```bash
# From the workspace root
cargo run -p filter_plotter

# Or from this directory
cargo run
```

### WASM (Web Browser)

#### Prerequisites

Install Trunk (WASM build tool):
```bash
cargo install trunk wasm-bindgen-cli
```

Add the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

#### Build and Serve

```bash
# From this directory (examples/filter_plotter)
trunk serve --open

# Or build for production
trunk build --release
```

The app will be available at `http://127.0.0.1:8080`

#### Deploy

After building with `trunk build --release`, the contents of the `dist/` directory can be deployed to any static web hosting:

```
dist/
├── index.html
├── filter_plotter.js
├── filter_plotter_bg.wasm
└── ... (other assets)
```

## Architecture

This example demonstrates:
- Conditional compilation for native vs WASM targets
- Clean separation of concerns using the citizen pattern
- Message-based communication between panels
- Reactive state management with `egui_mobius_reactive`
- Cross-platform compatibility (desktop and web)

## Code Structure

```
src/
├── main.rs          - Entry point with platform-specific initialization
├── backend/         - IIR filter implementation
├── dispatcher.rs    - Message routing and handling
├── messages.rs      - Application message types
├── panels/          - UI panel implementations
│   ├── logger.rs
│   ├── plot.rs
│   └── settings.rs
├── state.rs         - Shared application state
├── tabs.rs          - Dock tab management
└── theme.rs         - Visual styling
```

## Notes

- **WASM Limitations**: The WASM build runs single-threaded in the browser's event loop
- **Performance**: Native builds may perform better for computationally intensive operations
- **Storage**: WASM has no direct file system access; would require browser storage APIs for persistence
