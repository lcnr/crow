use std::{cmp, convert::TryFrom, ffi::CStr};

use static_assertions::{assert_type_eq_all, const_assert_eq};

use gl::types::*;
use glutin::{ContextWrapper, EventsLoop, PossiblyCurrent, Window, WindowBuilder};

use crate::{FinalizeError, NewContextError};

mod draw;
mod shader;
mod state;
pub(crate) mod tex;

use tex::RawTexture;

use shader::{DebugProgram, Program};
use state::OpenGlState;

assert_type_eq_all!(GLfloat, f32);
const_assert_eq!(true as GLboolean, gl::TRUE);
const_assert_eq!(false as GLboolean, gl::FALSE);

#[allow(non_upper_case_globals)]
const ARB_framebuffer_no_attachments: &[u8] = b"GL_ARB_framebuffer_no_attachments\0";

#[derive(Debug)]
pub struct GlConstants {
    pub max_texture_size: (u32, u32),
}

impl GlConstants {
    pub fn load() -> Self {
        fn get(pname: GLenum, name: &str) -> u32 {
            let mut v = 0;
            unsafe {
                // SAFETY: `pname` is valid
                gl::GetIntegerv(pname, &mut v);
            }

            if let Ok(v) = u32::try_from(v) {
                v
            } else {
                bug!("unexpected `max_{}`: {}", name, v)
            }
        }

        // must be at least 1024
        let texture_size = get(gl::MAX_TEXTURE_SIZE, "texture_size");
        let renderbuffer_size = get(gl::MAX_RENDERBUFFER_SIZE, "renderbuffer_size");
        let size = cmp::min(texture_size, renderbuffer_size);

        // FIXES https://github.com/lcnr/crow/issues/15
        // only check the max framebuffer size if the extension
        // `ARB_framebuffer_no_attachments` exists
        unsafe {
            // TODO: change the constant to `&CStr` once `CStr::from_bytes_with_nul_unchecked` is const
            let expected_extension =
                CStr::from_bytes_with_nul(ARB_framebuffer_no_attachments).unwrap();
            for i in 0.. {
                let extension = gl::GetStringi(gl::EXTENSIONS, i);
                let err = gl::GetError();
                match err {
                    gl::NO_ERROR => {
                        let extension = CStr::from_ptr(extension.cast());
                        if expected_extension == extension {
                            eprintln!("EXTENSION FOUND");
                            let framebuffer_width =
                                get(gl::MAX_FRAMEBUFFER_WIDTH, "framebuffer_width");
                            let framebuffer_height =
                                get(gl::MAX_FRAMEBUFFER_HEIGHT, "framebuffer_height");

                            return GlConstants {
                                max_texture_size: (
                                    cmp::min(size, framebuffer_width),
                                    cmp::min(size, framebuffer_height),
                                ),
                            };
                        }
                    }
                    gl::INVALID_VALUE => break,
                    err => bug!("unexpected error: {:?}", err),
                }
            }
        }

        eprintln!("EXTENSION NOT FOUND");
        GlConstants {
            max_texture_size: (size, size),
        }
    }
}

#[derive(Debug)]
pub struct Backend {
    state: OpenGlState,
    events_loop: EventsLoop,
    gl_context: ContextWrapper<PossiblyCurrent, Window>,
    constants: GlConstants,
    program: Program,
    debug_program: DebugProgram,
}

impl Backend {
    pub fn initialize(window: WindowBuilder) -> Result<Self, NewContextError> {
        let events_loop = EventsLoop::new();
        let gl_context = glutin::ContextBuilder::new()
            .with_depth_buffer(16)
            .with_vsync(false)
            .build_windowed(window, &events_loop)
            .map_err(NewContextError::CreationError)?;

        // It is essential to make the context current before calling `gl::load_with`.
        let gl_context = unsafe {
            gl_context
                .make_current()
                .map_err(|(_, e)| NewContextError::ContextError(e))?
        };

        // Load the OpenGL function pointers
        // TODO: `as *const _` will not be needed once glutin is updated to the latest gl version
        gl::load_with(|symbol| gl_context.get_proc_address(symbol) as *const _);

        unsafe {
            // SAFETY: `gl::BLEND` is a valid capability
            gl::Enable(gl::BLEND);
        }

        let (program, uniforms) = Program::new();
        let (debug_program, debug_uniforms) = DebugProgram::new();

        let state = OpenGlState::new(
            uniforms,
            debug_uniforms,
            (program.id, program.vao),
            gl_context
                .window()
                .get_inner_size()
                .map_or((1024, 720), |s| s.into()),
        );

        let constants = dbg!(GlConstants::load());

        Ok(Self {
            state,
            events_loop,
            gl_context,
            constants,
            program,
            debug_program,
        })
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.gl_context
            .window()
            .set_inner_size(From::from((width, height)))
    }

