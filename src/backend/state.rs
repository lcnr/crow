use gl::types::*;

use crate::{backend::shader::Uniforms, BlendMode};

#[derive(Debug)]
pub struct OpenGlState {
    uniforms: Uniforms,
    target_dimensions: (u32, u32),
    blend_mode: BlendMode,
    depth_active: bool,
    depth: f32,
    framebuffer: GLuint,
    texture: GLuint,
}

impl OpenGlState {
    pub fn new(uniforms: Uniforms, window_dimensions: (u32, u32)) -> Self {
        unsafe {
            let target_dimensions = window_dimensions;
            gl::Uniform2ui(
                uniforms.target_dimensions,
                target_dimensions.0,
                target_dimensions.1,
            );
            gl::Viewport(0, 0, target_dimensions.0 as _, target_dimensions.1 as _);

            let blend_mode = BlendMode::Alpha;
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let depth_active = false;
            gl::Disable(gl::DEPTH_TEST);
            let depth = 0.0;
            gl::Uniform1f(uniforms.depth, depth);

            let framebuffer = 0;
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            let texture = 0;
            gl::BindTexture(gl::TEXTURE_2D, texture);
            Self {
                uniforms,
                target_dimensions,
                blend_mode,
                depth_active,
                depth,
                framebuffer,
                texture,
            }
        }
    }

    pub fn update_target_dimensions(&mut self, target_dimensions: (u32, u32)) {
        if target_dimensions != self.target_dimensions {
            self.target_dimensions = target_dimensions;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.target_dimensions,
                    self.target_dimensions.0,
                    self.target_dimensions.1,
                );
                gl::Viewport(0, 0, target_dimensions.0 as _, target_dimensions.1 as _);
            }
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) {
        if blend_mode != self.blend_mode {
            self.blend_mode = blend_mode;
            unsafe {
                match self.blend_mode {
                    BlendMode::Alpha => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
                    BlendMode::Additive => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE),
                }
            }
        }
    }

    // we want to use the precise depth in the shader,
    // so checking for equality should be fine here.
    #[allow(clippy::float_cmp)]
    pub fn update_depth(&mut self, depth: Option<f32>) {
        unsafe {
            if let Some(depth) = depth {
                if !self.depth_active {
                    self.depth_active = true;
                    gl::Enable(gl::DEPTH_TEST);
                }

                if depth != self.depth {
                    self.depth = depth;
                    gl::Uniform1f(self.uniforms.depth, self.depth);
                }
            } else if self.depth_active {
                self.depth_active = false;
                gl::Disable(gl::DEPTH_TEST);
            }
        }
    }

    pub fn update_framebuffer(&mut self, framebuffer: GLuint) {
        if framebuffer != self.framebuffer {
            self.framebuffer = framebuffer;
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            }
        }
    }

    pub fn update_texture(&mut self, texture: GLuint) {
        if texture != self.texture {
            self.texture = texture;
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.texture);
            }
        }
    }
}
