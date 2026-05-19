// Instanced edge-segment shader.
//
// One instance per polyline segment. A unit quad is expanded in the
// vertex shader to an oriented rectangle along the segment, widened
// perpendicular by the stroke width plus an antialiasing margin. The
// fragment shader applies across-the-width AA and the dash / dot
// pattern, mirroring `render::paint_line`.
//
// The `Viewport` struct MUST match `canvas.wgsl` / `nodes.wgsl`.

struct Viewport {
    origin: vec2<f32>,
    zoom: f32,
    pixels_per_point: f32,
    bg_color: vec4<f32>,
    grid_color: vec4<f32>,
    canvas_min: vec2<f32>,
    grid_spacing: f32,
    dot_size: f32,
    grid_style: u32,
    flags: u32,
    canvas_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> vp: Viewport;

const FLAG_SRGB_TARGET: u32 = 2u;
const STYLE_SOLID: u32 = 0u;
const STYLE_DASHED: u32 = 1u;
const STYLE_DOTTED: u32 = 2u;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Signed perpendicular distance from the segment centerline, points.
    @location(0) across: f32,
    // Distance along the segment from its start, points.
    @location(1) along: f32,
    @location(2) @interpolate(flat) half_width: f32,
    @location(3) @interpolate(flat) color: vec4<f32>,
    @location(4) @interpolate(flat) style: u32,
};

// egui-wgpu maps clip space to the callback's canvas rect.
fn world_to_ndc(world: vec2<f32>) -> vec2<f32> {
    let sp = vp.origin + world * vp.zoom;
    let rel = (sp - vp.canvas_min) / vp.canvas_size;
    let ndc = rel * 2.0 - vec2<f32>(1.0);
    return vec2<f32>(ndc.x, -ndc.y);
}

@vertex
fn vs_edge(
    @builtin(vertex_index) vid: u32,
    @location(0) i_a: vec2<f32>,
    @location(1) i_b: vec2<f32>,
    @location(2) i_color: vec4<f32>,
    @location(3) i_width: f32,
    @location(4) i_style: u32,
) -> VsOut {
    // (t, side): t in {0,1} along the segment, side in {-1,+1} across.
    var ts = array<vec2<f32>, 6>(
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = ts[vid];

    let delta = i_b - i_a;
    let len = max(length(delta), 1e-6);
    let dir = delta / len;
    let normal = vec2<f32>(-dir.y, dir.x);

    let hw = i_width * 0.5;                       // world half-width
    // One extra point each side gives the AA ramp room to fall off.
    let aa_world = 1.0 / max(vp.zoom, 1e-6);
    let half_extent = hw + aa_world;

    let endpoint = i_a + delta * c.x;
    let world = endpoint + normal * (c.y * half_extent);

    var out: VsOut;
    out.clip = vec4<f32>(world_to_ndc(world), 0.0, 1.0);
    out.across = c.y * half_extent * vp.zoom;     // points
    out.along = c.x * len * vp.zoom;              // points
    out.half_width = hw * vp.zoom;
    out.color = i_color;
    out.style = i_style;
    return out;
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn emit(color: vec4<f32>) -> vec4<f32> {
    if ((vp.flags & FLAG_SRGB_TARGET) != 0u) {
        return color;
    }
    return vec4<f32>(linear_to_srgb(color.rgb), color.a);
}

// On-fraction of a dash period at distance `along`, with soft edges.
fn dash_on(along: f32, period: f32, on: f32) -> f32 {
    let phase = along - floor(along / period) * period;
    // Antialiased window [0, on] within the period.
    return smoothstep(-0.5, 0.5, phase) - smoothstep(on - 0.5, on + 0.5, phase);
}

@fragment
fn fs_edge(in: VsOut) -> @location(0) vec4<f32> {
    // Across-the-width antialiasing.
    var cov = 1.0 - smoothstep(in.half_width - 0.5, in.half_width + 0.5, abs(in.across));

    if (in.style == STYLE_DASHED) {
        cov = cov * dash_on(in.along, 12.0, 8.0);
    } else if (in.style == STYLE_DOTTED) {
        cov = cov * dash_on(in.along, 5.0, 2.0);
    }
    if (cov <= 0.0) {
        return vec4<f32>(0.0);
    }
    // Premultiplied — scaling the whole texel by coverage stays valid.
    return emit(in.color * cov);
}
