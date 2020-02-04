use gl::types::*;

use crate::{backend::shader::Uniforms, color, BlendMode};

#[derive(Debug)]
pub struct OpenGlState {
    uniforms: Uniforms,
    program: GLuint,
    target_dimensions: (u32, u32),
    viewport_dimensions: (u32, u32),
    blend_mode: BlendMode,
    depth_active: bool,
    depth: f32,
    framebuffer: GLuint,
    texture: GLuint,
    object_scale: (u32, u32),
    color_modulation: [[f32; 4]; 4],
    object_texture_dimensions: (u32, u32),
    object_texture_offset: (u32, u32),
    object_position: (i32, i32),
    object_dimensions: (u32, u32),
    invert_color: bool,
    flip_vertically: bool,
    flip_horizontally: bool,
}

impl OpenGlState {
    pub fn new(
        uniforms: Uniforms,
        (program, vao): (GLuint, GLuint),
        window_dimensions: (u32, u32),
    ) -> Self {
        unsafe {
            gl::UseProgram(program);
            gl::BindVertexArray(vao);

            let target_dimensions = window_dimensions;
            gl::Uniform2f(
                uniforms.target_dimensions,
                target_dimensions.0 as f32,
                target_dimensions.1 as f32,
            );

            let viewport_dimensions = window_dimensions;
            gl::Viewport(0, 0, viewport_dimensions.0 as _, viewport_dimensions.1 as _);

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

            let object_scale = (1, 1);
            gl::Uniform2ui(uniforms.object_scale, object_scale.0, object_scale.1);

            let color_modulation = color::IDENTITY;
            gl::UniformMatrix4fv(
                uniforms.color_modulation,
                1,
                gl::TRUE,
                &color_modulation as *const _ as *const f32,
            );

            let object_texture_dimensions = (128, 128);
            gl::Uniform2f(
                uniforms.object_texture_dimensions,
                object_texture_dimensions.0 as f32,
                object_texture_dimensions.1 as f32,
            );

            let object_texture_offset = (0, 0);
            gl::Uniform2ui(
                uniforms.object_texture_offset,
                object_texture_offset.0,
                object_texture_offset.1,
            );

            let object_position = (0, 0);
            gl::Uniform2f(
                uniforms.object_position,
                object_position.0 as f32,
                object_position.1 as f32,
            );

            let object_dimensions = (128, 128);
            gl::Uniform2ui(
                uniforms.object_dimensions,
                object_dimensions.0,
                object_dimensions.1,
            );

            let invert_color = false;
            gl::Uniform1ui(uniforms.invert_color, invert_color as _);

            let flip_vertically = false;
            gl::Uniform1ui(uniforms.flip_vertically, flip_vertically as _);

            let flip_horizontally = false;
            gl::Uniform1ui(uniforms.flip_horizontally, flip_horizontally as _);

            Self {
                uniforms,
                program,
                target_dimensions,
                viewport_dimensions,
                blend_mode,
                depth_active,
                depth,
                framebuffer,
                texture,
                object_scale,
                color_modulation,
                object_texture_dimensions,
                object_texture_offset,
                object_position,
                object_dimensions,
                invert_color,
                flip_vertically,
                flip_horizontally,
            }
        }
    }

    pub fn update_program(&mut self, program: GLuint, vao: GLuint) {
        if program != self.program {
            self.program = program;
            unsafe {
                gl::UseProgram(program);
                gl::BindVertexArray(vao);
            }
        }
    }

    pub fn update_target_dimensions(&mut self, target_dimensions: (u32, u32)) {
        if target_dimensions != self.target_dimensions {
            self.target_dimensions = target_dimensions;
            unsafe {
                gl::Uniform2f(
                    self.uniforms.target_dimensions,
                    self.target_dimensions.0 as f32,
                    self.target_dimensions.1 as f32,
                );
            }
        }
    }

