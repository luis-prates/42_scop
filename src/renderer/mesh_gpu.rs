use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use crate::renderer::shader_program::ShaderProgram;
use crate::scene::{TextureKind, Vertex};

#[derive(Clone)]
pub struct GpuTexture {
    pub id: u32,
    pub kind: TextureKind,
}

pub struct MeshGpu {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<GpuTexture>,
    pub has_uv_mapping: bool,
    pub vao: u32,
    vbo: u32,
    ebo: u32,
}

impl MeshGpu {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        textures: Vec<GpuTexture>,
        has_uv_mapping: bool,
    ) -> Result<Self, String> {
        let mut mesh = Self {
            vertices,
            indices,
            textures,
            has_uv_mapping,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        mesh.setup_mesh();
        Ok(mesh)
    }

    pub fn draw(&self, shader: &ShaderProgram) {
        unsafe {
            let mut diffuse_nr = 0;
            let mut specular_nr = 0;
            let mut normal_nr = 0;

            for (i, texture) in self.textures.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                let name = texture.kind.shader_uniform_prefix();
                let number = match texture.kind {
                    TextureKind::Diffuse => {
                        diffuse_nr += 1;
                        diffuse_nr
                    }
                    TextureKind::Specular => {
                        specular_nr += 1;
                        specular_nr
                    }
                    TextureKind::Normal => {
                        normal_nr += 1;
                        normal_nr
                    }
                };

                let sampler = CString::new(format!("{}{}", name, number))
                    .expect("shader sampler names are static ASCII");
                gl::Uniform1i(
                    gl::GetUniformLocation(shader.id(), sampler.as_ptr()),
                    i as i32,
                );
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
            }

            let use_generated_mapping = c_str!("useGeneratedMapping");
            gl::Uniform1i(
                gl::GetUniformLocation(shader.id(), use_generated_mapping.as_ptr()),
                if self.has_uv_mapping { 0 } else { 1 },
            );

            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);

            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    pub fn update_vertices(&mut self, vertices: &[Vertex]) {
        self.vertices.clear();
        self.vertices.extend_from_slice(vertices);
        self.upload_vertex_buffer();
    }

    fn setup_mesh(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);

            gl::BindVertexArray(self.vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            upload_buffer_data(gl::ARRAY_BUFFER, &self.vertices);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            upload_buffer_data(gl::ELEMENT_ARRAY_BUFFER, &self.indices);

            let size = size_of::<Vertex>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, position) as *const c_void,
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, normal) as *const c_void,
            );

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, tex_coords) as *const c_void,
            );

            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, tangent) as *const c_void,
            );

            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, bitangent) as *const c_void,
            );

            gl::EnableVertexAttribArray(5);
            gl::VertexAttribPointer(
                5,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, color) as *const c_void,
            );

            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(
                6,
                3,
                gl::FLOAT,
                gl::FALSE,
                size,
                offset_of!(Vertex, new_color) as *const c_void,
            );

            gl::BindVertexArray(0);
        }
    }

    fn upload_vertex_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            upload_buffer_data(gl::ARRAY_BUFFER, &self.vertices);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

unsafe fn upload_buffer_data<T>(target: u32, data: &[T]) {
    let size = (std::mem::size_of_val(data)) as isize;
    let ptr = if data.is_empty() {
        ptr::null()
    } else {
        data.as_ptr() as *const c_void
    };

    unsafe {
        gl::BufferData(target, size, ptr, gl::STATIC_DRAW);
    }
}

impl Drop for MeshGpu {
    fn drop(&mut self) {
        unsafe {
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
            }
            if self.vbo != 0 {
                gl::DeleteBuffers(1, &self.vbo);
            }
            if self.ebo != 0 {
                gl::DeleteBuffers(1, &self.ebo);
            }
        }
    }
}
