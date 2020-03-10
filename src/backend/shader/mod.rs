use std::{
    ffi::{c_void, CString},
    mem, ptr, str,
};

use gl::types::*;

use crate::ErrDontCare;

/// `position` is at location 0 in both programs
const POSITION_ATTR: GLuint = 0;
/// We never use an offset into the vertex buffer
const VBO_OFFSET: *const c_void = ptr::null();

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
        // SAFETY: `ty` is either `gl::VERTEX_SHADER` or `gl::FRAGMENT_SHADER`
        shader = gl::CreateShader(ty);
        if shader == 0 {
            bug!("gl::CreateShader failed");
        }

        // SAFETY:
        // `shader` is a shader object created by OpenGL
        // `count` is one
        gl::ShaderSource(
            shader,
            1,
            &src as *const &str as *const *const _,
            &(src.len() as GLint),
        );

        // SAFETY: `shader` is a shader object created by OpenGL
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        // SAFETY: `gl::COMPILE_STATUS` is a valid `pname`
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            // SAFETY: `gl::INFO_LENGTH` is a valid `pname`
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            // SAFETY: `maxLength` is the value of `gl::INFO_LOG_LENGTH`
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            // SAFETY: the content has been written by `gl::GetShaderInfoLog`
            buf.set_len((len as usize) - 1);
            bug!(
                "{}",
                str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
            );
        }
    }

    shader
}

/// uses the created program
fn compile_program(vertex: &str, fragment: &str) -> Result<GLuint, ErrDontCare> {
    let vs = compile_shader(vertex, gl::VERTEX_SHADER);
    let fs = compile_shader(fragment, gl::FRAGMENT_SHADER);
    unsafe {
        // SAFETY: can not fail
        let program = gl::CreateProgram();
        if program == 0 {
            bug!("gl::CreateShader failed");
        }

        // SAFETY:
        // `program` is a valid program object
        // `vs` and `fs` are both unused valid shader objects
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        // SAFETY:
        // `program` is a valid program object and not active
        gl::LinkProgram(program);

        // SAFETY:
        // `program` is a valid program object
        // `gl::LINK_STATUS` is a valid `pname`
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != gl::TRUE as GLint {
            let mut len: GLint = 0;
            // SAFETY: `gl::COMPILE_STATUS` is a valid `pname`
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            // SAFETY: `maxLength` is the value of `gl::INFO_LOG_LENGTH`
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            // SAFETY: the content has been written by `gl::GetProgramInfoLog`
            buf.set_len((len as usize) - 1);
            bug!(
                "{}",
                str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
            );
        }

        // SAFETY:
        // `program` is a valid program object
        // `fs` and `vs` are both valid shaders and attached to `program`
        gl::DetachShader(program, fs);
        gl::DeleteShader(fs);
        gl::DetachShader(program, vs);
        gl::DeleteShader(vs);

        // SAFETY: no OpenGlState is currently alive
        super::update_program(program)?;

        // SAFETY: `colorNumber` is zero, which is less than `GL_MAX_DRAW_BUFFERS`
        let color_str = CString::new("color").unwrap();
        gl::BindFragDataLocation(program, 0, color_str.as_ptr());
        Ok(program)
    }
}

fn init_vertex_buffer(vbo: GLuint, data: &[GLfloat]) {
    unsafe {
        // SAFETY: `gl::ARRAY_BUFFER` is a valid `target` and `vbo` is valid
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // SAFETY:
        // `gl::ARRAY_BUFFER` is a valid buffer target
        // `gl::STATIC_DRAW` is a valid usage
        // `size` is positive
        // `vbo` is bound to `target`
        // `GL_BUFFER_IMMUTABLE_STORAGE` is not yet set
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(data) as GLsizeiptr,
            &data[0] as *const f32 as *const _,
            gl::STATIC_DRAW,
        );
        // check for oom
        let gl_error = gl::GetError();
        match gl_error {
            gl::NO_ERROR => (),
            gl::OUT_OF_MEMORY => {
                // TODO: OpenGl is now in an undefined state,
                // consider aborting instead, as it is possible
                // to catch a panic
                panic!("OpenGl is out of memory and in an invalid state");
            }
            e => bug!("unexpected error: {}", e),
        }
    }
}

