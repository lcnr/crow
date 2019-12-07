use glium::{
    index::{NoIndices, PrimitiveType},
    uniform,
    uniforms::{MagnifySamplerFilter, Sampler, SamplerWrapFunction},
    DrawParameters, Program, Surface, Texture2d, VertexBuffer,
};

use crate::{vertex::Vertex, DrawConfig, ErrDontCare};

pub(crate) fn draw<T>(
    target: &mut T,
    target_dimensions: (u32, u32),
    texture: &Texture2d,
    object_position: (i32, i32),
    draw_config: &DrawConfig,
    program: &Program,
    vertex_buffer: &VertexBuffer<Vertex>,
) -> Result<(), ErrDontCare>
where
    T: Surface,
{
    let object = Sampler::new(texture)
        .wrap_function(SamplerWrapFunction::Clamp)
        .magnify_filter(MagnifySamplerFilter::Nearest);

    let uniforms = uniform! {
        target_dimensions: target_dimensions, // TODO: display.get_framebuffer_dimensions(),
        object_dimensions: texture.dimensions(),
        object_position: object_position,
        object_scale: draw_config.scale,
        object: object,
    };

    target
        .draw(
            vertex_buffer,
            &NoIndices(PrimitiveType::TriangleFan),
            program,
            &uniforms,
            &DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
        )
        .unwrap();

    Ok(())
}
