use std::{ffi::CString, ptr, str};

use gl::types::*;

use crate::ErrDontCare;

const VERTEX: &str = include_str!("vertex.glsl");
const FRAGMENT: &str = include_str!("fragment.glsl");

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
            );
        }
    }

    shader
}

#[derive(Debug)]
pub struct Program {
    pub id: GLuint,
}

impl Program {
    pub fn new() -> Result<Self, ErrDontCare> {
        let vs = compile_shader(VERTEX, gl::VERTEX_SHADER);
        let fs = compile_shader(FRAGMENT, gl::FRAGMENT_SHADER);

        let program;
        unsafe {
            program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
                );
            }

            gl::DeleteShader(fs);
            gl::DeleteShader(vs);
        }

        Ok(Program { id: program })
    }

    fn get_uniform_id(&self, name_str: &str) -> GLint {
        let name = CString::new(name_str).unwrap();
        let id = unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) };

        if id == -1 {
            panic!("unknown uniform: {}", name_str)
        } else {
            id
        }
    }

    pub fn get_uniforms(&self) -> Uniforms {
        Uniforms {
            object: self.get_uniform_id("object"),
            color_modulation: self.get_uniform_id("color_modulation"),
            invert_color: self.get_uniform_id("invert_color"),
            flip_vertically: self.get_uniform_id("flip_vertically"),
            flip_horizontally: self.get_uniform_id("flip_horizontally"),
            target_dimensions: self.get_uniform_id("target_dimensions"),
            object_texture_dimensions: self.get_uniform_id("object_texture_dimensions"),
            object_texture_offset: self.get_uniform_id("object_texture_offset"),
            object_dimensions: self.get_uniform_id("object_dimensions"),
            object_position: self.get_uniform_id("object_position"),
            object_scale: self.get_uniform_id("object_scale"),
            depth: self.get_uniform_id("depth"),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[derive(Debug)]
pub struct Uniforms {
    pub object: GLint,
    pub color_modulation: GLint,
    pub invert_color: GLint,
    pub flip_vertically: GLint,
    pub flip_horizontally: GLint,
    pub target_dimensions: GLint,
    pub object_texture_dimensions: GLint,
    pub object_texture_offset: GLint,
    pub object_dimensions: GLint,
    pub object_position: GLint,
    pub object_scale: GLint,
    pub depth: GLint,
}