fn get_uniform_id(program: GLuint, name_str: &str) -> GLint {
    let name = CString::new(name_str).unwrap();
    // SAFETY:`self.id` is a valid and linked program object
    let id = unsafe { gl::GetUniformLocation(program, name.as_ptr()) };

    if id == -1 {
        bug!("unknown uniform in program {}: {}", program, name_str)
    } else {
        id
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
            // SAFETY: `n` is positive
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            // SAFETY: `vao` was just returned from `gl::GenVertexArrays`
            gl::BindVertexArray(vao);

            init_vertex_buffer(vbo, &VERTEX_DATA);

            // SAFETY:
            // `vao` is the currently bound vertex array
            // `position` was specified with `layout (location = 0) in vec2`
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            gl::EnableVertexAttribArray(POSITION_ATTR);
            // SAFETY:
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            // `size` is two
            // `gl::FLOAT` is an accepted value
            // `stride` is zero
            // the offset into `vbo` is zero
            gl::VertexAttribPointer(
                POSITION_ATTR,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                VBO_OFFSET,
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

    pub fn get_uniforms(&self) -> Uniforms {
        Uniforms {
            source: get_uniform_id(self.id, "source"),
            color_modulation: get_uniform_id(self.id, "color_modulation"),
            invert_color: get_uniform_id(self.id, "invert_color"),
            flip_vertically: get_uniform_id(self.id, "flip_vertically"),
            flip_horizontally: get_uniform_id(self.id, "flip_horizontally"),
            target_dimensions: get_uniform_id(self.id, "target_dimensions"),
            source_texture_dimensions: get_uniform_id(self.id, "source_texture_dimensions"),
            source_texture_offset: get_uniform_id(self.id, "source_texture_offset"),
            source_dimensions: get_uniform_id(self.id, "source_dimensions"),
            source_position: get_uniform_id(self.id, "source_position"),
            source_scale: get_uniform_id(self.id, "source_scale"),
            depth: get_uniform_id(self.id, "depth"),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `id` was generated by OpenGL and `n` is one
            gl::DeleteProgram(self.id);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Uniforms {
    pub source: GLint,
    pub color_modulation: GLint,
    pub invert_color: GLint,
    pub flip_vertically: GLint,
    pub flip_horizontally: GLint,
    pub target_dimensions: GLint,
    pub source_texture_dimensions: GLint,
    pub source_texture_offset: GLint,
    pub source_dimensions: GLint,
    pub source_position: GLint,
    pub source_scale: GLint,
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

        let mut vao = [0; 2];
        let mut vbo = [0; 2];

        unsafe {
            // SAFETY: `n` is positive
            gl::GenVertexArrays(2, &mut vao as *mut [GLuint] as *mut GLuint);
            gl::GenBuffers(2, &mut vbo as *mut [GLuint] as *mut GLuint);

            // SAFETY: `vao` was just returned from `gl::GenVertexArrays`
            gl::BindVertexArray(vao[0]);
            init_vertex_buffer(vbo[0], &LINES_VERTEX_DATA);

            // SAFETY:
            // `vao[0]` is the currently bound vertex array
            // `position` was specified with `layout (location = 0) in vec4`
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            gl::EnableVertexAttribArray(POSITION_ATTR);
            // SAFETY:
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            // `size` is four
            // `gl::FLOAT` is an accepted value
            // `stride` is zero
            // the offset into `vbo` is zero
            gl::VertexAttribPointer(
                POSITION_ATTR,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            // SAFETY: `vao` was just returned from `gl::GenVertexArrays`
            gl::BindVertexArray(vao[1]);
            init_vertex_buffer(vbo[1], &RECTANGLES_VERTEX_DATA);

            // SAFETY:
            // `vao[1]` is the currently bound vertex array
            // `position` was specified with `layout (location = 0) in vec4`
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            gl::EnableVertexAttribArray(POSITION_ATTR);
            // SAFETY:
            // `POSITION_ATTR` is less than `GL_MAX_VERTEX_ATTRIBS`
            // `size` is two
            // `gl::FLOAT` is an accepted value
            // `stride` is zero
            // the offset into `vbo` is zero
            gl::VertexAttribPointer(
                POSITION_ATTR,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
        }

        let line_color_uniform = get_uniform_id(program, "line_color");
        let start_end = get_uniform_id(program, "start_end");

        Ok((
            Self {
                id: program,
                vao,
                vbo,
            },
            DebugUniforms {
                line_color: line_color_uniform,
                start_end,
            },
        ))
    }
}

impl Drop for DebugProgram {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `id` was generated by OpenGL and `n` is two
            gl::DeleteProgram(self.id);
            gl::DeleteBuffers(2, &self.vbo as *const [GLuint] as *const GLuint);
            gl::DeleteVertexArrays(2, &self.vao as *const [GLuint] as *const GLuint);
        }
    }
}

#[derive(Debug)]
pub struct DebugUniforms {
    pub line_color: GLint,
    pub start_end: GLint,
}
