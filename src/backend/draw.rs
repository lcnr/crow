use gl::types::*;

use crate::{
    backend::{tex::RawTexture, Backend},
    DrawConfig, ErrDontCare,
};

impl Backend {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        target_framebuffer: GLuint,
        target_dimensions: (u32, u32),
        object_texture: &RawTexture,
        object_texture_offset: (u32, u32),
        object_dimensions: (u32, u32),
        object_position: (i32, i32),
        draw_config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        unsafe {
            gl::Uniform2ui(
                self.uniforms.object_texture_dimensions,
                object_texture.dimensions.0,
                object_texture.dimensions.1,
            );
            gl::Uniform2ui(
                self.uniforms.object_texture_offset,
                object_texture_offset.0,
                object_texture_offset.1,
            );
            gl::Uniform2ui(
                self.uniforms.object_dimensions,
                object_dimensions.0,
                object_dimensions.1,
            );
            gl::Uniform2i(
                self.uniforms.object_position,
                object_position.0,
                object_position.1,
            );
            gl::Uniform2ui(
                self.uniforms.object_scale,
                draw_config.scale.0,
                draw_config.scale.1,
            );
            gl::UniformMatrix4fv(
                self.uniforms.color_modulation,
                1,
                gl::TRUE,
                &draw_config.color_modulation as *const _ as *const f32,
            );
            gl::Uniform1ui(self.uniforms.invert_color, draw_config.invert_colors as _);
            gl::Uniform1ui(
                self.uniforms.flip_vertically,
                draw_config.flip_vertically as _,
            );
            gl::Uniform1ui(
                self.uniforms.flip_horizontally,
                draw_config.flip_horizontally as _,
            );

            self.state.update_target_dimensions(target_dimensions);
            self.state.update_blend_mode(draw_config.blend_mode);
            self.state.update_depth(draw_config.depth);
            self.state.update_framebuffer(target_framebuffer);
            self.state.update_texture(object_texture.id);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }

        Ok(())
    }
}
