use std::{path::Path, ptr};

use gl::types::*;

use image::{ImageError, RgbaImage};

use crate::{backend::Backend, ErrDontCare, LoadTextureError};

#[derive(Debug)]
pub struct RawTexture {
    pub id: GLuint,
    pub frame_buffer_id: GLuint,
    pub depth_id: GLuint,
    pub dimensions: (u32, u32),
    pub is_framebuffer: bool,
}

impl Drop for RawTexture {
    fn drop(&mut self) {
        if self.is_framebuffer {
            unsafe { gl::DeleteFramebuffers(1, &self.frame_buffer_id as *const _) }
            unsafe { gl::DeleteRenderbuffers(1, &self.depth_id as *const _) }
        }
        unsafe { gl::DeleteTextures(1, &self.id as *const _) }
    }
}

impl RawTexture {
    pub fn new(backend: &mut Backend, dimensions: (u32, u32)) -> Result<RawTexture, ErrDontCare> {
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id as *mut _);
            backend.state.update_texture(id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                dimensions.0 as _,
                dimensions.1 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            Ok(Self {
                id,
                frame_buffer_id: 0,
                depth_id: 0,
                dimensions,
                is_framebuffer: false,
            })
        }
    }

    pub fn from_image(backend: &mut Backend, image: RgbaImage) -> Result<RawTexture, ErrDontCare> {
        let image_dimensions = image.dimensions();

        // open gl presents images upside down,
        // we therefore flip it to get the desired output.
        let reversed_data: Vec<u8> = image
            .into_raw()
            .chunks(image_dimensions.0 as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id as *mut _);
            backend.state.update_texture(id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                image_dimensions.0 as _,
                image_dimensions.1 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                reversed_data.as_ptr() as *const _,
            );

            Ok(Self {
                id,
                frame_buffer_id: 0,
                depth_id: 0,
                dimensions: image_dimensions,
                is_framebuffer: false,
            })
        }
    }

    pub fn load<P: AsRef<Path>>(
        backend: &mut Backend,
        path: P,
    ) -> Result<RawTexture, LoadTextureError> {
        let image = match image::open(path) {
            Ok(image) => image.to_rgba(),
            Err(ImageError::IoError(e)) => return Err(LoadTextureError::IoError(e)),
            Err(todo) => {
                eprintln!("Texture::load: {:?}", todo);
                return Err(LoadTextureError::Unspecified);
            }
        };

        Self::from_image(backend, image).map_err(|e| match e {
            ErrDontCare => LoadTextureError::Unspecified,
        })
    }

    pub fn add_framebuffer(&mut self, backend: &mut Backend) -> Result<(), ErrDontCare> {
        assert!(!self.is_framebuffer);
        self.is_framebuffer = true;
        let (depth, buffer) = frame_buffer(backend, self.id, self.dimensions);

        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        self.depth_id = depth;
        self.frame_buffer_id = buffer;

        Ok(())
    }

    pub fn clone_as_target(previous: &Self, backend: &mut Backend) -> Result<Self, ErrDontCare> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture as *mut _);
            backend.state.update_texture(texture);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                previous.dimensions.0 as _,
                previous.dimensions.1 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        }
        let (depth, buffer) = frame_buffer(backend, texture, previous.dimensions);
        // clear the new texture
        unsafe {
            let mut old = [1.0, 1.0, 1.0, 1.0];
            gl::GetFloatv(
                gl::COLOR_CLEAR_VALUE,
                &mut old as *mut [GLfloat; 4] as *mut GLfloat,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(old[0], old[1], old[2], old[3]);
        }
        backend.draw(
            buffer,
            previous.dimensions,
            previous,
            (0, 0),
            previous.dimensions,
            (0, 0),
            &Default::default(),
        )?;
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
        Ok(Self {
            id: texture,
            depth_id: depth,
            frame_buffer_id: buffer,
            dimensions: previous.dimensions,
            is_framebuffer: true,
        })
    }
}

/// this function does not reset the currently used framebuffer
fn frame_buffer(
    backend: &mut Backend,
    texture: GLuint,
    dimensions: (u32, u32),
) -> (GLuint, GLuint) {
    let mut buffer = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut buffer as *mut _);
    }
    backend.state.update_framebuffer(buffer);

    let mut depth = 0;
    unsafe {
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);

        gl::GenRenderbuffers(1, &mut depth as *mut _);
        gl::BindRenderbuffer(gl::RENDERBUFFER, depth);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            dimensions.0 as _,
            dimensions.1 as _,
        );
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth,
        );
        gl::DrawBuffers(1, &gl::COLOR_ATTACHMENT0 as *const _);
    }

    unsafe {
        assert_eq!(
            gl::CheckFramebufferStatus(gl::FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE
        );
    }
    (depth, buffer)
}
