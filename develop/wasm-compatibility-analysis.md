# WASM Compatibility Analysis for egui_mobius

## Executive Summary

Converting egui_mobius to run in WASM is **feasible but requires moderate refactoring**. The main blocker is the use of `std::thread` and `tokio` with full threading features. The best candidate for WASM conversion is **`filter_plotter`** since it doesn't use async features.

## Key Blockers

### 1. **Threading (CRITICAL BLOCKER)**
- **Location**: `@c:\dev\egui_mobius\crates\egui_mobius\src\slot.rs:60` (line 60)
- **Issue**: Uses `std::thread::spawn` in `Slot::start()`
- **Impact**: WASM doesn't support native threads
- **Fix**: Replace with WASM-compatible alternatives or make thread spawning conditional

### 2. **Tokio Runtime (CRITICAL BLOCKER)**
- **Location**: 
  - `@c:\dev\egui_mobius\crates\egui_mobius\Cargo.toml:25` - `tokio = { features = ["full"] }`
  - `@c:\dev\egui_mobius\crates\egui_mobius\src\slot.rs:75` - `tokio::spawn`
  - `@c:\dev\egui_mobius\crates\egui_mobius\src\runtime.rs:104`
- **Issue**: Tokio's "full" features include thread pools, file I/O, and process management
- **Impact**: Not compatible with WASM
- **Fix**: Use `wasm-bindgen-futures` and remove tokio, or use conditional compilation

### 3. **File System Access (BLOCKER FOR SOME EXAMPLES)**
- **Location**: `@c:\dev\egui_mobius\examples\clock_async\src\main.rs:123-153`
- **Issue**: Uses `std::fs` for config file operations
- **Impact**: WASM has no file system access (requires browser APIs)
- **Fix**: Use `web-sys` local storage or remove persistence in WASM builds

### 4. **Channel Types**
- **Location**: `@c:\dev\egui_mobius\crates\egui_mobius\src\slot.rs:14` - `std::sync::mpsc`
- **Issue**: `std::sync::mpsc` uses blocking operations
- **Impact**: May work but not optimal for WASM
- **Fix**: Consider using `futures::channel::mpsc` or `async-channel`

## What Works Already ✅

1. **egui_mobius_reactive** - No threading, uses `parking_lot` which has WASM support
2. **egui_citizen** - Only depends on reactive, no threading
3. **egui/eframe** - Already WASM-compatible with proper feature flags
4. **Core architecture** - The signal/slot pattern is WASM-friendly

## Recommended Path Forward

### Option A: Convert `filter_plotter` (EASIEST)

**Why this is best:**
- No async/threading currently used
- No file I/O
- Simple in-process backend
- Already uses `egui_citizen` which is WASM-ready

**Required changes:**
1. Add WASM target configuration
2. Make `eframe` features conditional:
   ```toml
   [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
   eframe = { workspace = true, features = ["default"] }
   
   [target.'cfg(target_arch = "wasm32")'.dependencies]
   eframe = { workspace = true, features = ["wasm_web"] }
   wasm-bindgen = "0.2"
   ```
3. Add trunk/wasm-pack build configuration
4. **Critical**: Remove or make optional any `Slot::start()` calls (threaded slots)

### Option B: Add WASM Support to Core Library (COMPREHENSIVE)

**Changes needed in `egui_mobius`:**

1. **Make threading optional with feature flags:**
   ```toml
   [features]
   default = ["threading"]
   threading = []
   wasm = ["wasm-bindgen-futures"]
   ```

2. **Conditional compilation in `slot.rs`:**
   ```rust
   #[cfg(feature = "threading")]
   pub fn start<F>(&mut self, mut handler: F)
   where F: FnMut(T) + Send + 'static
   {
       let receiver = Arc::clone(&self.receiver);
       std::thread::spawn(move || {
           // existing code
       });
   }
   
   #[cfg(not(feature = "threading"))]
   pub fn start<F>(&mut self, mut handler: F)
   where F: FnMut(T) + 'static  // Remove Send bound for WASM
   {
       compile_error!("Threaded slots not supported in WASM. Use start_wasm() instead.");
   }
   ```

