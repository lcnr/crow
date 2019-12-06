use glium::{
    glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent},
    texture::{RawImage2d, Texture2d},
    uniform,
    uniforms::{MagnifySamplerFilter, Sampler, SamplerWrapFunction},
    Display, DrawParameters, Surface,
};

use std::path::Path;

pub mod shader;
pub mod vertex;

use vertex::Vertex;

macro_rules! todo {
    ($($arg:tt)*) => ({
        eprint!("{}:{}: ", file!(), line!());
        eprintln!($($arg)*);
    })
}

/// An error in cases where dealing with errors is hard.
/// This will be slowly replaced by useful errors later on.
#[derive(Debug, Clone, Copy)]
pub struct ErrDontCare;

pub struct GlobalContext {}

impl GlobalContext {
    pub fn new() -> Result<Self, ErrDontCare> {
        let mut events_loop = EventsLoop::new();
        let wb = WindowBuilder::new();
        let cb = ContextBuilder::new();
        let display = Display::new(wb, cb, &events_loop).unwrap();

        let vertex1 = Vertex {
            position: [-0.5, -0.5],
            tex_coords: [0.0, 0.0],
        };
        let vertex2 = Vertex {
            position: [0.5, -0.5],
            tex_coords: [1.0, 0.0],
        };
        let vertex3 = Vertex {
            position: [0.5, 0.5],
            tex_coords: [1.0, 1.0],
        };
        let vertex4 = Vertex {
            position: [-0.5, 0.5],
            tex_coords: [0.0, 1.0],
        };
        let shape = vec![vertex1, vertex2, vertex3, vertex4];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

        let program =
            glium::Program::from_source(&display, shader::VERTEX, shader::FRAGMENT, None).unwrap();

        let texture = Self::load_image(&display, "./test.png");
        let sampler = Sampler::new(&texture)
            .wrap_function(SamplerWrapFunction::Clamp)
            .magnify_filter(MagnifySamplerFilter::Nearest);

        let mut closed = false;
        let mut t: f32 = 0.0;
        while !closed {
            t = (t + 0.0002) % 1.0;
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [ t , 0.0, 0.0, 1.0f32],
                ],
                tex: sampler,
            };

            target
                .draw(
                    &vertex_buffer,
                    &indices,
                    &program,
                    &uniforms,
                    &DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
            target.finish().unwrap();

            // listing the events produced by application and waiting to be received
            events_loop.poll_events(|ev| match ev {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => closed = true,
                    _ => (),
                },
                _ => (),
            });
        }
        unimplemented!()
    }

    fn load_image<P: AsRef<Path>>(display: &Display, path: P) -> Texture2d {
        let image = image::open(path).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        Texture2d::new(display, image).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Texture {}

#[derive(Debug, Clone)]
pub struct DrawContext {}

impl Texture {
    pub fn draw(&self, ctx: &DrawContext, target: &mut Texture) {
        todo!("Texture::draw");
        unimplemented!("{:?}, {:?}", ctx, target);
    }
}
