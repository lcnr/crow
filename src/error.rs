use std::io;

/// An error in cases where dealing with errors is hard.
/// This will be slowly replaced by useful error types later on.
#[derive(Debug, Clone, Copy)]
pub struct ErrDontCare;

/// The super type of every error in this crate.
/// If this is used as a return type, the question mark operator can always be used.
///
/// # Examples
///
/// ```rust, no_run
/// use crow::{Context, glutin::{WindowBuilder, EventsLoop}, Texture};
///
/// fn main() -> Result<(), crow::Error> {
///     let mut ctx = Context::new(WindowBuilder::new(), EventsLoop::new())?;
///
///     let image = Texture::load(&mut ctx, "this/path/does/not/exist.png")?;
///
///     ctx.draw(&mut ctx.window_surface(), &image, (0, 0), &Default::default())?;
///     
///     ctx.finalize_frame()?;
///     Ok(())
/// }
///
/// ```
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    /// An error condition which is not further specified,
    /// will be slowly replaced by more useful error kinds.
    Unspecified,
}

impl From<ErrDontCare> for Error {
    fn from(_: ErrDontCare) -> Self {
        Error::Unspecified
    }
}

/// An error returned by `Texture::load`.
#[derive(Debug)]
pub enum LoadTextureError {
    IoError(io::Error),
    Unspecified,
}

impl From<LoadTextureError> for Error {
    fn from(e: LoadTextureError) -> Self {
        match e {
            LoadTextureError::IoError(io) => Error::IoError(io),
            LoadTextureError::Unspecified => Error::Unspecified,
        }
    }
}
