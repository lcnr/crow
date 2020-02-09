use std::{
    ffi::{self, CStr, CString},
    ptr, slice,
};

use gl::types::*;
use glutin::{ContextWrapper, EventsLoop, PossiblyCurrent, Window, WindowBuilder};

use crate::ErrDontCare;

mod draw;
mod shader;
mod state;
pub(crate) mod tex;

use tex::RawTexture;

use shader::{LinesProgram, Program};
use state::OpenGlState;

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
    state: OpenGlState,
    events_loop: EventsLoop,
    gl_window: ContextWrapper<PossiblyCurrent, Window>,
    program: Program,
    lines_program: LinesProgram,
}

impl Backend {
    pub fn initialize(window: WindowBuilder, events_loop: EventsLoop) -> Result<Self, ErrDontCare> {
        let gl_window = glutin::ContextBuilder::new()
            .with_depth_buffer(16)
            .with_vsync(false)
            .build_windowed(window, &events_loop)
            .unwrap();

        // It is essential to make the context current before calling `gl::load_with`.
        let gl_window = unsafe { gl_window.make_current() }.unwrap();

        // Load the OpenGL function pointers
        // TODO: `as *const _` will not be needed once glutin is updated to the latest gl version
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(debug_callback), ptr::null());
        }

        unsafe {
            gl::Enable(gl::BLEND);
        }

        let (program, uniforms) = Program::new()?;
        let (lines_program, lines_uniforms) = LinesProgram::new()?;

        let state = OpenGlState::new(
            uniforms,
            lines_uniforms,
            (program.id, program.vao),
            gl_window
                .window()
                .get_inner_size()
                .map_or((1024, 720), |s| s.into()),
        );

        Ok(Self {
            state,
            events_loop,
            gl_window,
            program,
            lines_program,
        })
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.gl_window
            .window()
            .set_inner_size(From::from((width, height)))
    }

    pub fn window(&self) -> &Window {
        self.gl_window.window()
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        self.gl_window.window().get_inner_size().unwrap().into()
    }

    pub fn take_screenshot(&mut self, (width, height): (u32, u32)) -> Vec<u8> {
        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = 4 * width as usize * height as usize;
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        self.state.update_framebuffer(0);
        unsafe {
            gl::ReadPixels(
                0,
                0,
                width as _,
                height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
            if gl::GetError() != gl::NO_ERROR {
                panic!("failed to take a screenshot");
            }
            data.set_len(byte_count);
        }

        data
    }

    pub fn get_image_data(&mut self, texture: &RawTexture) -> Vec<u8> {
        let (width, height) = texture.dimensions;

        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = 4 * width as usize * height as usize;
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        unsafe {
            // FIXME: consider using glGetTextureImage even if it is only supported since OpenGL 4.5
            self.state.update_texture(texture.id);
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
            if gl::GetError() != gl::NO_ERROR {
                panic!("failed to take a screenshot");
            }
            data.set_len(byte_count);
        }

        data
    }

    pub fn clear_texture_depth(&mut self, texture: &mut RawTexture) -> Result<(), ErrDontCare> {
        self.state.update_framebuffer(texture.frame_buffer_id);
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }

    pub fn clear_color(
        &mut self,
        buffer_id: GLuint,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        self.state.update_framebuffer(buffer_id);
        unsafe {
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        Ok(())
    }

    pub fn finalize_frame(&mut self) -> Result<(), ErrDontCare> {
        self.gl_window.swap_buffers().unwrap();
        self.state.update_framebuffer(0);
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        Ok(())
    }

    pub fn events_loop(&mut self) -> &mut EventsLoop {
        &mut self.events_loop
    }
}
