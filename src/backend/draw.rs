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
        source_texture: &RawTexture,
        source_texture_offset: (u32, u32),
        source_dimensions: (u32, u32),
        source_position: (i32, i32),
        draw_config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let s = &mut self.state;
        s.update_program(self.program.id);
        s.update_vao(self.program.vao);
        s.update_blend_mode(draw_config.blend_mode);
        s.update_framebuffer(target_framebuffer);
        s.update_texture(source_texture.id);
        s.update_depth(draw_config.depth);

        s.update_color_modulation(draw_config.color_modulation);
        s.update_target_dimensions(target_dimensions);
        s.update_viewport_dimensions(target_dimensions);
        s.update_source_scale(draw_config.scale);
        s.update_source_texture_dimensions(source_texture.dimensions);
        s.update_source_texture_offset(source_texture_offset);
        s.update_source_position(source_position);
        s.update_source_dimensions(source_dimensions);
        s.update_invert_color(draw_config.invert_color);
        s.update_flip_vertically(draw_config.flip_vertically);
        s.update_flip_horizontally(draw_config.flip_horizontally);
        unsafe {
            // SAFETY:
            // `gl::TRIANGLE_STRIP` is an accepted value
            // `count` is positive
            // We never map the data store of a buffer object
            // No geometry shader is active
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }

        Ok(())
    }

    pub fn debug_draw(
        &mut self,
        rectangle: bool,
        target_framebuffer: GLuint,
        target_dimensions: (u32, u32),
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        let s = &mut self.state;
        s.update_program(self.debug_program.id);
        s.update_vao(self.debug_program.vao[rectangle as usize]);
        s.update_framebuffer(target_framebuffer);
        s.update_viewport_dimensions(target_dimensions);
        s.disable_depth();
        s.update_debug_color(color);
        let data = (
            (from.0 as f32 + 0.5) / target_dimensions.0 as f32 * 2.0 - 1.0,
            (from.1 as f32 + 0.5) / target_dimensions.1 as f32 * 2.0 - 1.0,
            (to.0 as f32 + 0.75) / target_dimensions.0 as f32 * 2.0 - 1.0,
            (to.1 as f32 + 0.75) / target_dimensions.1 as f32 * 2.0 - 1.0,
        );
        s.update_debug_start_end(data);
        unsafe {
            // SAFETY:
            // `gl::LINE_STRIP` is an accepted value
            // `count` is positive
            // We never map the data store of a buffer object
            // No geometry shader is active
            gl::DrawArrays(gl::LINE_STRIP, 0, if rectangle { 5 } else { 2 });
        }

        Ok(())
    }
}
