// Mesh shape — single VAO+VBO, 6-float `xyz rgb` stride, `STATIC_DRAW`
// upload — follows alumina-interface's `GpuLines` in `src/renderer.rs`
// (Timothy Schmidt, MIT). See `render3d/mod.rs` for the full credit note.

use glow::{Context, HasContext as _};

/// A VBO+VAO pair with 6 floats per vertex: `xyz rgb`.
/// Bind with an `UnlitProgram` before calling `draw`.
pub struct ColoredMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    vertex_count: i32,
    primitive: u32, // glow::LINES or glow::TRIANGLES
}

impl ColoredMesh {
    /// Allocate the VAO + VBO pair and configure attribute pointers
    /// for the `xyz rgb` stride. The mesh starts empty; call
    /// `upload()` to fill it.
    ///
    /// # Safety
    ///
    /// `gl` must be the current OpenGL context on the calling thread.
    /// `primitive` must be one of `glow::LINES`, `glow::TRIANGLES`,
    /// or any primitive constant accepted by `gl.draw_arrays`.
    pub unsafe fn new(gl: &Context, primitive: u32) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().expect("create_vertex_array");
            let vbo = gl.create_buffer().expect("create_buffer");
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            // stride = 6 floats * 4 bytes = 24; position at 0, color at 12.
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 24, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 24, 12);
            gl.bind_vertex_array(None);
            Self { vao, vbo, vertex_count: 0, primitive }
        }
    }

    /// Upload `verts` to the VBO with `STATIC_DRAW`. The slice
    /// length must be a multiple of 6 — each vertex is `xyz rgb`.
    ///
    /// # Safety
    ///
    /// `gl` must be the current OpenGL context on the calling thread.
    /// Panics if the slice length is not a multiple of 6.
    pub unsafe fn upload(&mut self, gl: &Context, verts: &[f32]) {
        assert_eq!(verts.len() % 6, 0, "ColoredMesh verts must be in 6-float (xyz rgb) stride");
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(verts),
                glow::STATIC_DRAW,
            );
        }
        self.vertex_count = (verts.len() / 6) as i32;
    }

    /// Issue the draw call for this mesh's vertex buffer using its
    /// configured primitive. No-op if the mesh is empty.
    ///
    /// # Safety
    ///
    /// `gl` must be the current OpenGL context on the calling thread,
    /// and a compatible shader program — `UnlitProgram` — must be
    /// bound first.
    pub unsafe fn draw(&self, gl: &Context) {
        if self.vertex_count == 0 {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(self.primitive, 0, self.vertex_count);
        }
    }
}

unsafe impl Send for ColoredMesh {}
unsafe impl Sync for ColoredMesh {}
