// Canvas background + procedural grid shader.
//
// A single fullscreen triangle, scissored to the canvas rect. The
// fragment shader fills the background and computes the grid per-pixel
// from the viewport transform — no grid geometry is uploaded, and the
// grid stays crisp at any zoom.

// MUST match `nodes.wgsl` and the `ViewportUniform` struct in `mod.rs`.
struct Viewport {
    // Screen point (egui points) where world (0, 0) lands, and
    // pixels-per-world-unit.
    origin: vec2<f32>,
    zoom: f32,
    pixels_per_point: f32,
    // Canvas background, linear premultiplied.
    bg_color: vec4<f32>,
    // Grid ink, linear, alpha = 1 (the shader applies the tier alpha).
    grid_color: vec4<f32>,
    // Canvas rect top-left, egui points (used by the node shader).
    canvas_min: vec2<f32>,
    grid_spacing: f32,   // world units between grid lines
    dot_size: f32,       // dot diameter, world units (dot style)
    grid_style: u32,     // 0 = lines, 1 = dots
    flags: u32,          // bit0 = show_grid, bit1 = srgb_target
    canvas_size: vec2<f32>,  // canvas rect size, egui points
};

@group(0) @binding(0) var<uniform> vp: Viewport;

const FLAG_SHOW_GRID: u32 = 1u;
const FLAG_SRGB_TARGET: u32 = 2u;

const STYLE_LINES: u32 = 0u;
const STYLE_DOTS: u32 = 1u;

// Tier alphas, mirrored from `render::paint_grid` (n / 255).
const LINE_MINOR_A: f32 = 0.10196;   //  26
const LINE_MAJOR_A: f32 = 0.23529;   //  60
const DOT_MINOR_A: f32 = 0.35294;    //  90
const DOT_MAJOR_A: f32 = 0.58824;    // 150

// Fullscreen triangle — egui's scissor rect clips it to the canvas.
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

// Major every 5th line, anchored at world 0 — matches the CPU renderer.
fn is_major(index: f32) -> bool {
    return (i32(index) % 5) == 0;
}

// Premultiplied "over" composite of `ink` at coverage `a` onto `dst`.
fn composite(dst: vec4<f32>, ink: vec3<f32>, a: f32) -> vec4<f32> {
    return vec4<f32>(ink * a, a) + dst * (1.0 - a);
}

// Coverage of the nearest grid line on one axis, plus its tier alpha.
// `w` is the world coordinate; distance is measured in egui points so a
// line is ~1 point wide with antialiased edges, like the CPU stroke.
fn axis_line(w: f32) -> vec2<f32> {
    let g = w / vp.grid_spacing;
    let nearest = round(g);
    let dist_points = abs(g - nearest) * vp.grid_spacing * vp.zoom;
    let coverage = 1.0 - smoothstep(0.0, 1.0, dist_points);
    let tier = select(LINE_MINOR_A, LINE_MAJOR_A, is_major(nearest));
    return vec2<f32>(coverage, tier);
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    var color = vp.bg_color;

    let show_grid = (vp.flags & FLAG_SHOW_GRID) != 0u;
    if (show_grid && vp.grid_spacing > 0.0) {
        // Physical pixel -> egui point -> world coordinate.
        let point = frag.xy / vp.pixels_per_point;
        let world = (point - vp.origin) / vp.zoom;
        let ink = vp.grid_color.rgb;

        if (vp.grid_style == STYLE_DOTS) {
            let g = world / vp.grid_spacing;
            let nearest = round(g);
            let d = (g - nearest) * vp.grid_spacing * vp.zoom; // points
            let dist = length(d);
            let radius = clamp(vp.dot_size * vp.zoom * 0.5, 0.6, 6.0);
            let coverage = 1.0 - smoothstep(radius - 0.75, radius + 0.75, dist);
            let major = is_major(nearest.x) && is_major(nearest.y);
            let tier = select(DOT_MINOR_A, DOT_MAJOR_A, major);
            color = composite(color, ink, tier * coverage);
        } else {
            // Lines: composite each axis. Minor first so a major line
            // reads cleanly where the two cross.
            let lx = axis_line(world.x);
            let ly = axis_line(world.y);
            if (lx.y < ly.y) {
                color = composite(color, ink, lx.x * lx.y);
                color = composite(color, ink, ly.x * ly.y);
            } else {
                color = composite(color, ink, ly.x * ly.y);
                color = composite(color, ink, lx.x * lx.y);
            }
        }
    }

    // An sRGB-format target converts linear -> sRGB on store; a UNORM
    // target does not, so convert in-shader for that case.
    if ((vp.flags & FLAG_SRGB_TARGET) != 0u) {
        return color;
    }
    return vec4<f32>(linear_to_srgb(color.rgb), color.a);
}
