//! A pixel perfect 2D graphics library
//!
//! # Examples
//!
//! ```no_run
//! use crow::{glutin::window::WindowBuilder, Context, DrawConfig, Texture, WindowSurface};
//!
//! fn main() -> Result<(), crow::Error> {
//!     let mut ctx = Context::new(WindowBuilder::new())?;
//!
//!     let texture = Texture::load(&mut ctx, "./textures/player.png")?;
//!
//!     ctx.run(move |ctx: &mut Context, surface: &mut WindowSurface, _| {
//!         ctx.clear_color(surface, (0.4, 0.4, 0.8, 1.0));
//!         ctx.draw(surface, &texture, (100, 150), &DrawConfig::default());
//!         true
//!     })
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

#[macro_use]
extern crate log;

use std::{any, fmt, marker::PhantomData, rc::Rc};

use static_assertions::assert_not_impl_any;


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
mod texture;

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

/// A struct storing the global state which is used
/// for all operations which require access to the GPU.
///
/// # Examples
///
/// ```no_run
/// use crow::{glutin::window::WindowBuilder, Context, DrawConfig, Texture, WindowSurface};
///
/// fn main() -> Result<(), crow::Error> {
///     let mut ctx = Context::new(WindowBuilder::new())?;
///
///     let texture = Texture::load(&mut ctx, "./textures/player.png")?;
///
///     ctx.run(move |ctx: &mut Context, surface: &mut WindowSurface, _| {
///         ctx.clear_color(surface, (0.4, 0.4, 0.8, 1.0));
///         ctx.draw(surface, &texture, (100, 150), &DrawConfig::default());
///         true
///     })
/// }
/// ```
#[derive(Debug)]
pub struct Context {
    backend: Backend,
}

assert_not_impl_any!(Context: Send, Sync, Clone);

/// A handle which can be used to draw to the window.
#[derive(Debug)]
pub struct WindowSurface {
    _marker: PhantomData<*const ()>,
}

assert_not_impl_any!(WindowSurface: Send, Sync, Clone);

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