3. **Replace tokio with wasm-compatible async:**
   - Use `wasm-bindgen-futures` for spawning
   - Use `futures::channel::mpsc` instead of `std::sync::mpsc`
   - Add conditional compilation for async runtime

4. **Update workspace Cargo.toml:**
   ```toml
   [target.'cfg(target_arch = "wasm32")'.dependencies]
   tokio = { workspace = true, features = ["sync", "macros"] }  # Remove "full"
   wasm-bindgen = "0.2"
   wasm-bindgen-futures = "0.4"
   web-sys = "0.3"
   ```

## Example Rankings for WASM Conversion

| Example | Difficulty | Blockers | Recommendation |
|---------|-----------|----------|----------------|
| **filter_plotter** | ⭐ Easy | None | **BEST CHOICE** |
| getting_started | ⭐⭐ Medium | May use threading | Good if simple |
| reactive | ⭐⭐ Medium | Depends on features | Doable |
| clock_reactive | ⭐⭐⭐ Medium | Reactive only | Possible |
| clock_async | ⭐⭐⭐⭐ Hard | Tokio, threads, file I/O | Challenging |
| citizen_signal_async | ⭐⭐⭐⭐ Hard | Tokio, async dispatcher | Challenging |
| dashboard_async | ⭐⭐⭐⭐ Hard | Tokio, async | Challenging |

## Concrete Implementation Plan for `filter_plotter`

### Step 1: Add WASM build configuration
Create `examples/filter_plotter/index.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Filter Plotter</title>
    <style>
        body { margin: 0; padding: 0; }
        canvas { width: 100%; height: 100vh; }
    </style>
</head>
<body>
    <script type="module">
        import init from './filter_plotter.js';
        init();
    </script>
</body>
</html>
```

### Step 2: Add Trunk.toml
Create `examples/filter_plotter/Trunk.toml`:
```toml
[build]
target = "index.html"
release = false
```

### Step 3: Update Cargo.toml
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
eframe = { workspace = true, features = ["default"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
eframe = { workspace = true, features = ["wasm_web"] }
wasm-bindgen = "0.2"
web-sys = "0.3"
```

### Step 4: Conditional main function
```rust
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // existing native code
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    
    let web_options = eframe::WebOptions::default();
    
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "filter_plotter",
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await
            .expect("Failed to start eframe");
    });
}
```

### Step 5: Build and test
```bash
# Install prerequisites
cargo install trunk wasm-bindgen-cli

# Build for WASM
cd examples/filter_plotter
trunk build --release

# Serve locally
trunk serve --open
```

## Dependencies That Need Attention

| Crate | WASM Support | Notes |
|-------|--------------|-------|
| egui | ✅ Full | Built-in WASM support |
| eframe | ✅ Full | Use "wasm_web" feature |
| egui_dock | ✅ Yes | No special changes needed |
| egui_plot | ✅ Yes | WASM-compatible |
| tokio | ⚠️ Partial | Must avoid "full" features |
| chrono | ✅ Yes | Works with "wasmbind" feature |
| serde/serde_json | ✅ Yes | Fully compatible |
| parking_lot | ✅ Yes | Has WASM support |

## Performance Considerations

1. **Memory**: WASM has 32-bit address space (4GB limit)
2. **Single-threaded**: All async runs on main thread via event loop
3. **No SharedArrayBuffer**: Can't share memory between workers easily
4. **Startup time**: WASM binary needs to be downloaded and compiled

## Testing Strategy

1. Start with `filter_plotter` (no async/threading)
2. Verify all egui_citizen features work
3. Test dock panel operations
4. Verify plot rendering
5. Check performance with larger datasets
6. Once working, document pattern for other examples

## Conclusion

**TL;DR: `filter_plotter` can be converted to WASM with ~2-4 hours of work**. The async examples require more substantial refactoring of the core library to support conditional compilation for WASM vs native threading.

The architecture is fundamentally WASM-compatible; the main work is:
1. Removing/conditionalizing thread spawning
2. Replacing tokio "full" with WASM-friendly alternatives
3. Adding proper build configuration
4. Testing and documentation
