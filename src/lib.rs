use std::{
    any, fmt,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use glutin::EventsLoop;

mod backend;
pub mod color;

use backend::{tex::RawTexture, Backend};

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

static SINGLETON: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct GlobalContext {
    backend: Backend,
}

impl GlobalContext {
    /// Creates a new `GlobalContext`.
    pub fn new() -> Result<Self, ErrDontCare> {
        if SINGLETON.compare_and_swap(false, true, Ordering::AcqRel) {
            panic!("Tried to initialize a second GlobalContext");
        }

        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new();

        let backend = Backend::initialize(window, events_loop)?;
        Ok(Self { backend })
    }

    /// Draws the `texture` on top of the `target`.
    /// This permanently alters the `target`, in case
    /// the original is still required, consider cloning the target first.
    pub fn draw_to_texture(
        &mut self,
        target: &mut Texture,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let target = if let Some(inner) = Rc::get_mut(&mut target.inner) {
            if !inner.is_framebuffer {
                *inner = RawTexture::clone_as_target(inner, &mut self.backend)?;
            }
            inner
        } else {
            target.inner = Rc::new(RawTexture::clone_as_target(
                &target.inner,
                &mut self.backend,
            )?);
            Rc::get_mut(&mut target.inner).unwrap()
        };

        self.backend.draw(
            target.frame_buffer_id,
            target.dimensions,
            &texture.inner,
            position,
            config,
        )
    }

    /// Directly draws the `texture` to the screen at the specified `position`.
    /// For this to be shown on screen, it is required to call
    /// `finalize_frame`.
    pub fn draw(
        &mut self,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let dim = self.backend.window_dimensions();
        self.backend.draw(0, dim, &texture.inner, position, config)
    }

    /// Loads a texture from an image located at `path`.
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture, ErrDontCare> {
        let raw = backend::tex::RawTexture::load(path)?;

        Ok(Texture {
            inner: Rc::new(raw),
        })
    }

    pub fn events_loop(&mut self) -> &mut EventsLoop {
        self.backend.events_loop()
    }

    /// Presents the current frame to the screen and prepares for the next frame.
    pub fn finalize_frame(&mut self) -> Result<(), ErrDontCare> {
        self.backend.finalize_frame()
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    inner: Rc<backend::tex::RawTexture>,
}

/// How exactly should a texture be drawn?
#[derive(Debug, Clone)]
pub struct DrawConfig {
    /// The scale of the drawn texture in pixels.
    pub scale: (u32, u32),
    /// The depth at which the texture should be drawn,
    /// pixels with a depth smaller than `depth` will not
    /// be overwritten.
    ///
    /// Draw calls with `depth >= 1.0` are ignored.
    pub depth: Option<f32>,
    /// Changes the color of the given pixel using matrix multiplication.
    pub color_modulation: [[f32; 4]; 4],
}

impl Default for DrawConfig {
    fn default() -> Self {
        Self {
            scale: (1, 1),
            depth: None,
            color_modulation: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn two_contexts() {
        let _a = GlobalContext::new();
        let _b = GlobalContext::new();
    }
}