    pub fn update_viewport_dimensions(&mut self, viewport_dimensions: (u32, u32)) {
        if viewport_dimensions != self.viewport_dimensions {
            self.viewport_dimensions = viewport_dimensions;
            unsafe {
                gl::Viewport(0, 0, viewport_dimensions.0 as _, viewport_dimensions.1 as _);
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

    pub fn disable_depth(&mut self) {
        if self.depth_active {
            self.depth_active = false;
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
        }
    }

    // we want to use the precise depth in the shader,
    // so checking for equality should be fine here.
    #[allow(clippy::float_cmp)]
    pub fn update_depth(&mut self, depth: Option<f32>) {
        if let Some(depth) = depth {
            unsafe {
                if !self.depth_active {
                    self.depth_active = true;
                    gl::Enable(gl::DEPTH_TEST);
                }

                if depth != self.depth {
                    self.depth = depth;
                    gl::Uniform1f(self.uniforms.depth, self.depth);
                }
            }
        } else {
            self.disable_depth()
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

    pub fn update_object_scale(&mut self, object_scale: (u32, u32)) {
        if object_scale != self.object_scale {
            self.object_scale = object_scale;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.object_scale,
                    self.object_scale.0,
                    self.object_scale.1,
                );
            }
        }
    }

    pub fn update_color_modulation(&mut self, color_modulation: [[f32; 4]; 4]) {
        if color_modulation != self.color_modulation {
            self.color_modulation = color_modulation;
            unsafe {
                gl::UniformMatrix4fv(
                    self.uniforms.color_modulation,
                    1,
                    gl::TRUE,
                    &self.color_modulation as *const _ as *const f32,
                )
            }
        }
    }

    pub fn update_object_texture_dimensions(&mut self, object_texture_dimensions: (u32, u32)) {
        if object_texture_dimensions != self.object_texture_dimensions {
            self.object_texture_dimensions = object_texture_dimensions;
            unsafe {
                gl::Uniform2f(
                    self.uniforms.object_texture_dimensions,
                    self.object_texture_dimensions.0 as f32,
                    self.object_texture_dimensions.1 as f32,
                );
            }
        }
    }

    pub fn update_object_texture_offset(&mut self, object_texture_offset: (u32, u32)) {
        if object_texture_offset != self.object_texture_offset {
            self.object_texture_offset = object_texture_offset;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.object_texture_offset,
                    self.object_texture_offset.0,
                    self.object_texture_offset.1,
                );
            }
        }
    }

    pub fn update_object_position(&mut self, object_position: (i32, i32)) {
        if object_position != self.object_position {
            self.object_position = object_position;
            unsafe {
                gl::Uniform2f(
                    self.uniforms.object_position,
                    self.object_position.0 as f32,
                    self.object_position.1 as f32,
                );
            }
        }
    }

    pub fn update_object_dimensions(&mut self, object_dimensions: (u32, u32)) {
        if object_dimensions != self.object_dimensions {
            self.object_dimensions = object_dimensions;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.object_dimensions,
                    self.object_dimensions.0,
                    self.object_dimensions.1,
                );
            }
        }
    }

    pub fn update_invert_color(&mut self, invert_color: bool) {
        if invert_color != self.invert_color {
            self.invert_color = invert_color;
            unsafe {
                gl::Uniform1ui(self.uniforms.invert_color, self.invert_color as _);
            }
        }
    }

    pub fn update_flip_vertically(&mut self, flip_vertically: bool) {
        if flip_vertically != self.flip_vertically {
            self.flip_vertically = flip_vertically;
            unsafe {
                gl::Uniform1ui(self.uniforms.flip_vertically, self.flip_vertically as _);
            }
        }
    }

    pub fn update_flip_horizontally(&mut self, flip_horizontally: bool) {
        if flip_horizontally != self.flip_horizontally {
            self.flip_horizontally = flip_horizontally;
            unsafe {
                gl::Uniform1ui(self.uniforms.flip_horizontally, self.flip_horizontally as _);
            }
        }
    }
}
