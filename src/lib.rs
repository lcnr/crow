//! A pixel perfect 2D graphics library
//!
//! # Examples
//!
//! ```no_run
//! use crow::{
//!     glutin::{Event, WindowBuilder, WindowEvent},
//!     Context, DrawConfig, Texture,
//! };
//!
//! fn main() -> Result<(), crow::Error> {
//!     let mut ctx = Context::new(WindowBuilder::new())?;
//!
//!     let texture = Texture::load(&mut ctx, "path/to/texture.png")?;
//!     let mut surface = ctx.window_surface();
//!
//!     let mut fin = false;
//!     loop {
//!         ctx.event_loop().poll_events(|event| match event {
//!             Event::WindowEvent {
//!                 event: WindowEvent::CloseRequested,
//!                 ..
//!             } => fin = true,
//!             _ => (),
//!         });
//!
//!         ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));
//!         ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default());
//!
//!         ctx.finalize_frame()?;
//!
//!         if fin {
//!             break;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
// #![warn(missing_doc_code_examples)]
#![warn(
    deprecated_in_future,
    missing_debug_implementations,
    trivial_casts,
    unused_extern_crates,
    missing_docs,
    clippy::clone_on_ref_ptr,
    clippy::cargo_common_metadata,
    clippy::cast_lossless,
    clippy::checked_conversions,
    clippy::default_trait_access
)]

use std::{any, fmt, marker::PhantomData, path::Path, rc::Rc};

use static_assertions::assert_not_impl_any;

use glutin::event_loop::EventLoop;

use image::RgbaImage;

#[cfg(all(feature = "serde", not(feature = "serde1")))]
compile_error!("Tried using the feature `serde` directly, consider enabling `serde1` instead");

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

macro_rules! bug {
    ($msg:expr$(,)?) => ({
        panic!("{}\n\n    This might be a bug, consider filing an issue at https://github.com/lcnr/crow/issues/new", $msg)
    });
    ($fmt:expr, $($arg:tt)+) => ({
        panic!("{}\n\n    This might be a bug, consider filing an issue at https://github.com/lcnr/crow/issues/new", format_args!($fmt, $($arg)+))
    });
}

mod backend;
mod context;
mod error;

pub mod color;
pub mod target;

pub use error::*;
pub use glutin;
pub use image;

use backend::{tex::RawTexture, Backend};

trait UnwrapBug<T> {
    fn unwrap_bug(self) -> T;
}

impl<T, E: fmt::Debug> UnwrapBug<T> for Result<T, E> {
    fn unwrap_bug(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => bug!("unexpected internal error: {:?}", e),
        }
    }
}

#[derive(Clone, Copy)]
struct SkipDebug<T>(T);

impl<T> fmt::Debug for SkipDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipDebug<{}>", any::type_name::<T>())
    }
}

/// A trait implemented by types upon which can be drawn.
///
/// It is recommended to use the corresponding methods of `Context`
/// instead of calling the methods of this trait directly.
pub trait DrawTarget {
    /// Draws the `texture` onto `self`.
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    );

    /// Sets each pixel of `self` to `color`.
    fn receive_clear_color(&mut self, ctx: &mut Context, color: (f32, f32, f32, f32));

    /// Resets the depth buffer of `self` to `1.0`.
    fn receive_clear_depth(&mut self, ctx: &mut Context);

    /// Draws a line from `from` to `to`.
    fn receive_line(
        &mut self,
        ctx: &mut Context,
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    );

    /// Draws the bounding box of an axis-aligned rectangle specified by
    /// its `lower_left` and `upper_right` corner.
    ///
    /// In case `lower_left` is to the right or above `upper_right`, the two points will be flipped.
    fn receive_rectangle(
        &mut self,
        ctx: &mut Context,
        lower_left: (i32, i32),
        upper_right: (i32, i32),
        color: (f32, f32, f32, f32),
    );
}

impl<T: DrawTarget> DrawTarget for &mut T {
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) {
        <T>::receive_draw(self, ctx, texture, position, config)
    }

    fn receive_clear_color(&mut self, ctx: &mut Context, color: (f32, f32, f32, f32)) {
        <T>::receive_clear_color(self, ctx, color)
    }

    fn receive_clear_depth(&mut self, ctx: &mut Context) {
        <T>::receive_clear_depth(self, ctx)
    }

    fn receive_line(
        &mut self,
        ctx: &mut Context,
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        <T>::receive_line(self, ctx, from, to, color)
    }

    fn receive_rectangle(
        &mut self,
        ctx: &mut Context,
        lower_left: (i32, i32),
        upper_right: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        <T>::receive_rectangle(self, ctx, lower_left, upper_right, color)
    }
}

