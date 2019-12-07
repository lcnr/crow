use std::{any, fmt, path::Path};

use glium::{
    glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent},
    texture::{RawImage2d, Texture2d},
    Display, Frame, Program, Surface, VertexBuffer,
};

macro_rules! todo {
    ($($arg:tt)*) => ({
        eprint!("{}:{}: ", file!(), line!());
        eprintln!($($arg)*);
    })
}

mod draw;
mod shader;
mod vertex;

use vertex::Vertex;

/// An error in cases where dealing with errors is hard.
/// This will be slowly replaced by useful errors later on.
#[derive(Debug, Clone, Copy)]
pub struct ErrDontCare;

#[derive(Clone, Copy)]
struct SkipDebug<T>(T);

impl<T> fmt::Debug for SkipDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipDebug<{}>", any::type_name::<T>())
    }
}

#[derive(Debug)]
pub struct RenderingContext {
    display: SkipDebug<Display>,
    current_frame: SkipDebug<Frame>,
    events_loop: EventsLoop,
    default_program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
}

impl RenderingContext {
    /// Creates a new `GlobalContext`.
    pub fn new() -> Result<Self, ErrDontCare> {
        let events_loop = EventsLoop::new();
        let wb = WindowBuilder::new();
        let cb = ContextBuilder::new();
        let display = Display::new(wb, cb, &events_loop).unwrap();

        let vertex_buffer = vertex::initialize_vertex_buffer(&display)?;

        let default_program =
            Program::from_source(&display, shader::VERTEX, shader::FRAGMENT, None).map_err(
                |err| {
                    todo!("RenderingContext::new: {:?}", err);
                    ErrDontCare
                },
            )?;

        let current_frame = SkipDebug(display.draw());
        Ok(Self {
            display: SkipDebug(display),
            current_frame,
            events_loop,
            default_program,
            vertex_buffer,
        })
    }

    pub fn clone_texture(&mut self, texture: &Texture) -> Result<Texture, ErrDontCare> {
        let mut new = Texture {
            inner: Texture2d::empty(&self.display.0, texture.inner.width(), texture.inner.height()).map_err(
                |err| {
                    todo!("RenderingContext::clone_texture: {:?}", err);
                    ErrDontCare
                },
            )?
        };

        self.draw(&mut new, texture, (0, 0), &Default::default())?;
        Ok(new)
    }

    /// Draws the `texture` on top of the `target`.
    /// This permanently alters the `target`, in case
    /// the original is still required, consider cloning it first
    /// by calling `RenderingContext::clone_texture`.
    pub fn draw(
        &mut self,
        target: &mut Texture,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let dimensions = target.inner.dimensions();
        let mut surface = target.inner.as_surface();
        draw::draw(
            &mut surface,
            dimensions,
            &texture.inner,
            position,
            config,
            &self.default_program,
            &self.vertex_buffer,
        )
    }

    /// Directly draws the `texture` to the screen at the specified `position`.
    /// For this to be shown on screen, it is required to call
    /// `finalize_frame`.
    pub fn draw_to_screen(
        &mut self,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        draw::draw(
            &mut self.current_frame.0,
            self.display.0.get_framebuffer_dimensions(),
            &texture.inner,
            position,
            config,
            &self.default_program,
            &self.vertex_buffer,
        )
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, ErrDontCare> {
        let image = image::open(path)
            .map_err(|err| {
                todo!("RenderingContext::load_texture: {:?}", err);
                ErrDontCare
            })?
            .to_rgba();

        let image_dimensions = image.dimensions();

        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture2d = Texture2d::new(&self.display.0, image).map_err(|err| {
            todo!("RenderingContext::load_texture: {:?}", err);
            ErrDontCare
        })?;

        Ok(Texture { inner: texture2d })
    }

    pub fn game_loop(&mut self) -> Result<(), ErrDontCare> {
        let texture = self.load_texture("test.png")?;
        let copy = self.clone_texture(&texture)?;

        let mut closed = false;
        let mut t: f32 = 0.0;
        let mut now = std::time::Instant::now();
        let mut frames = 0;
        while !closed {
            frames += 1;
            t = t + 0.02;

            self.draw_to_screen(&texture, (t as i32, 17), &DrawConfig::default())?;
            self.draw_to_screen(&copy, (t as i32, 200), &DrawConfig::default())?;
            self.finalize_frame()?;

            // listing the events produced by application and waiting to be received
            self.events_loop.poll_events(|ev| match ev {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => closed = true,
                    _ => (),
                },
                _ => (),
            });
            if now.elapsed() > std::time::Duration::from_secs(1) {
                println!("fps: {}", frames);
                frames = 0;
                now = std::time::Instant::now();
            }
        }
        unimplemented!()
    }

    pub fn finalize_frame(&mut self) -> Result<(), ErrDontCare> {
        self.current_frame.0.set_finish().map_err(|err| {
            todo!("RenderingContext::finalize_frame: {:?}", err);
            ErrDontCare
        })?;

        self.current_frame = SkipDebug(self.display.0.draw());
        self.current_frame.0.clear_color(0.0, 0.0, 1.0, 1.0);
        Ok(())
    }
}

impl Drop for RenderingContext {
    fn drop(&mut self) {
        if let Err(err) = self.current_frame.0.set_finish() {
            panic!("GlobalContext::drop: set_finish failed: {:?}", err);
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    inner: Texture2d,
}

/// How exactly should a texture be drawn?
#[derive(Debug, Clone)]
pub struct DrawConfig {
    /// The scale of the drawn texture in pixels.
    pub scale: (u32, u32),
}

impl Default for DrawConfig {
    fn default() -> Self {
        Self { scale: (1, 1) }
    }
}
