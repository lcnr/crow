use std::{
    any, fmt,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use glutin::{EventsLoop, WindowBuilder};

use image::RgbaImage;

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
    /// Creates a new `GlobalContext`. It is not possible to have more
    /// than one `GlobalContext` at a time.
    ///
    /// To create a new `GlobalContext` a previous context was used,
    /// The previous context has to be dropped using the method
    /// `GlobalContext::unlock_unchecked()`. This is a workaround and
    /// will probably be fixed in a future release.
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

    /// Stores the current state of the given `texture` in an image.
    /// This function is fairly slow and should not be used carelessly.
    pub fn get_texture_data(&self, texture: &Texture) -> RgbaImage {
        let (width, height) = texture.dimensions();

        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = 4 * width as usize * height as usize;
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        unsafe {
            // FIXME: consider using glGetTextureImage even if it is only supported since OpenGL 4.5
            gl::BindTexture(gl::TEXTURE_2D, texture.inner.id);
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
            if gl::GetError() != gl::NO_ERROR {
                panic!("failed to take a screenshot");
            }
            data.set_len(byte_count);
        }

        RgbaImage::from_vec(width, height, backend::tex::flip_image_data(data, width)).unwrap()
    }

    /// Stores the current state of the window in an image.
    /// This function is fairly slow and should not be used carelessly.
    ///
    /// It is currently not possible to screenshot a part of the screen.
    pub fn take_screenshot(&self) -> RgbaImage {
        let (width, height) = self.window_dimensions();

        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = 4 * width as usize * height as usize;
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ReadPixels(
                0,
                0,
                width as _,
                height as _,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
            if gl::GetError() != gl::NO_ERROR {
                panic!("failed to take a screenshot");
            }
            data.set_len(byte_count);
        }

        RgbaImage::from_vec(width, height, backend::tex::flip_image_data(data, width)).unwrap()
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

    /// Drops this context while allowing the initialization of a new one afterwards.
    ///
    /// # Safety
    ///
    /// This method may lead to undefined behavior if a struct, for example a `Texture`, which was created using
    /// the current context, is used with the new context.
    pub unsafe fn unlock_unchecked(self) {
        // FIXME: Actually reason about the ordering. This should be correct afaik.
        INITIALIZED.store(false, Ordering::Release);
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

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum BlendMode {
    /// `src_alpha * src_color + (1.0 - src_alpha) * dst_color`
    Alpha,
    /// `src_alpha * src_color + 1.0 * dst_color`
    Additive,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Alpha
    }
}

/// How exactly should a texture be drawn?
///
/// This struct has a hidden unstable field, so it can only be constructed using FRU.
///
/// # Examples
///
/// ```rust
/// # #[allow(unused_variable)]
/// use crow::{DrawConfig, color};
///
/// let gray_scale = DrawConfig {
///     color_modulation: color::GREYSCALE,
///     ..Default::default()
/// };
///
/// let strange = DrawConfig {
///     scale: (2, 1),
///     flip_horizontally: true,
///     depth: Some(0.6),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct DrawConfig {
    /// The scale of the drawn texture in pixels.
    pub scale: (u32, u32),
    /// If the texture should be flipped on the y axis.
    pub flip_vertically: bool,
    /// If the texture should be flipped on the x axis.
    pub flip_horizontally: bool,
    /// The depth at which the texture should be drawn,
    /// pixels with a depth smaller than `depth` will not
    /// be overwritten.
    ///
    /// Draw calls with `depth >= 1.0` are ignored.
    pub depth: Option<f32>,
    /// Changes the color of the given pixel using matrix multiplication.
    pub color_modulation: [[f32; 4]; 4],
    /// If the red, green and blue color values of the texture should be inverted.
    pub invert_colors: bool,
    /// How the texture should be drawn on the target.
    pub blend_mode: BlendMode,
    // `#[non_exhaustive]` forbids FRU, so we use a hidden field instead,
    #[doc(hidden)]
    pub __non_exhaustive: (),
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
            flip_vertically: false,
            flip_horizontally: false,
            blend_mode: Default::default(),
            __non_exhaustive: (),
        }
    }
}
