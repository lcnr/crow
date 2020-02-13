use gl::types::*;

use crate::{
    backend::shader::{DebugUniforms, Uniforms},
    color, BlendMode,
};

#[derive(Debug)]
pub struct OpenGlState {
    uniforms: Uniforms,
    debug_uniforms: DebugUniforms,
    program: GLuint,
    vao: GLuint,
    target_dimensions: (u32, u32),
    viewport_dimensions: (u32, u32),
    blend_mode: BlendMode,
    depth_active: bool,
    depth: f32,
    framebuffer: GLuint,
    texture: GLuint,
    source_scale: (u32, u32),
    color_modulation: [[f32; 4]; 4],
    source_texture_dimensions: (u32, u32),
    source_texture_offset: (u32, u32),
    source_position: (i32, i32),
    source_dimensions: (u32, u32),
    invert_color: bool,
    flip_vertically: bool,
    flip_horizontally: bool,
    debug_color: (f32, f32, f32, f32),
    debug_start_end: (f32, f32, f32, f32),
}

impl OpenGlState {
    pub fn new(
        uniforms: Uniforms,
        debug_uniforms: DebugUniforms,
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

            let source_scale = (1, 1);
            gl::Uniform2ui(uniforms.source_scale, source_scale.0, source_scale.1);

            let color_modulation = color::IDENTITY;
            gl::UniformMatrix4fv(
                uniforms.color_modulation,
                1,
                gl::TRUE,
                &color_modulation as *const _ as *const f32,
            );

            let source_texture_dimensions = (128, 128);
            gl::Uniform2f(
                uniforms.source_texture_dimensions,
                source_texture_dimensions.0 as f32,
                source_texture_dimensions.1 as f32,
            );

            let source_texture_offset = (0, 0);
            gl::Uniform2ui(
                uniforms.source_texture_offset,
                source_texture_offset.0,
                source_texture_offset.1,
            );

            let source_position = (0, 0);
            gl::Uniform2f(
                uniforms.source_position,
                source_position.0 as f32,
                source_position.1 as f32,
            );

            let source_dimensions = (128, 128);
            gl::Uniform2ui(
                uniforms.source_dimensions,
                source_dimensions.0,
                source_dimensions.1,
            );

            let invert_color = false;
            gl::Uniform1ui(uniforms.invert_color, invert_color as _);

            let flip_vertically = false;
            gl::Uniform1ui(uniforms.flip_vertically, flip_vertically as _);

            let flip_horizontally = false;
            gl::Uniform1ui(uniforms.flip_horizontally, flip_horizontally as _);

            Self {
                uniforms,
                vao,
                debug_uniforms,
                program,
                target_dimensions,
                viewport_dimensions,
                blend_mode,
                depth_active,
                depth,
                framebuffer,
                texture,
                source_scale,
                color_modulation,
                source_texture_dimensions,
                source_texture_offset,
                source_position,
                source_dimensions,
                invert_color,
                flip_vertically,
                flip_horizontally,
                debug_color: (0.0, 0.0, 0.0, 0.0),
                debug_start_end: (std::f32::MIN, std::f32::MIN, std::f32::MIN, std::f32::MIN),
            }
        }
    }

    pub fn update_program(&mut self, program: GLuint) {
        if program != self.program {
            self.program = program;
            unsafe {
                gl::UseProgram(program);
            }
        }
    }

    pub fn update_vao(&mut self, vao: GLuint) {
        if vao != self.vao {
            self.vao = vao;
            unsafe {
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

    pub fn update_source_scale(&mut self, source_scale: (u32, u32)) {
        if source_scale != self.source_scale {
            self.source_scale = source_scale;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.source_scale,
                    self.source_scale.0,
                    self.source_scale.1,
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

    pub fn update_source_texture_dimensions(&mut self, source_texture_dimensions: (u32, u32)) {
        if source_texture_dimensions != self.source_texture_dimensions {
            self.source_texture_dimensions = source_texture_dimensions;
            unsafe {
                gl::Uniform2f(
                    self.uniforms.source_texture_dimensions,
                    self.source_texture_dimensions.0 as f32,
                    self.source_texture_dimensions.1 as f32,
                );
            }
        }
    }

    pub fn update_source_texture_offset(&mut self, source_texture_offset: (u32, u32)) {
        if source_texture_offset != self.source_texture_offset {
            self.source_texture_offset = source_texture_offset;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.source_texture_offset,
                    self.source_texture_offset.0,
                    self.source_texture_offset.1,
                );
            }
        }
    }

    pub fn update_source_position(&mut self, source_position: (i32, i32)) {
        if source_position != self.source_position {
            self.source_position = source_position;
            unsafe {
                gl::Uniform2f(
                    self.uniforms.source_position,
                    self.source_position.0 as f32,
                    self.source_position.1 as f32,
                );
            }
        }
    }

    pub fn update_source_dimensions(&mut self, source_dimensions: (u32, u32)) {
        if source_dimensions != self.source_dimensions {
            self.source_dimensions = source_dimensions;
            unsafe {
                gl::Uniform2ui(
                    self.uniforms.source_dimensions,
                    self.source_dimensions.0,
                    self.source_dimensions.1,
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

    pub fn update_debug_color(&mut self, debug_color: (f32, f32, f32, f32)) {
        if debug_color != self.debug_color {
            self.debug_color = debug_color;
        }
        unsafe {
            gl::Uniform4f(
                self.debug_uniforms.color,
                debug_color.0,
                debug_color.1,
                debug_color.2,
                debug_color.3,
            );
        }
    }
    pub fn update_debug_start_end(&mut self, debug_start_end: (f32, f32, f32, f32)) {
        if debug_start_end != self.debug_start_end {
            self.debug_start_end = debug_start_end;
        }
        unsafe {
            gl::Uniform4f(
                self.debug_uniforms.start_end,
                debug_start_end.0,
                debug_start_end.1,
                debug_start_end.2,
                debug_start_end.3,
            );
        }
    }
}
