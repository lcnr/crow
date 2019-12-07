use std::{any, fmt, path::Path};

use glium::{
    glutin::{ContextBuilder, EventsLoop, WindowBuilder},
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
pub struct GlobalContext {
    display: SkipDebug<Display>,
    current_frame: SkipDebug<Frame>,
    events_loop: EventsLoop,
    default_program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
}

impl GlobalContext {
    /// Creates a new context, using the given window `builder`.
    pub fn from_window_builder(builder: WindowBuilder) -> Result<Self, ErrDontCare> {
        let events_loop = EventsLoop::new();
        let cb = ContextBuilder::new();
        let display = Display::new(builder, cb, &events_loop).map_err(|err| {
            todo!("GlobalContext::from_window_builder: {:?}", err);
            ErrDontCare
        })?;

        let vertex_buffer = vertex::initialize_vertex_buffer(&display)?;

        let default_program =
            Program::from_source(&display, shader::VERTEX, shader::FRAGMENT, None).map_err(
                |err| {
                    todo!("GlobalContext::from_window_builder: {:?}", err);
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

    /// Creates a new `GlobalContext`.
    pub fn new() -> Result<Self, ErrDontCare> {
        Self::from_window_builder(WindowBuilder::new())
    }

    /// Creates a copy of the given `texture`.
    pub fn clone_texture(&mut self, texture: &Texture) -> Result<Texture, ErrDontCare> {
        let mut new = Texture {
            inner: Texture2d::empty(
                &self.display.0,
                texture.inner.width(),
                texture.inner.height(),
            )
            .map_err(|err| {
                todo!("GlobalContext::clone_texture: {:?}", err);
                ErrDontCare
            })?,
        };

        self.draw(&mut new, texture, (0, 0), &Default::default())?;
        Ok(new)
    }

    /// Draws the `texture` on top of the `target`.
    /// This permanently alters the `target`, in case
    /// the original is still required, consider cloning it first
    /// by calling `GlobalContext::clone_texture`.
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

    /// Loads a texture from an image located at `path`.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, ErrDontCare> {
        let image = image::open(path)
            .map_err(|err| {
                todo!("GlobalContext::load_texture: {:?}", err);
                ErrDontCare
            })?
            .to_rgba();

        let image_dimensions = image.dimensions();

        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture2d = Texture2d::new(&self.display.0, image).map_err(|err| {
            todo!("GlobalContext::load_texture: {:?}", err);
            ErrDontCare
        })?;

        Ok(Texture { inner: texture2d })
    }

    /// Returns the `EventsLoop` of this context.
    pub fn events_loop(&mut self) -> &mut EventsLoop {
        &mut self.events_loop
    }

    /// Presents the current frame to the screen and prepares for the next frame.
    pub fn finalize_frame(&mut self) -> Result<(), ErrDontCare> {
        self.current_frame.0.set_finish().map_err(|err| {
            todo!("GlobalContext::finalize_frame: {:?}", err);
            ErrDontCare
        })?;

        self.current_frame = SkipDebug(self.display.0.draw());
        self.current_frame.0.clear_color(0.0, 0.0, 1.0, 1.0);
        Ok(())
    }
}

impl Drop for GlobalContext {
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
