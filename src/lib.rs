use std::{
    any, fmt,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use glutin::{EventsLoop, WindowBuilder};

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

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct GlobalContext {
    backend: Backend,
}

impl GlobalContext {
    /// Creates a new `GlobalContext`.
    pub fn new(window: WindowBuilder) -> Result<Self, ErrDontCare> {
        if INITIALIZED.compare_and_swap(false, true, Ordering::AcqRel) {
            panic!("Tried to initialize a second GlobalContext");
        }

        let backend = Backend::initialize(window)?;
        Ok(Self { backend })
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        self.backend.window_dimensions()
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.backend.resize_window(width, height)
    }

    fn prepare_texture_as_draw_target<'a>(
        &mut self,
        tex: &'a mut Texture,
    ) -> Result<&'a mut RawTexture, ErrDontCare> {
        if let Some(inner) = Rc::get_mut(&mut tex.inner) {
            if !inner.is_framebuffer {
                inner.add_framebuffer()?;
            }
        } else {
            tex.inner = Rc::new(RawTexture::clone_as_target(&tex.inner, &mut self.backend)?);
        };

        Rc::get_mut(&mut tex.inner).ok_or_else(|| panic!("Rc::get_mut"))
    }

    pub fn clear_texture_depth(&mut self, texture: &mut Texture) -> Result<(), ErrDontCare> {
        let target = self.prepare_texture_as_draw_target(texture)?;
        self.backend.clear_texture_depth(target)
    }

    /// Overwrites every pixel of `texture` with `color`
    pub fn clear_texture_color(
        &mut self,
        texture: &mut Texture,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        let target = self.prepare_texture_as_draw_target(texture)?;
        self.backend.clear_texture_color(target, color)
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
        let target = self.prepare_texture_as_draw_target(target)?;

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

    /// Creates a new texture with the given `dimensions`.
    ///
    /// The content of the texture is undefined after its creation.
    pub fn new_texture(&mut self, dimensions: (u32, u32)) -> Result<Texture, ErrDontCare> {
        let raw = backend::tex::RawTexture::new(dimensions)?;

        Ok(Texture {
            inner: Rc::new(raw),
        })
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

impl Texture {
    pub fn dimensions(&self) -> (u32, u32) {
        self.inner.dimensions
    }
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
    /// If the red, green and blue values of the texture should be inverted.
    pub invert_colors: bool,
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
            invert_colors: false,
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
