use std::{ffi::CString, mem, ptr, str};

use gl::types::*;

use crate::ErrDontCare;

#[rustfmt::skip]
static VERTEX_DATA: [GLfloat; 8] = [
    0.0, 0.0,
    1.0, 0.0,
    0.0, 1.0,
    1.0, 1.0
];

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

fn compile_program(vertex: &str, fragment: &str) -> Result<GLuint, ErrDontCare> {
    let vs = compile_shader(vertex, gl::VERTEX_SHADER);
    let fs = compile_shader(fragment, gl::FRAGMENT_SHADER);
    let program;
    unsafe {
        program = gl::CreateProgram();
        assert_ne!(program, 0, "gl::CreateProgram() failed");
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

        gl::DetachShader(program, fs);
        gl::DeleteShader(fs);
        gl::DetachShader(program, vs);
        gl::DeleteShader(vs);
        Ok(program)
    }
}

#[derive(Debug)]
pub struct Program {
    pub id: GLuint,
    pub vao: GLuint,
    vbo: GLuint,
}

impl Program {
    pub fn new() -> Result<(Self, Uniforms), ErrDontCare> {
        let program = compile_program(VERTEX, FRAGMENT)?;
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mem::size_of_val(&VERTEX_DATA) as GLsizeiptr,
                &VERTEX_DATA[0] as *const _ as *const _,
                gl::STATIC_DRAW,
            );

            // Use shader program
            gl::UseProgram(program);
            let color_str = CString::new("color").unwrap();
            gl::BindFragDataLocation(program, 0, color_str.as_ptr());

            // Specify the layout of the vertex data
            let pos_str = CString::new("position").unwrap();
            let pos_attr = gl::GetAttribLocation(program, pos_str.as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
        }

        let prog = Program {
            id: program,
            vao,
            vbo,
        };

        let uniforms = prog.get_uniforms();
        Ok((prog, uniforms))
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
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

#[derive(Debug, Clone)]
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

#[rustfmt::skip]
static LINES_VERTEX_DATA: [GLfloat; 8] = [
    1.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 1.0,
];

#[rustfmt::skip]
static RECTANGLES_VERTEX_DATA: [GLfloat; 20] = [
    1.0, 1.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,
    0.0, 1.0, 1.0, 0.0,
    1.0, 1.0, 0.0, 0.0,
];

/// vao 0 is for drawing lines
/// vao 1 for drawing rectangles
#[derive(Debug)]
pub struct DebugProgram {
    pub id: GLuint,
    pub vao: [GLuint; 2],
    pub vbo: [GLuint; 2],
}

impl DebugProgram {
    pub fn new() -> Result<(Self, DebugUniforms), ErrDontCare> {
        let program = compile_program(
            include_str!("vertex_debug.glsl"),
            include_str!("fragment_debug.glsl"),
        )?;

        let pos_attr = unsafe {
            // Use shader program
            gl::UseProgram(program);
            let color_str = CString::new("color").unwrap();
            gl::BindFragDataLocation(program, 0, color_str.as_ptr());

            // Specify the layout of the vertex data
            let pos_str = CString::new("position").unwrap();
            gl::GetAttribLocation(program, pos_str.as_ptr())
        };

        let mut vao = [0; 2];
        let mut vbo = [0; 2];

        unsafe {
            // Create Vertex Array and Buffer Objects
            gl::GenVertexArrays(2, &mut vao as *mut [GLuint] as *mut GLuint);
            gl::GenBuffers(2, &mut vbo as *mut [GLuint] as *mut GLuint);

            // initialize lines vao
            gl::BindVertexArray(vao[0]);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mem::size_of_val(&LINES_VERTEX_DATA) as GLsizeiptr,
                &LINES_VERTEX_DATA[0] as *const _ as *const _,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            // initialize rectangles vao
            gl::BindVertexArray(vao[1]);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                mem::size_of_val(&RECTANGLES_VERTEX_DATA) as GLsizeiptr,
                &RECTANGLES_VERTEX_DATA[0] as *const _ as *const _,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
        }

        let name_str = "line_color";
        let name = CString::new(name_str).unwrap();
        let color_uniform = unsafe { gl::GetUniformLocation(program, name.as_ptr()) };

        if color_uniform == -1 {
            panic!("unknown uniform: {}", name_str)
        }

        let name_str = "start_end";
        let name = CString::new(name_str).unwrap();
        let start_end = unsafe { gl::GetUniformLocation(program, name.as_ptr()) };

        if start_end == -1 {
            panic!("unknown uniform: {}", name_str)
        }

        Ok((
            Self {
                id: program,
                vao,
                vbo,
            },
            DebugUniforms {
                color: color_uniform,
                start_end,
            },
        ))
    }
}

impl Drop for DebugProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
            gl::DeleteBuffers(2, &self.vbo as *const [GLuint] as *const GLuint);
            gl::DeleteVertexArrays(2, &self.vao as *const [GLuint] as *const GLuint);
        }
    }
}

#[derive(Debug)]
pub struct DebugUniforms {
    pub color: GLint,
    pub start_end: GLint,
}
