use gl::types::*;

use crate::{
    backend::{tex::RawTexture, Backend},
    DrawConfig, ErrDontCare,
};

impl Backend {
    pub fn draw(
        &mut self,
        target_framebuffer: GLuint,
        target_dimensions: (u32, u32),
        object_texture: &RawTexture,
        object_position: (i32, i32),
        draw_config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        unsafe {
            gl::Uniform2ui(
                self.uniforms.target_dimensions,
                target_dimensions.0,
                target_dimensions.1,
            );
            gl::Uniform2ui(
                self.uniforms.object_dimensions,
                object_texture.dimensions.0,
                object_texture.dimensions.1,
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
            gl::Uniform1ui(
                self.uniforms.invert_color,
                draw_config.invert_colors as _,
            );

            if let Some(depth) = draw_config.depth {
                gl::Enable(gl::DEPTH_TEST);
                gl::Uniform1f(self.uniforms.depth, depth);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }

            gl::BindTexture(gl::TEXTURE_2D, object_texture.id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, target_framebuffer);
            gl::Viewport(0, 0, target_dimensions.0 as _, target_dimensions.1 as _);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }

        Ok(())
    }
}
