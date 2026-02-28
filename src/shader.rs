use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::ptr;

use gl::types::*;

use crate::math::{Matrix4, Vector3};

pub struct Shader {
    pub id: u32,
}

/// NOTE: mixture of `shader_s.h` and `shader_m.h` (the latter just contains
/// a few more setters for uniforms)
#[allow(dead_code)]
impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Result<Shader, String> {
        let mut shader = Shader { id: 0 };

        // 1. retrieve the vertex/fragment source code from filesystem
        let mut vshader_file = File::open(vertex_path)
            .map_err(|e| format!("Failed to open vertex shader '{}': {}", vertex_path, e))?;
        let mut fshader_file = File::open(fragment_path)
            .map_err(|e| format!("Failed to open fragment shader '{}': {}", fragment_path, e))?;
        let mut vertex_code = String::new();
        let mut fragment_code = String::new();
        vshader_file
            .read_to_string(&mut vertex_code)
            .map_err(|e| format!("Failed to read vertex shader: {}", e))?;
        fshader_file
            .read_to_string(&mut fragment_code)
            .map_err(|e| format!("Failed to read fragment shader: {}", e))?;

        let vshader_code = CString::new(vertex_code.as_bytes())
            .map_err(|e| format!("Vertex shader contains null byte: {}", e))?;
        let fshader_code = CString::new(fragment_code.as_bytes())
            .map_err(|e| format!("Fragment shader contains null byte: {}", e))?;

        // 2. compile shaders
        unsafe {
            // vertex shader
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &vshader_code.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            shader.check_compile_errors(vertex, "VERTEX")?;

            // fragment shader
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &fshader_code.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            shader.check_compile_errors(fragment, "FRAGMENT")?;
            // shader program
            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.check_compile_errors(id, "PROGRAM")?;
            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            shader.id = id;
        }

        Ok(shader)
    }

    /// activate the shader
    /// ------------------------------------------------------------------------
    pub unsafe fn use_program(&self) {
        unsafe { gl::UseProgram(self.id) }
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    pub unsafe fn set_bool(&self, name: &CStr, value: bool) {
        unsafe { gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32) };
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_int(&self, name: &CStr, value: i32) {
        unsafe { gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value) };
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_float(&self, name: &CStr, value: f32) {
        unsafe { gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value) };
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector3(&self, name: &CStr, value: &Vector3) {
        unsafe {
            gl::Uniform3fv(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                1,
                value.as_ptr(),
            )
        };
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec3(&self, name: &CStr, x: f32, y: f32, z: f32) {
        unsafe { gl::Uniform3f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z) };
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_mat4(&self, name: &CStr, mat: &Matrix4) {
        unsafe {
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                1,
                gl::FALSE,
                mat.as_ptr(),
            )
        };
    }

    /// utility function for checking shader compilation/linking errors.
    /// ------------------------------------------------------------------------
    unsafe fn check_compile_errors(&self, shader: u32, type_: &str) -> Result<(), String> {
        let mut success = gl::FALSE as GLint;
        let mut info_log = vec![0_u8; 1024];

        if type_ != "PROGRAM" {
            unsafe {
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

                if success != gl::TRUE as GLint {
                    gl::GetShaderInfoLog(
                        shader,
                        1024,
                        ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );
                    let error_msg = String::from_utf8_lossy(&info_log);
                    let error_msg = error_msg.trim_matches('\0');
                    return Err(format!(
                        "Shader compilation error ({}): {}",
                        type_, error_msg
                    ));
                }
            };
        } else {
            unsafe {
                gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);

                if success != gl::TRUE as GLint {
                    gl::GetProgramInfoLog(
                        shader,
                        1024,
                        ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );
                    let error_msg = String::from_utf8_lossy(&info_log);
                    let error_msg = error_msg.trim_matches('\0');
                    return Err(format!("Program linking error ({}): {}", type_, error_msg));
                }
            };
        }
        Ok(())
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        if self.id != 0 {
            unsafe {
                gl::DeleteProgram(self.id);
            }
        }
    }
}
