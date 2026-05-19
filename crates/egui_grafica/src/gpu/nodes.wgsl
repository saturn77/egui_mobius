// Instanced node-body shader.
//
// One instance per scene node. A unit quad is expanded in the vertex
// shader to the node's world rect, then shaded as rect / circle /
// ellipse by a signed-distance field in the fragment shader. The border
// is an inside stroke, matching `render::paint_node`.
//
// The `Viewport` struct MUST match `canvas.wgsl` and the
// `ViewportUniform` struct in `mod.rs` — same fields, same order.

struct Viewport {
    origin: vec2<f32>,
    zoom: f32,
    pixels_per_point: f32,
    bg_color: vec4<f32>,
    grid_color: vec4<f32>,
    screen_size: vec2<f32>,
    grid_spacing: f32,
    dot_size: f32,
    grid_style: u32,
    flags: u32,
    pad0: u32,
    pad1: u32,
};

@group(0) @binding(0) var<uniform> vp: Viewport;

const KIND_RECT: u32 = 0u;
const KIND_CIRCLE: u32 = 1u;
const KIND_ELLIPSE: u32 = 2u;
const FLAG_SRGB_TARGET: u32 = 2u;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<f32>,
    @location(2) @interpolate(flat) fill: vec4<f32>,
    @location(3) @interpolate(flat) border: vec4<f32>,
    @location(4) @interpolate(flat) border_width: f32,
    @location(5) @interpolate(flat) kind: u32,
};

// World point -> normalized device coordinates. egui's render pass spans
// the whole surface, so clip space maps to the full window.
fn world_to_ndc(world: vec2<f32>) -> vec2<f32> {
    let sp = vp.origin + world * vp.zoom;            // egui points
    let ndc = sp / vp.screen_size * 2.0 - vec2<f32>(1.0);
    return vec2<f32>(ndc.x, -ndc.y);
}

@vertex
fn vs_node(
    @builtin(vertex_index) vid: u32,
    @location(0) i_pos: vec2<f32>,
    @location(1) i_size: vec2<f32>,
    @location(2) i_fill: vec4<f32>,
    @location(3) i_border: vec4<f32>,
    @location(4) i_border_width: f32,
    @location(5) i_kind: u32,
) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let corner = corners[vid];
    let world = i_pos + corner * i_size;

    var out: VsOut;
    out.clip = vec4<f32>(world_to_ndc(world), 0.0, 1.0);
    out.local = corner;
    out.size = i_size;
    out.fill = i_fill;
    out.border = i_border;
    out.border_width = i_border_width;
    out.kind = i_kind;
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

@fragment
fn fs_node(in: VsOut) -> @location(0) vec4<f32> {
    // Border stroke width, in egui points.
    let bw = max(in.border_width * vp.zoom, 0.0);

    if (in.kind == KIND_RECT) {
        // Distance inward to the nearest edge, in points.
        let edge = min(in.local, vec2<f32>(1.0) - in.local) * in.size * vp.zoom;
        let d = min(edge.x, edge.y);
        // d < bw -> border band, else fill.
        let t = smoothstep(bw - 0.5, bw + 0.5, d);
        return emit(mix(in.border, in.fill, t));
    }

    // Circle / ellipse: signed distance to the contour, in points.
    // Negative inside, zero on the edge.
    let p = (in.local - vec2<f32>(0.5)) * in.size;       // world offset from center
    var sd: f32;
    if (in.kind == KIND_CIRCLE) {
        let r = min(in.size.x, in.size.y) * 0.5;
        sd = (length(p) - r) * vp.zoom;
    } else {
        let r = in.size * 0.5;
        sd = (length(p / r) - 1.0) * min(r.x, r.y) * vp.zoom;
    }

    // Antialiased coverage at the outer edge.
    let coverage = 1.0 - smoothstep(-0.5, 0.5, sd);
    if (coverage <= 0.0) {
        return vec4<f32>(0.0);
    }
    // Inside: the border band is [-bw, 0]; deeper is fill.
    let t = smoothstep(-bw - 0.5, -bw + 0.5, sd);
    let color = mix(in.fill, in.border, t);
    // Premultiplied — scaling the whole texel by coverage stays valid.
    return emit(color * coverage);
}
