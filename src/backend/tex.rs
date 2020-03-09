use std::{ffi::c_void, ptr};

use gl::types::*;

use image::RgbaImage;

use crate::{backend::Backend, ErrDontCare, NewTextureError, UnwrapBug};

#[derive(Debug)]
pub struct RawTexture {
    pub id: GLuint,
    pub framebuffer_id: GLuint,
    pub depth_id: GLuint,
    pub dimensions: (u32, u32),
    pub is_framebuffer: bool,
}

impl Drop for RawTexture {
    fn drop(&mut self) {
        // SAFETY: `n` is `1` for all functions
        if self.is_framebuffer {
            unsafe { gl::DeleteFramebuffers(1, &self.framebuffer_id as *const _) }
            unsafe { gl::DeleteRenderbuffers(1, &self.depth_id as *const _) }
        }
        unsafe { gl::DeleteTextures(1, &self.id as *const _) }
    }
}

impl RawTexture {
    fn internal_new(
        backend: &mut Backend,
        dimensions: (u32, u32),
        data: *const c_void,
    ) -> Result<RawTexture, NewTextureError> {
        let max_size = backend.constants().max_texture_size;
        if dimensions.0 > max_size || dimensions.1 > max_size {
            return Err(NewTextureError::InvalidTextureSize {
                width: dimensions.0,
                height: dimensions.1,
            });
        }

        let mut id = 0;
        unsafe {
            // SAFETY: `n` is one.
            gl::GenTextures(1, &mut id as *mut _);
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
            is_framebuffer: false,
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

    pub fn add_framebuffer(&mut self, backend: &mut Backend) -> Result<(), ErrDontCare> {
        assert!(!self.is_framebuffer);
        self.is_framebuffer = true;
        let mut buffer = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut buffer as *mut _);
        }
        backend.state.update_framebuffer(buffer);

        let mut depth = 0;
        unsafe {
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.id, 0);

            gl::GenRenderbuffers(1, &mut depth as *mut _);
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT,
                self.dimensions.0 as _,
                self.dimensions.1 as _,
            );
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                depth,
            );
            gl::DrawBuffers(1, &gl::COLOR_ATTACHMENT0 as *const _);

            assert_eq!(
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );

            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        self.depth_id = depth;
        self.framebuffer_id = buffer;

        Ok(())
    }

    pub fn clone_as_target(previous: &Self, backend: &mut Backend) -> Result<Self, ErrDontCare> {
        let mut clone = Self::new(backend, previous.dimensions).unwrap_bug();
        clone.add_framebuffer(backend)?;
        backend.clear_color(clone.framebuffer_id, (0.0, 0.0, 0.0, 0.0))?;
        backend.draw(
            clone.framebuffer_id,
            previous.dimensions,
            previous,
            (0, 0),
            previous.dimensions,
            (0, 0),
            &Default::default(),
        )?;

        Ok(clone)
    }
}
