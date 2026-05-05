/// 6-vertex RGB axis gizmo: X = red, Y = green, Z = blue.
/// Returns a flat `xyz rgb` buffer ready for `ColoredMesh::upload`.
///
/// `z_base` lets callers lift the origin slightly above the grid plane so
/// the X/Y axis lines don't Z-fight with grid centerlines at z=0. Pass 0
/// if the gizmo is rendered in isolation.
pub fn axes_vertices(length: f32, z_base: f32) -> Vec<f32> {
    let l = length;
    let z = z_base;
    vec![
        // X axis: red
        0.0, 0.0, z,        1.0, 0.0, 0.0,
        l,   0.0, z,        1.0, 0.0, 0.0,
        // Y axis: green
        0.0, 0.0, z,        0.0, 1.0, 0.0,
        0.0, l,   z,        0.0, 1.0, 0.0,
        // Z axis: blue
        0.0, 0.0, z,        0.0, 0.0, 1.0,
        0.0, 0.0, l + z,    0.0, 0.0, 1.0,
    ]
}
