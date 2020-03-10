use std::{ffi::c_void, ptr};

use gl::types::*;

use image::RgbaImage;

use crate::{backend::Backend, DrawConfig, NewTextureError, UnwrapBug};

#[derive(Debug)]
pub struct RawTexture {
    pub id: GLuint,
    pub framebuffer_id: GLuint,
    pub depth_id: GLuint,
    pub dimensions: (u32, u32),
    pub has_framebuffer: bool,
}

impl Drop for RawTexture {
    fn drop(&mut self) {
        // SAFETY: `n` is `1` for all functions
        if self.has_framebuffer {
            unsafe { gl::DeleteFramebuffers(1, &self.framebuffer_id) }
            unsafe { gl::DeleteRenderbuffers(1, &self.depth_id) }
        }
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}

impl RawTexture {
    fn internal_new(
        backend: &mut Backend,
        dimensions: (u32, u32),
        data: *const c_void,
    ) -> Result<RawTexture, NewTextureError> {
        let (max_width, max_height) = backend.constants().max_texture_size;
        if (dimensions.0 == 0 || dimensions.1 == 0)
            || (dimensions.0 > max_width || dimensions.1 > max_height)
        {
            return Err(NewTextureError::InvalidTextureSize {
                width: dimensions.0,
                height: dimensions.1,
            });
        }

        let mut id = 0;
        unsafe {
            // SAFETY: `n` is one.
            gl::GenTextures(1, &mut id);
            backend.state.update_texture(id);

            // TODO: consider using `gl::CLAMP_TO_BORDER` with an invisible border instead.

            // SAFETY:
            // `gl::TEXTURE_2D` is a valid target
            // `gl::TEXTUREWRAP_(S|T)` and `gl::TEXTURE_(MIN|MAG)_FILTER` are valid `pname`
            // `gl::CLAMP_TO_EDGE` is a valid `param` for `gl::TEXTURE_WRAP_(S|T)`
            // `gl::NEAREST` is a valid `param` for `gl::TEXTURE_(MIN|MAG)_FILTER`
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

            // SAFETY:
            // `gl::TEXTURE_2D` is a valid `target`
            // `gl::UNSIGNED_BYTE` is a valid `type` constant
            // `width` and `height` are both in the range `0..=GL_MAX_TEXTURE_SIZE`
            // `gl::RGBA8` is a valid sized `internalformat`
            // `level` and `border` are 0
            // We never bind something to `GL_PIXEL_UNPACK_BUFFER`
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                dimensions.0 as _,
                dimensions.1 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data,
            );
        }

        Ok(Self {
            id,
            framebuffer_id: 0,
            depth_id: 0,
            dimensions,
            has_framebuffer: false,
        })
    }

    pub fn new(
        backend: &mut Backend,
        dimensions: (u32, u32),
    ) -> Result<RawTexture, NewTextureError> {
        Self::internal_new(backend, dimensions, ptr::null())
    }

    pub fn from_image(
        backend: &mut Backend,
        image: RgbaImage,
    ) -> Result<RawTexture, NewTextureError> {
        let dimensions = image.dimensions();
        // open gl presents images upside down,
        // we therefore flip it to get the desired output.
        let reversed_data: Vec<u8> = image
            .into_raw()
            .chunks(dimensions.0 as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        Self::internal_new(backend, dimensions, reversed_data.as_ptr() as *const _)
    }

    pub fn add_framebuffer(&mut self, backend: &mut Backend) {
        assert!(!self.has_framebuffer);
        let mut buffer = 0;
        let mut depth = 0;

        unsafe {
            // SAFETY: `n` is 1
            gl::GenFramebuffers(1, &mut buffer);

            backend.state.update_framebuffer(buffer);

            // SAFETY:
            // `gl::FRAMEBUFFER` is a valid `target`
            // We just bound `buffer` to `target` meaning that buffer is not zero
            // `gl::COLOR_ATTACHMENT0` is a valid `attachment`
            // `self.id` is a valid `texture` which supports the `level` zero.
            // `self.id` is a `gl::TEXTURE_2D`
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.id, 0);

            // SAFETY: `n` is 1
            gl::GenRenderbuffers(1, &mut depth);

            // SAFETY:
            // `target` is `gl::RENDERBUFFER`
            // `depth` was returned from `gl::GenRenderbuffers`
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth);

            // SAFETY:
            // `target` is `gl::RENDERBUFFER`
            // `width` and `height` in the range `0..=gl::MAX_RENDERBUFFER_SIZE`
            // `gl::DEPTH_COMPONENT16` is a depth-renderable format
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT16,
                self.dimensions.0 as _,
                self.dimensions.1 as _,
            );
            // check if GL is out of memory
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

            // SAFETY:
            // `gl::FRAMEBUFFER` is a valid `target`
            // We just bound `buffer` to `target` meaning that buffer is not zero
            // `gl::DEPTH_ATTACHMENT` is a valid `attachment`
            // the `renderbuffertarget` is `gl::RENDERBUFFER`
            // `depth` has type `gl::RENDERBUFFER` and was returned from `gl::GenRenderbuffers`
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                depth,
            );

            // SAFETY:
            // `gl::COLOR_ATTACHMENT0` is an accepted value
            // the current framebuffer is not the default
            // `n` is one
            // `gl::COLOR_ATTACHMENT0` has been added to the current framebuffer
            gl::DrawBuffers(1, &gl::COLOR_ATTACHMENT0);

            // ATTACHMENT COMPLETENESS:
            // the source object still exists and did not change its type
            // image size is not zero or greater than GL_MAX_FRAMEBUFFER_(WIDTH|HEIGHT)
            // no samples are attached
            // FRAMEBUFFER COMPLETENESS:
            // all attachments are ATTACHMENT COMPLETE
            // exactly one image is attached
            // the draw buffer has an image attached

            // SAFETY:
            // `gl::FRAMEBUFFER` is a valid `target`
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                bug!("incomplete framebuffer");
            }

            // SAFETY:
            // no undefined bit is set in `mask`
            // `glBegin` and `glEnd` are never used
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        self.depth_id = depth;
        self.framebuffer_id = buffer;

        self.has_framebuffer = true;
    }

    pub fn clone_as_target(previous: &Self, backend: &mut Backend) -> Self {
        let mut clone = Self::new(backend, previous.dimensions).unwrap_bug();
        clone.add_framebuffer(backend);
        backend.clear_color(clone.framebuffer_id, (0.0, 0.0, 0.0, 0.0));
        backend.draw(
            clone.framebuffer_id,
            previous.dimensions,
            previous,
            (0, 0),
            previous.dimensions,
            (0, 0),
            &DrawConfig::default(),
        );

        clone
    }
}
