//! A pixel perfect 2D graphics library
use std::{
    any, fmt,
    marker::PhantomData,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use static_assertions::assert_not_impl_any;

use glutin::{EventsLoop, WindowBuilder};

use image::RgbaImage;

mod backend;
pub mod color;

pub use glutin;

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

/// A trait implemented by types upon which can be drawn.
pub trait DrawTarget {
    /// Draws the `texture` onto `self`.
    fn receive_draw_call(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare>;
}

impl<T: DrawTarget> DrawTarget for &mut T {
    fn receive_draw_call(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        <T>::receive_draw_call(self, ctx, texture, position, config)
    }
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Context {
    backend: Backend,
}

assert_not_impl_any!(Context: Send, Sync, Clone);

impl Context {
    /// Creates a new `Context`. It is not possible to have more
    /// than one `Context` at a time.
    ///
    /// To create a new `Context` a previous context was used,
    /// The previous context has to be dropped using the method
    /// `Context::unlock_unchecked()`. This is a workaround and
    /// will probably be fixed in a future release.
    pub fn new(window: WindowBuilder) -> Result<Self, ErrDontCare> {
        if INITIALIZED.compare_and_swap(false, true, Ordering::AcqRel) {
            panic!("Tried to initialize a second Context");
        }

        let backend = Backend::initialize(window)?;
        Ok(Self { backend })
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        self.backend.window_dimensions()
    }

    pub fn window_width(&self) -> u32 {
        self.window_dimensions().0
    }

    pub fn window_height(&self) -> u32 {
        self.window_dimensions().1
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.backend.resize_window(width, height)
    }

    /// Returns a handle to the window surface which can be used
    /// in [`Context::draw`] to draw to the window.
    ///
    /// [`Context::draw`]: struct.Context.html#method.draw
    pub fn window_surface(&self) -> WindowSurface {
        WindowSurface {
            _marker: PhantomData,
        }
    }

    /// Draws the `source` onto `target`. To draw to the window,
    /// use [`Context::window_surface`] as a target.
    ///
    /// [`Context::window_surface`]: struct.Context.html#method.window_surface
    pub fn draw<T>(
        &mut self,
        target: &mut T,
        source: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare>
    where
        T: DrawTarget,
    {
        target.receive_draw_call(self, source, position, config)
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

        let reversed_data = data
            .chunks(width as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        RgbaImage::from_vec(width, height, reversed_data).unwrap()
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

/// A handle which can be used to draw to the window.
#[derive(Debug)]
pub struct WindowSurface {
    _marker: PhantomData<*const ()>,
}

assert_not_impl_any!(WindowSurface: Send, Sync);

impl DrawTarget for WindowSurface {
    /// Draws `texture` to the window, to finish the frame, call [`Context::finalize_frame`].
    ///
    /// [`Context::finalize_frame`]: struct.Context.html#method.finalize_frame
    fn receive_draw_call(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let dim = ctx.backend.window_dimensions();
        ctx.backend.draw(
            0,
            dim,
            &texture.inner,
            texture.position,
            texture.size,
            position,
            config,
        )
    }
}

/// A two dimensional texture stored in video memory.
///
/// `Texture`s are copy-on-write, so cloning a texture is cheap
/// until the clone or the original is modified.
///
/// Transparency is supported.
#[derive(Debug, Clone)]
pub struct Texture {
    inner: Rc<backend::tex::RawTexture>,
    position: (u32, u32),
    size: (u32, u32),
}

assert_not_impl_any!(Texture: Send, Sync);

impl Texture {
    /// Creates a new texture with the given `dimensions`.
    ///
    /// The content of the texture is undefined after its creation.
    pub fn new(ctx: &mut Context, dimensions: (u32, u32)) -> Result<Self, ErrDontCare> {
        // ctx is only needed for safety
        let _ = ctx;
        let raw = backend::tex::RawTexture::new(dimensions)?;

        Ok(Texture {
            inner: Rc::new(raw),
            position: (0, 0),
            size: dimensions,
        })
    }

    /// Loads a texture from an image located at `path`.
    pub fn load<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Texture, ErrDontCare> {
        // ctx is only needed for safety
        let _ = ctx;

        let raw = backend::tex::RawTexture::load(path)?;

        let size = raw.dimensions;

        Ok(Texture {
            inner: Rc::new(raw),
            position: (0, 0),
            size,
        })
    }

    /// Returns the part of `self` specified by `position` and `size` as a `Texture`.
    ///
    /// # Panics
    ///
    /// This function panics if part of the requested section would be outside of the original texture.
    pub fn get_section(&self, position: (u32, u32), size: (u32, u32)) -> Texture {
        assert!(position.0 + size.0 <= self.size.0);
        assert!(position.1 + size.1 <= self.size.1);

        Texture {
            inner: self.inner.clone(),
            position: (self.position.0 + position.0, self.position.1 + position.1),
            size,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.0
    }

    pub fn height(&self) -> u32 {
        self.size.1
    }

    fn prepare_as_draw_target<'a>(
        &'a mut self,
        ctx: &mut Context,
    ) -> Result<&'a mut RawTexture, ErrDontCare> {
        if self.position != (0, 0) || self.size != self.inner.dimensions {
            let mut inner = RawTexture::new(self.size)?;
            inner.add_framebuffer()?;
            ctx.backend.draw(
                inner.frame_buffer_id,
                self.size,
                &self.inner,
                self.position,
                self.size,
                (0, 0),
                &Default::default(),
            )?;

            self.inner = Rc::new(inner);
        } else if let Some(inner) = Rc::get_mut(&mut self.inner) {
            if !inner.is_framebuffer {
                inner.add_framebuffer()?;
            }
        } else {
            self.inner = Rc::new(RawTexture::clone_as_target(&self.inner, &mut ctx.backend)?);
        }

        Rc::get_mut(&mut self.inner).ok_or_else(|| panic!("Rc::get_mut"))
    }

    /// Stores the current state of this `Texture` in an image.
    /// This function is fairly slow and should not be used carelessly.
    pub fn get_image_data(&self, ctx: &Context) -> RgbaImage {
        let _ = ctx;
        let (width, height) = self.inner.dimensions;

        // FIXME: this could theoretically overflow, leading to memory unsafety.
        let byte_count = 4 * width as usize * height as usize;
        let mut data: Vec<u8> = Vec::with_capacity(byte_count);

        unsafe {
            // FIXME: consider using glGetTextureImage even if it is only supported since OpenGL 4.5
            gl::BindTexture(gl::TEXTURE_2D, self.inner.id);
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

        let skip_above = height - (self.position.1 + self.size.1);
        let skip_vertical = self.position.0 * 4;
        let take_vertical = self.size.0 * 4;

        let image_data = data
            .chunks(width as usize * 4)
            .skip(skip_above as usize)
            .rev()
            .skip(self.position.1 as usize)
            .flat_map(|row| {
                row.iter()
                    .skip(skip_vertical as usize)
                    .take(take_vertical as usize)
            })
            .copied()
            .collect();

        RgbaImage::from_vec(self.size.0, self.size.1, image_data).unwrap()
    }

    /// Overwrites every pixel of `self` with the specified `color`
    pub fn clear_color(
        &mut self,
        ctx: &mut Context,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        let target = self.prepare_as_draw_target(ctx)?;
        ctx.backend.clear_texture_color(target, color)
    }

    /// Resets the depth buffer to `1.0` for every pixel.
    pub fn clear_depth(&mut self, ctx: &mut Context) -> Result<(), ErrDontCare> {
        let target = self.prepare_as_draw_target(ctx)?;
        ctx.backend.clear_texture_depth(target)
    }
}

impl DrawTarget for Texture {
    /// Draws the `texture` onto `self`.
    /// This permanently alters `self`, in case
    /// the original is still required,
    /// consider cloning this `Texture` first.
    ///
    /// It is recommended to call [`Context::draw`] instead of
    /// using this method directly.
    ///
    /// [`Context::draw`]: struct.Context.html#method.draw
    fn receive_draw_call(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        let target = self.prepare_as_draw_target(ctx)?;

        ctx.backend.draw(
            target.frame_buffer_id,
            target.dimensions,
            &texture.inner,
            texture.position,
            texture.size,
            position,
            config,
        )
    }
}

/// Used in `DrawConfig` to specify how
/// each pixel should be draw onto the target.
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
