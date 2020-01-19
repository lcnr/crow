use std::{
    ffi::{self, CStr, CString},
    mem, ptr, slice, str,
};

use gl::types::*;
use glutin::{ContextWrapper, EventsLoop, PossiblyCurrent, Window, WindowBuilder};

use crate::ErrDontCare;

mod draw;
mod shader;
pub(crate) mod tex;

use shader::{Program, Uniforms};

static VERTEX_DATA: [GLfloat; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];

extern "system" fn debug_callback(
    source: GLenum,
    ty: GLenum,
    _id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _: *mut ffi::c_void,
) {
    println!("OPEN GL ERROR:");
    print!("  SOURCE: ");
    match source {
        gl::DEBUG_SOURCE_API => println!("DEBUG_SOURCE_API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("DEBUG_SOURCE_WINDOW_SYSTEM"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("DEBUG_SOURCE_SHADER_COMPILER"),
        gl::DEBUG_SOURCE_THIRD_PARTY => println!("DEBUG_SOURCE_THIRD_PARTY"),
        gl::DEBUG_SOURCE_APPLICATION => println!("DEBUG_SOURCE_APPLICATION"),
        gl::DEBUG_SOURCE_OTHER => println!("DEBUG_SOURCE_OTHER"),
        unexpected => println!("UNEXPECTED: {}", unexpected),
    };

    print!("  TYPE: ");
    match ty {
        gl::DEBUG_TYPE_ERROR => println!("DEBUG_TYPE_ERROR"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("DEBUG_TYPE_DEPRECATED_BEHAVIOR"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("DEBUG_TYPE_UNDEFINED_BEHAVIOR"),
        gl::DEBUG_TYPE_PORTABILITY => println!("DEBUG_TYPE_PORTABILITY"),
        gl::DEBUG_TYPE_PERFORMANCE => println!("DEBUG_TYPE_PERFORMANCE"),
        gl::DEBUG_TYPE_MARKER => println!("DEBUG_TYPE_MARKER"),
        gl::DEBUG_TYPE_PUSH_GROUP => println!("DEBUG_TYPE_PUSH_GROUP"),
        gl::DEBUG_TYPE_POP_GROUP => println!("DEBUG_TYPE_POP_GROUP"),
        gl::DEBUG_TYPE_OTHER => println!("DEBUG_TYPE_OTHER"),
        unexpected => println!("UNEXPECTED: {}", unexpected),
    };

    print!("  SEVERITY: ");
    match severity {
        gl::DEBUG_SEVERITY_LOW => println!("DEBUG_SEVERITY_LOW"),
        gl::DEBUG_SEVERITY_MEDIUM => println!("DEBUG_SEVERITY_MEDIUM"),
        gl::DEBUG_SEVERITY_HIGH => println!("DEBUG_SEVERITY_HIGH"),
        gl::DEBUG_SEVERITY_NOTIFICATION => println!("DEBUG_SEVERITY_NOTIFICATION"),
        unexpected => println!("UNEXPECTED: {}", unexpected),
    };

    let s;
    let msg = unsafe {
        if length < 0 {
            CStr::from_ptr(message)
        } else {
            let slice: &[u8] = slice::from_raw_parts(message as *const u8, length as usize);
            s = CString::new(slice).unwrap();
            &s
        }
    };

    println!("  MESSAGE: {}", msg.to_str().unwrap());
}

#[derive(Debug)]
pub struct Backend {
    uniforms: Uniforms,
    events_loop: EventsLoop,
    gl_window: ContextWrapper<PossiblyCurrent, Window>,
    program: Program,
    frame_buffers: Vec<GLuint>,
    vao: GLuint,
    vbo: GLuint,
}

impl Drop for Backend {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Backend {
    pub fn initialize(window: WindowBuilder) -> Result<Self, ErrDontCare> {
        let events_loop = EventsLoop::new();
        let gl_window = glutin::ContextBuilder::new()
            .build_windowed(window, &events_loop)
            .unwrap();

        // It is essential to make the context current before calling `gl::load_with`.
        let gl_window = unsafe { gl_window.make_current() }.unwrap();

        // Load the OpenGL function pointers
        // TODO: `as *const _` will not be needed once glutin is updated to the latest gl version
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), 0 as *const _);
        }

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Enable(gl::DEPTH_TEST);
        }

        let program = Program::new()?;

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
                (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&VERTEX_DATA[0]),
                gl::STATIC_DRAW,
            );

            // Use shader program
            gl::UseProgram(program.id);
            gl::BindFragDataLocation(program.id, 0, CString::new("out_color").unwrap().as_ptr());

            // Specify the layout of the vertex data
            let pos_attr =
                gl::GetAttribLocation(program.id, CString::new("position").unwrap().as_ptr());
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

        // prepare screen for the first frame
        unsafe {
            gl::ClearColor(0.1, 0.4, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let uniforms = program.get_uniforms();

        Ok(Self {
            uniforms,
            events_loop,
            gl_window,
            frame_buffers: Vec::new(),
            program,
            vao,
            vbo,
        })
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.gl_window
            .window()
            .set_inner_size(From::from((width, height)))
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        self.gl_window.window().get_inner_size().unwrap().into()
    }

    pub fn clear_texture_depth(
        &mut self,
        texture: &mut tex::RawTexture,
    ) -> Result<(), ErrDontCare> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, texture.frame_buffer_id);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }

    pub fn clear_texture_color(
        &mut self,
        texture: &mut tex::RawTexture,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, texture.frame_buffer_id);

            let mut old = [1.0, 1.0, 1.0, 1.0];
            gl::GetFloatv(
                gl::COLOR_CLEAR_VALUE,
                &mut old as *mut [GLfloat; 4] as *mut GLfloat,
            );
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(old[0], old[1], old[2], old[3]);
        }

        Ok(())
    }

    pub fn finalize_frame(&mut self) -> Result<(), ErrDontCare> {
        self.gl_window.swap_buffers().unwrap();
        unsafe {
            // reset the depth of each drawn to texture
            for frame_buffer in self.frame_buffers.drain(..) {
                gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }

    pub fn events_loop(&mut self) -> &mut EventsLoop {
        &mut self.events_loop
    }
}