    pub fn window(&self) -> &Window {
        self.gl_context.window()
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        if let Some(dimensions) = self.gl_context.window().get_inner_size() {
            dimensions.into()
        } else {
            bug!("failed to get window_dimensions")
        }
    }

    pub fn take_screenshot(&mut self, (width, height): (u32, u32)) -> Vec<u8> {
        let byte_count = usize::checked_mul(height as usize, width as usize)
            .and_then(|p| p.checked_mul(4))
            .unwrap_or_else(|| {
                bug!(
                    "screen byte count does not fit into a usize: {}x{}",
                    width,
                    height
                )
            });
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        self.state.update_framebuffer(0);
        unsafe {
            // SAFETY:
            // `gl::RGBA` is an accepted format
            // `gl::UNSIGNED_BYTE` is an accepted type
            // `width` and `height` are both positive
            // `GL_PIXEL_PACK_BUFFER` and `GL_READ_FRAMEBUFFER_BINDING`
            //      are never used and zero by default
            gl::ReadPixels(
                0,
                0,
                width as _,
                height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
            // SAFETY: the buffer has the correct capacity and has been initialized by gl::ReadPixels
            data.set_len(byte_count);
        }

        data
    }

    pub fn get_image_data(&mut self, texture: &RawTexture) -> Vec<u8> {
        let (width, height) = texture.dimensions;

        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = usize::checked_mul(height as usize, width as usize)
            .and_then(|p| p.checked_mul(4))
            .unwrap_or_else(|| {
                bug!(
                    "texture byte count does not fit into a usize: {}x{}",
                    width,
                    height
                )
            });
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        unsafe {
            self.state.update_texture(texture.id);
            // SAFETY:
            // `gl::TEXTURE_2D` is an accepted target
            // `gl::RGBA` is an accepted format
            // `gl::UNSIGNED_BYTE` is an accepted type
            // `level` is set to 0
            // `GL_PIXEL_PACK_BUFFER` is never used and zero by default.
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );

            // SAFETY: the buffer has the correct capacity and has been initialized by gl::GetTexImage
            data.set_len(byte_count);
        }

        data
    }

    pub fn clear_depth(&mut self, framebuffer: GLuint) {
        self.state.update_framebuffer(framebuffer);
        unsafe {
            // SAFETY:
            // no undefined bit is set in `mask`
            // `glBegin` and `glEnd` are never used
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn clear_color(&mut self, buffer_id: GLuint, color: (f32, f32, f32, f32)) {
        self.state.update_framebuffer(buffer_id);
        unsafe {
            // SAFETY: this function is always safe
            gl::ClearColor(color.0, color.1, color.2, color.3);
            // SAFETY:
            // no undefined bit is set in `mask`
            // `glBegin` and `glEnd` are never used
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn finalize_frame(&mut self) -> Result<(), FinalizeError> {
        self.gl_context
            .swap_buffers()
            .map_err(FinalizeError::ContextError)?;
        self.state.update_framebuffer(0);
        self.clear_depth(0);
        Ok(())
    }

    pub fn constants(&self) -> &GlConstants {
        &self.constants
    }

    pub fn events_loop(&mut self) -> &mut EventsLoop {
        &mut self.events_loop
    }
}

/// Sets the currently active program to `program`.
///
/// SAFETY: this function must only be called by `OpenGlState` once the state exists
unsafe fn update_program(program: GLuint) {
    // SAFETY:
    // `program` is a value generated by OpenGl
    // `program` is a program object
    // transform feedback mode is not active
    gl::UseProgram(program);

    // check if `program` could not be made part of current state
    let gl_error = gl::GetError();
    if gl_error != gl::NO_ERROR {
        // TODO: consider returning an error
        //
        // According to the reference (https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glUseProgram.xhtml),
        // this function may fail with `gl_error == GL_INVALID_OPERATION` if this operation failed.
        //
        // As both the state after failure is not described and I don't when such failures could occur,
        // we just `panic` for now.
        bug!("unexpected error: {}", gl_error)
    }
}
