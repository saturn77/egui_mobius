/// XY-plane gridlines at Z=0 for spatial context. Returns an `xyz rgb` vertex
/// buffer for the LINES primitive. Lines sit at integer multiples of `step`
/// from world origin so the corner at world (0,0) always lands on a grid
/// intersection — including the centerlines at x=0 and y=0. The axes gizmo
/// renders at a tiny positive Z (see `axes_vertices`'s `z_base`) so it
/// wins the depth test cleanly over these without Z-fighting.
pub fn grid_vertices(half_extent: f32, step: f32, color: [f32; 3]) -> Vec<f32> {
    let [r, g, b] = color;
    let n = (half_extent / step).floor() as i32;
    let mut v = Vec::with_capacity((4 * n as usize) * 12);

    // Lines parallel to Y axis (varying x)
    for i in -n..=n {
        let x = i as f32 * step;
        v.extend_from_slice(&[x, -half_extent, 0.0, r, g, b]);
        v.extend_from_slice(&[x,  half_extent, 0.0, r, g, b]);
    }

    // Lines parallel to X axis (varying y)
    for i in -n..=n {
        let y = i as f32 * step;
        v.extend_from_slice(&[-half_extent, y, 0.0, r, g, b]);
        v.extend_from_slice(&[ half_extent, y, 0.0, r, g, b]);
    }

    v
}