/// This is the core trait of this library.
pub trait Application {
    /// This function is called once per `frame`,
    /// returning `true` stops the application.
    fn frame(&mut self, ctx: &mut Context, surface: &mut WindowSurface) -> bool;

    /// This function is called after `frame` has returned `true`
    /// and can be used to cleanup ressources.
    fn shutdown(self);
}

/// A struct storing the global state which is used
/// for all operations which require access to the GPU.
///
/// # Examples
///
/// ```no_run
/// use crow::{
///     glutin::{Event, WindowBuilder, WindowEvent},
///     Context, DrawConfig, Texture,
/// };
///
/// fn main() -> Result<(), crow::Error> {
///     let mut ctx = Context::new(WindowBuilder::new())?;
///
///     let texture = Texture::load(&mut ctx, "path/to/texture.png")?;
///     let mut surface = ctx.window_surface();
///
///     let mut fin = false;
///     loop {
///         ctx.event_loop().poll_events(|event| match event {
///             Event::WindowEvent {
///                 event: WindowEvent::CloseRequested,
///                 ..
///             } => fin = true,
///             _ => (),
///         });
///
///         ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));
///         ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default());
///
///         ctx.finalize_frame()?;
///
///         if fin {
///             break;
///         }
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Context {
    backend: Backend,
    event_loop: Option<EventLoop<()>>,
}

assert_not_impl_any!(Context: Send, Sync, Clone);

/// A handle which can be used to draw to the window.
///
/// Created by calling `Context::window_surface()`.
#[derive(Debug)]
pub struct WindowSurface {
    _marker: PhantomData<*const ()>,
}

assert_not_impl_any!(WindowSurface: Send, Sync);

impl DrawTarget for WindowSurface {
    /// Draws `texture` to the window, to finish the frame, call [`Context::finalize_frame`].
    ///
    /// [`Context::finalize_frame`]: struct.Context.html#method.finalize_frame
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) {
        let dim = ctx.backend.window_dimensions();
        let dpi = ctx.backend.dpi_factor();
        ctx.backend.draw(
            0,
            dim,
            dpi,
            &texture.inner,
            texture.position,
            texture.size,
            position,
            config,
        )
    }

    fn receive_clear_color(&mut self, ctx: &mut Context, color: (f32, f32, f32, f32)) {
        ctx.backend.clear_color(0, color)
    }

    fn receive_clear_depth(&mut self, ctx: &mut Context) {
        ctx.backend.clear_depth(0)
    }

    fn receive_line(
        &mut self,
        ctx: &mut Context,
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        let dim = ctx.backend.window_dimensions();
        let dpi = ctx.backend.dpi_factor();
        ctx.backend.debug_draw(false, 0, dim, dpi, from, to, color)
    }

    fn receive_rectangle(
        &mut self,
        ctx: &mut Context,
        lower_left: (i32, i32),
        upper_right: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        let dim = ctx.backend.window_dimensions();
        let dpi = ctx.backend.dpi_factor();
        ctx.backend
            .debug_draw(true, 0, dim, dpi, lower_left, upper_right, color)
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
    inner: Rc<RawTexture>,
    position: (u32, u32),
    size: (u32, u32),
}

assert_not_impl_any!(Texture: Send, Sync);

impl Texture {
    fn from_raw(raw: RawTexture) -> Self {
        let size = raw.dimensions;

        Texture {
            inner: Rc::new(raw),
            position: (0, 0),
            size,
        }
    }

    /// Creates a new texture with the given `dimensions`.
    ///
    /// The content of the texture is undefined after its creation.
    pub fn new(ctx: &mut Context, dimensions: (u32, u32)) -> Result<Self, NewTextureError> {
        let raw = RawTexture::new(&mut ctx.backend, dimensions)?;

        Ok(Self::from_raw(raw))
    }

    /// Creates a new texture from the given `image`.
    pub fn from_image(ctx: &mut Context, image: RgbaImage) -> Result<Self, NewTextureError> {
        let raw = RawTexture::from_image(&mut ctx.backend, image)?;

        Ok(Self::from_raw(raw))
    }

    /// Loads a texture from an image located at `path`.
    pub fn load<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Texture, LoadTextureError> {
        let image = image::open(path).map_err(LoadTextureError::ImageError)?;

        let raw = RawTexture::from_image(&mut ctx.backend, image.to_rgba())?;

        Ok(Self::from_raw(raw))
    }

