use gl::types::*;

use crate::{
    backend::shader::{DebugUniforms, Uniforms},
    BlendMode,
};

fn update_blend_mode(blend_mode: BlendMode) {
    unsafe {
        // SAFETY:
        // `gl::SRC_ALPHA` is a valid `sfactor`
        // both `gl::ONE_MINUS_SRC_ALPHA` is a valid `dfactor`
        match blend_mode {
            BlendMode::Alpha => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
            BlendMode::Additive => gl::BlendFunc(gl::SRC_ALPHA, gl::ONE),
        }
    }
}
/// TODO: in case `update_program` fails, there might not be a current program object, meaning
/// that `glUniform` can error.
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
    source_rotation: i32,
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
            // SAFETY: i am the senate
            super::update_program(program);

            // SAFETY: vao was previously returned from `glGenVertexArrays`.
            gl::BindVertexArray(vao);

            let target_dimensions = window_dimensions;
            // SAFETY: `target_dimensions` is declared as a `vec2`
            gl::Uniform2f(
                uniforms.target_dimensions,
                target_dimensions.0 as f32,
                target_dimensions.1 as f32,
            );

            let viewport_dimensions = window_dimensions;
            // SAFETY: both `width` and `height` are positive
            gl::Viewport(0, 0, viewport_dimensions.0 as _, viewport_dimensions.1 as _);

            let blend_mode = BlendMode::Alpha;
            update_blend_mode(blend_mode);

            let depth_active = false;
            let depth = 0.0;

            // SAFETY: `gl::DEPTH_TEST` is a valid `cap`.
            gl::Disable(gl::DEPTH_TEST);
            // SAFETY: `depth` is declared as a `float`
            gl::Uniform1f(uniforms.depth, depth);

            let framebuffer = 0;
            // SAFETY:
            // `gl::FRAMEBUFFER` is a valid target
            // `framebuffer` was previously returned from `glGenFramebuffers`
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            let texture = 0;
            // SAFETY:
            // `gl::TEXTURE_2D` is a valid target
            // `texture` is 0, which represents the default texture (TODO: is this actually true)
            gl::BindTexture(gl::TEXTURE_2D, texture);
            assert_eq!(gl::NO_ERROR, gl::GetError());

            let source_scale = (1, 1);
            // SAFETY: `source_scale` is declared as a `uvec2`
            gl::Uniform2ui(uniforms.source_scale, source_scale.0, source_scale.1);

            let source_rotation = 0;
            // An angle of 0 means identity matrix
            // SAFETY: `source_rotation` is declared as a `mat2`
            let rot_mat: [[f32; 2]; 2] = [[1.0, 0.0], [0.0, 1.0]];
            gl::UniformMatrix2fv(uniforms.source_rotation, 1, gl::FALSE, rot_mat[0].as_ptr());

            // By default, all uniforms are 0
            let color_modulation = [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ];

            let source_texture_dimensions = (128, 128);
            // SAFETY: `source_texture_dimensions` is declared as a `vec2`
            gl::Uniform2f(
                uniforms.source_texture_dimensions,
                source_texture_dimensions.0 as f32,
                source_texture_dimensions.1 as f32,
            );

            let source_texture_offset = (0, 0);
            // SAFETY: `source_texture_offset` is declared as a `uvec2`
            gl::Uniform2ui(
                uniforms.source_texture_offset,
                source_texture_offset.0,
                source_texture_offset.1,
            );

            let source_position = (0, 0);
            // SAFETY: `source_position` is declared as a `vec2`
            gl::Uniform2f(
                uniforms.source_position,
                source_position.0 as f32,
                source_position.1 as f32,
            );

            let source_dimensions = (128, 128);
            // SAFETY: `source_dimensions` is declared as a `uvec2`
            gl::Uniform2ui(
                uniforms.source_dimensions,
                source_dimensions.0,
                source_dimensions.1,
            );

            let invert_color = false;
            // SAFETY: `invert_color` is declared as a `bool`
            gl::Uniform1ui(uniforms.invert_color, invert_color as _);

            let flip_vertically = false;
            // SAFETY: `flip_vertically` is declared as a `bool`
            gl::Uniform1ui(uniforms.flip_vertically, flip_vertically as _);

            let flip_horizontally = false;
            // SAFETY: `flip_horizontally` is declared as a `bool`
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
                source_rotation,
                color_modulation,
                source_texture_dimensions,
                source_texture_offset,
                source_position,
                source_dimensions,
                invert_color,
                flip_vertically,
                flip_horizontally,
                // set `debug_color` and `debug_start_end` to the default value
                debug_color: (0.0, 0.0, 0.0, 0.0),
                debug_start_end: (0.0, 0.0, 0.0, 0.0),
            }
        }
    }

    pub fn update_program(&mut self, program: GLuint) {
        if program != self.program {
            self.program = program;
            unsafe {
                // SAFETY: i am the senate
                super::update_program(self.program)
            }
        }
    }

    pub fn update_vao(&mut self, vao: GLuint) {
        if vao != self.vao {
            self.vao = vao;
            unsafe {
                // SAFETY: vao was previously returned from `glGenVertexArrays`.
                gl::BindVertexArray(vao);
            }
        }
    }

    pub fn update_target_dimensions(&mut self, target_dimensions: (u32, u32)) {
        if target_dimensions != self.target_dimensions {
            self.target_dimensions = target_dimensions;
            unsafe {
                // SAFETY:
                // TODO: in case `update_program` fails, there might not be a current program object.
                // `uniforms.target_dimensions` is declared as a `vec2`
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
                // SAFETY: both `width` and `height` are positive
                gl::Viewport(0, 0, viewport_dimensions.0 as _, viewport_dimensions.1 as _);
            }
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) {
        if blend_mode != self.blend_mode {
            self.blend_mode = blend_mode;
            update_blend_mode(self.blend_mode);
        }
    }

    pub fn disable_depth(&mut self) {
        if self.depth_active {
            self.depth_active = false;
            unsafe {
                // SAFETY: `gl::DEPTH_TEST` is a valid `cap`.
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
                    // SAFETY: `gl::DEPTH_TEST` is a valid `cap`.
                    gl::Enable(gl::DEPTH_TEST);
                }

                if depth != self.depth {
                    self.depth = depth;
                    // SAFETY: `depth` is declared as a `float`
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
                // SAFETY:
                // `gl::FRAMEBUFFER` is a valid target
                // `framebuffer` was previously returned from `glGenFramebuffers`
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            }
        }
    }

    pub fn update_texture(&mut self, texture: GLuint) {
        if texture != self.texture {
            self.texture = texture;
            unsafe {
                // SAFETY:
                // `gl::TEXTURE_2D` is a valid target
                // `self.texture` was created using `glGenTexture`
                //      and is only ever bound to `gl::TEXTURE_2D`
                gl::BindTexture(gl::TEXTURE_2D, self.texture);
            }
        }
    }

    pub fn update_source_scale(&mut self, source_scale: (u32, u32)) {
        if source_scale != self.source_scale {
            self.source_scale = source_scale;
            unsafe {
                // SAFETY: `source_scale` is declared as a `uvec2`
                gl::Uniform2ui(
                    self.uniforms.source_scale,
                    self.source_scale.0,
                    self.source_scale.1,
                );
            }
        }
    }

    pub fn update_source_rotation(&mut self, source_rotation: i32) {
        if source_rotation != self.source_rotation {
            // Build rotation matrices
            let angle = (source_rotation as f32).to_radians();
            let rot_mat: [[f32; 2]; 2] = [[angle.cos(), -angle.sin()], [angle.sin(), angle.cos()]];
            self.source_rotation = source_rotation;
            unsafe {
                gl::UniformMatrix2fv(
                    self.uniforms.source_rotation,
                    1,
                    gl::FALSE,
                    rot_mat[0].as_ptr(),
                );
            }
        }
    }

    pub fn update_color_modulation(&mut self, color_modulation: [[f32; 4]; 4]) {
        if color_modulation != self.color_modulation {
            self.color_modulation = color_modulation;
            let color_modulation: *const _ = &self.color_modulation;
            unsafe {
                // SAFETY:
                // `color_modulation` is declared as a `mat4`
                // `self.color_modulation` is an array of 16 `GLfloat`.
                gl::UniformMatrix4fv(
                    self.uniforms.color_modulation,
                    1,
                    gl::TRUE,
                    color_modulation.cast(),
                )
            }
        }
    }

    pub fn update_source_texture_dimensions(&mut self, source_texture_dimensions: (u32, u32)) {
        if source_texture_dimensions != self.source_texture_dimensions {
            self.source_texture_dimensions = source_texture_dimensions;
            unsafe {
                // SAFETY: `source_texture_dimensions` is declared as a `vec2`
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
                // SAFETY: `source_texture_offset` is declared as a `uvec2`
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
                // SAFETY: `source_position` is declared as a `vec2`
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
                // SAFETY: `source_dimensions` is declared as a `uvec2`
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
                // SAFETY: `invert_color` is declared as a `bool`
                gl::Uniform1ui(self.uniforms.invert_color, self.invert_color as _);
            }
        }
    }

    pub fn update_flip_vertically(&mut self, flip_vertically: bool) {
        if flip_vertically != self.flip_vertically {
            self.flip_vertically = flip_vertically;
            unsafe {
                // SAFETY: `flip_vertically` is declared as a `bool`
                gl::Uniform1ui(self.uniforms.flip_vertically, self.flip_vertically as _);
            }
        }
    }

    pub fn update_flip_horizontally(&mut self, flip_horizontally: bool) {
        if flip_horizontally != self.flip_horizontally {
            self.flip_horizontally = flip_horizontally;
            unsafe {
                // SAFETY: `flip_horizontally` is declared as a `bool`
                gl::Uniform1ui(self.uniforms.flip_horizontally, self.flip_horizontally as _);
            }
        }
    }

    pub fn update_debug_color(&mut self, debug_color: (f32, f32, f32, f32)) {
        if debug_color != self.debug_color {
            self.debug_color = debug_color;
            unsafe {
                // SAFETY: `line_color` is declared as `vec4`
                gl::Uniform4f(
                    self.debug_uniforms.line_color,
                    debug_color.0,
                    debug_color.1,
                    debug_color.2,
                    debug_color.3,
                );
            }
        }
    }
    pub fn update_debug_start_end(&mut self, debug_start_end: (f32, f32, f32, f32)) {
        if debug_start_end != self.debug_start_end {
            self.debug_start_end = debug_start_end;
        }
        unsafe {
            // SAFETY: `start_end` is declared as `vec4`
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
