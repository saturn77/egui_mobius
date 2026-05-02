# filter_plotter

Citizen-pattern tutorial example: a biquad IIR filter backend driven
by a settings panel, with results plotted in a second panel and
events streamed to a logger panel. All three panels are docked via
`egui_dock` and coordinated through `egui_citizen::Dispatcher`.

The book walks through this example file by file.

## Run native

```bash
cargo run -p filter_plotter
```

## Run in the browser (WASM)

One-time setup:

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

Then:

```bash
cd examples/filter_plotter
trunk serve --open       # development, opens http://127.0.0.1:8080
trunk build --release    # production, output in ./dist/
```

The `dist/` directory after a release build is a self-contained
static site — drop it on any web host.

Windows users: `build_wasm.ps1` wraps the trunk commands and checks
prerequisites. Linux/macOS just use `trunk` directly.

## Notes

- WASM builds run single-threaded in the browser event loop; the
  filter backend runs in the same JS task as the UI.
- No file system access in WASM — persistence would need browser
  storage APIs.
