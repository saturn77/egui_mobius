// Canvas background / grid shader.
//
// Phase 0 draws a flat background fill across the canvas rect. The
// `ViewportUniform` already carries the grid parameters so Phase 1 can
// extend `fs_main` to mix a procedural grid in without touching Rust.

struct Viewport {
    // Screen point where world (0, 0) lands, and pixels-per-world-unit.
    origin: vec2<f32>,
    zoom: f32,
    pixels_per_point: f32,
    // Canvas background, linear premultiplied.
    bg_color: vec4<f32>,
    // Grid ink, linear premultiplied (Phase 1).
    grid_color: vec4<f32>,
    grid_spacing: f32,
    dot_size: f32,
    grid_style: u32,   // 0 = lines, 1 = dots
    flags: u32,        // bit0 = show_grid, bit1 = srgb_target
};

@group(0) @binding(0) var<uniform> vp: Viewport;

const FLAG_SHOW_GRID: u32 = 1u;
const FLAG_SRGB_TARGET: u32 = 2u;

// Fullscreen triangle — covers clip space with three vertices; egui's
// scissor rect clips it down to the canvas rect.
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((idx << 1u) & 2u), f32(idx & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    var color = vp.bg_color;

    // Phase 1 will mix the procedural grid into `color` here, using
    // `frag.xy`, `vp.origin`, `vp.zoom`, and the grid parameters.

    // An sRGB-format target converts linear -> sRGB on store; a UNORM
    // target does not, so we convert in-shader for that case.
    if ((vp.flags & FLAG_SRGB_TARGET) != 0u) {
        return color;
    }
    return vec4<f32>(linear_to_srgb(color.rgb), color.a);
}
