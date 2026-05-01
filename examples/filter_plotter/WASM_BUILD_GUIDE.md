# WASM Build Guide for filter_plotter

This guide shows you how to build and run `filter_plotter` as a web application.

## Prerequisites

### 1. Install Trunk
Trunk is a WASM web application bundler for Rust:
```powershell
cargo install trunk
```

### 2. Install wasm-bindgen-cli
This tool generates JavaScript bindings for Rust/WASM:
```powershell
cargo install wasm-bindgen-cli
```

### 3. Add WASM Target
Add the WebAssembly compilation target to your Rust toolchain:
```powershell
rustup target add wasm32-unknown-unknown
```

## Building and Running

### Development Server

From the `examples/filter_plotter` directory:

```powershell
# Serve with live reload on http://127.0.0.1:8080
trunk serve --open
```

This will:
- Build your Rust code to WASM
- Generate JavaScript bindings
- Serve the app locally
- Automatically rebuild on file changes
- Open your default browser

### Production Build

```powershell
# Build optimized release version
trunk build --release
```

Output will be in the `dist/` directory:
```
dist/
├── index.html
├── filter_plotter.js
├── filter_plotter_bg.wasm
└── snippets/ (helper JS files)
```

## Deployment

After building with `trunk build --release`, you can deploy the `dist/` directory to any static web hosting:

### GitHub Pages
1. Copy `dist/` contents to your GitHub Pages repository
2. Push to GitHub

### Netlify/Vercel
1. Drag and drop the `dist/` folder to their web interface

### Self-hosted
1. Copy `dist/` contents to your web server's public directory
2. Ensure MIME types are configured:
   - `.wasm` → `application/wasm`
   - `.js` → `application/javascript`

## Troubleshooting

### Build Errors

**"error: failed to download wasm-bindgen..."**
- Solution: Update wasm-bindgen-cli to match the version in Cargo.lock:
  ```powershell
  cargo install wasm-bindgen-cli --version 0.2.x
  ```

**"error: cannot find macro env! in this scope"**
- Solution: Clean build artifacts:
  ```powershell
  cargo clean
  trunk clean
  trunk build
  ```

### Runtime Errors

**Blank page in browser**
- Check browser console (F12) for errors
- Ensure JavaScript is enabled
- Try a different browser (Chrome, Firefox, Edge)

**WASM module fails to load**
- Check that the server is serving `.wasm` files with correct MIME type
- Some corporate firewalls block `.wasm` files

## Performance Tips

### Optimize Build Size
```powershell
# Add to Cargo.toml for smaller WASM binaries
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
panic = 'abort'     # Smaller binary
strip = true        # Remove debug symbols
```

### Monitor Build Size
```powershell
# Check the size of generated WASM
dir dist\*.wasm
```

Typical sizes:
- Debug build: ~5-10 MB
- Release build: ~1-3 MB (with optimizations)

## Differences from Native Build

| Feature | Native | WASM |
|---------|--------|------|
| Threading | ✅ Multi-threaded | ⚠️ Single-threaded |
| File I/O | ✅ Direct access | ❌ Browser APIs only |
| Performance | ✅ Faster | ⚠️ ~70-80% of native |
| Startup | ✅ Instant | ⚠️ Download + compile |
| Distribution | 📦 Executable | 🌐 Web page |

## Configuration

### Customize Port
Edit `Trunk.toml`:
```toml
[serve]
port = 3000
```

### Enable Hot Reload
Already enabled by default with `trunk serve`

### Add Assets
Place files in the same directory as `index.html` and reference them:
```html
<link rel="icon" href="favicon.ico">
```

## Next Steps

- **Deploy your app**: Follow the deployment section above
- **Add more features**: Modify the Rust code and rebuild
- **Optimize performance**: Use the performance tips section
- **Share your work**: The WASM build is easy to share via URL!

## Resources

- [Trunk Documentation](https://trunkrs.dev/)
- [eframe WASM Guide](https://github.com/emilk/eframe_template)
- [Rust WASM Book](https://rustwasm.github.io/book/)
- [egui Web Demo](https://www.egui.rs/#demo)