    /// Returns the part of `self` specified by `position` and `size` as a `Texture`.
    ///
    /// # Panics
    ///
    /// This function panics if part of the requested section would be outside of the original texture.
    pub fn get_section(&self, position: (u32, u32), size: (u32, u32)) -> Texture {
        assert!(
            position.0 + size.0 <= self.size.0,
            "invalid section width: {} + {} > {}",
            position.0,
            size.0,
            self.size.0
        );
        assert!(
            position.1 + size.1 <= self.size.1,
            "invalid section heigth: {} + {} > {}",
            position.1,
            size.1,
            self.size.1
        );

        Texture {
            inner: Rc::clone(&self.inner),
            position: (self.position.0 + position.0, self.position.1 + position.1),
            size,
        }
    }

    /// Returns the dimensions of this texture.
    pub fn dimensions(&self) -> (u32, u32) {
        self.size
    }

    /// Returns the width of this texture.
    pub fn width(&self) -> u32 {
        self.size.0
    }

    /// Returns the height of this texture.
    pub fn height(&self) -> u32 {
        self.size.1
    }

    fn prepare_as_draw_target<'a>(&'a mut self, ctx: &mut Context) -> &'a mut RawTexture {
        if self.position != (0, 0) || self.size != self.inner.dimensions {
            let mut inner = RawTexture::new(&mut ctx.backend, self.size).unwrap_bug();
            inner.add_framebuffer(&mut ctx.backend);
            ctx.backend.draw(
                inner.framebuffer_id,
                self.size,
                1,
                &self.inner,
                self.position,
                self.size,
                (0, 0),
                &DrawConfig::default(),
            );

            self.inner = Rc::new(inner);
        } else if let Some(inner) = Rc::get_mut(&mut self.inner) {
            if !inner.has_framebuffer {
                inner.add_framebuffer(&mut ctx.backend);
            }
        } else {
            self.inner = Rc::new(RawTexture::clone_as_target(&self.inner, &mut ctx.backend));
        }

        Rc::get_mut(&mut self.inner).unwrap()
    }

    /// Stores the current state of this `Texture` in an image.
    /// This function is fairly slow and should not be used carelessly.
    pub fn get_image_data(&self, ctx: &mut Context) -> RgbaImage {
        let _ = ctx;

        let data = ctx.backend.get_image_data(&self.inner);

        let (width, height) = self.inner.dimensions;
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
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) {
        let target = self.prepare_as_draw_target(ctx);

        ctx.backend.draw(
            target.framebuffer_id,
            target.dimensions,
            1,
            &texture.inner,
            texture.position,
            texture.size,
            position,
            config,
        )
    }

    fn receive_clear_color(&mut self, ctx: &mut Context, color: (f32, f32, f32, f32)) {
        let target = self.prepare_as_draw_target(ctx);
        ctx.backend.clear_color(target.framebuffer_id, color)
    }

    fn receive_clear_depth(&mut self, ctx: &mut Context) {
        let target = self.prepare_as_draw_target(ctx);
        ctx.backend.clear_depth(target.framebuffer_id)
    }

    fn receive_line(
        &mut self,
        ctx: &mut Context,
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        let target = self.prepare_as_draw_target(ctx);

        ctx.backend.debug_draw(
            false,
            target.framebuffer_id,
            target.dimensions,
            1,
            from,
            to,
            color,
        )
    }

    fn receive_rectangle(
        &mut self,
        ctx: &mut Context,
        lower_left: (i32, i32),
        upper_right: (i32, i32),
        color: (f32, f32, f32, f32),
    ) {
        let target = self.prepare_as_draw_target(ctx);

        ctx.backend.debug_draw(
            true,
            target.framebuffer_id,
            target.dimensions,
            1,
            lower_left,
            upper_right,
            color,
        )
    }
}

/// Used in `DrawConfig` to specify how
/// each pixel should be draw onto the target.
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// This struct has a hidden unstable field as it
/// should only be constructed using functional record update (FRU).
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct DrawConfig {
    /// The scale of the drawn texture in drawn pixels per source pixel.
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
    pub invert_color: bool,
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
            color_modulation: color::IDENTITY,
            invert_color: false,
            flip_vertically: false,
            flip_horizontally: false,
            blend_mode: BlendMode::default(),
            __non_exhaustive: (),
        }
    }
}
