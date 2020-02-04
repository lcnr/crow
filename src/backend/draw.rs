use std::mem;

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
        let s = &mut self.state;
        s.update_program(self.program.id, self.program.vao);
        s.update_blend_mode(draw_config.blend_mode);
        s.update_framebuffer(target_framebuffer);
        s.update_texture(object_texture.id);
        s.update_depth(draw_config.depth);

        s.update_color_modulation(draw_config.color_modulation);
        s.update_target_dimensions(target_dimensions);
        s.update_viewport_dimensions(target_dimensions);
        s.update_object_scale(draw_config.scale);
        s.update_object_texture_dimensions(object_texture.dimensions);
        s.update_object_texture_offset(object_texture_offset);
        s.update_object_position(object_position);
        s.update_object_dimensions(object_dimensions);
        s.update_invert_color(draw_config.invert_color);
        s.update_flip_vertically(draw_config.flip_vertically);
        s.update_flip_horizontally(draw_config.flip_horizontally);
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }

        Ok(())
    }

    pub fn draw_line(
        &mut self,
        target_framebuffer: GLuint,
        target_dimensions: (u32, u32),
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        let s = &mut self.state;
        s.update_program(self.lines_program.id, self.lines_program.vao);
        s.update_framebuffer(target_framebuffer);
        s.update_viewport_dimensions(target_dimensions);
        s.disable_depth();
        unsafe {
            let data: [GLfloat; 4] = [
                (from.0 as f32 + 0.5) / target_dimensions.0 as f32 * 2.0 - 1.0,
                (from.1 as f32 + 0.5) / target_dimensions.1 as f32 * 2.0 - 1.0,
                (to.0 as f32 + 0.5) / target_dimensions.0 as f32 * 2.0 - 1.0,
                (to.1 as f32 + 0.5) / target_dimensions.1 as f32 * 2.0 - 1.0,
            ];
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                mem::size_of_val(&data) as _,
                &data as *const _ as *const _,
            );
            gl::Uniform4f(
                self.lines_program.color_uniform,
                color.0,
                color.1,
                color.2,
                color.3,
            );
            gl::DrawArrays(gl::LINES, 0, 2);
        }

        Ok(())
    }
}
