use std::{
    error,
    fmt::{self, Display, Formatter},
};

/// The super type of every error in this crate.
/// If this is used as a return type, the question mark operator can always be used.
#[derive(Debug)]
pub enum Error {
    /// Tried to create a texture with dimensions which are
    /// greater than the maximum allowed texture size or zero.
    InvalidTextureSize {
        /// The requested width.
        width: u32,
        /// The requested height.
        height: u32,
    },
    /// Error created by `image::load`.
    ImageError(image::ImageError),
    /// Error created by `glutin::ContextBuilder::build_windowed`.
    CreationError(glutin::CreationError),
    /// Error created by `glutin::ContextWrapper::make_current`
    /// or `glutin::ContextWrapper::swap_buffers`.
    ContextError(glutin::ContextError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTextureSize { width, height } => write!(
                f,
                "failed to create a texture of the given size: {}x{}",
                width, height
            ),
            Self::ImageError(err) => write!(f, "{}", err),
            Self::CreationError(err) => write!(f, "{}", err),
            Self::ContextError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug)]
/// The error returned by `Context::new`.
pub enum NewContextError {
    /// Error created by `glutin::ContextBuilder::build_windowed`.
    CreationError(glutin::CreationError),
    /// Error created by `glutin::ContextWrapper::make_current`.
    ContextError(glutin::ContextError),
}

impl Display for NewContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationError(err) => write!(f, "{}", err),
            Self::ContextError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for NewContextError {}

impl From<NewContextError> for Error {
    fn from(e: NewContextError) -> Self {
        match e {
            NewContextError::CreationError(e) => Error::CreationError(e),
            NewContextError::ContextError(e) => Error::ContextError(e),
        }
    }
}

/// The error returned by `Context::finalize_frame`.
#[derive(Debug)]
pub enum FinalizeError {
    /// Error created by `glutin::ContextWrapper::swap_buffers`.
    ContextError(glutin::ContextError),
}

impl Display for FinalizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for FinalizeError {}

impl From<FinalizeError> for Error {
    fn from(e: FinalizeError) -> Self {
        match e {
            FinalizeError::ContextError(e) => Error::ContextError(e),
        }
    }
}

/// The error returned by `Texture::load`.
#[derive(Debug)]
pub enum LoadTextureError {
    /// Tried to create a texture with dimensions which are
    /// greater than the maximum allowed texture size or zero.
    InvalidTextureSize {
        /// The requested width.
        width: u32,
        /// The requested height.
        height: u32,
    },
    /// Error created by `image::load`.
    ImageError(image::ImageError),
}

impl Display for LoadTextureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTextureSize { width, height } => write!(
                f,
                "failed to create a texture of the given size: {}x{}",
                width, height
            ),
            Self::ImageError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for LoadTextureError {}

impl From<LoadTextureError> for Error {
    fn from(e: LoadTextureError) -> Self {
        match e {
            LoadTextureError::InvalidTextureSize { width, height } => {
                Error::InvalidTextureSize { width, height }
            }
            LoadTextureError::ImageError(e) => Error::ImageError(e),
        }
    }
}

/// The error returned by `Texture::new`.
#[derive(Debug)]
pub enum NewTextureError {
    /// Tried to create a texture with dimensions which are
    /// greater than the maximum allowed texture size or zero.
    InvalidTextureSize {
        /// The requested width.
        width: u32,
        /// The requested height.
        height: u32,
    },
}

impl Display for NewTextureError {
    fn fmt<'a>(&'a self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTextureSize { width, height } => write!(
                f,
                "failed to create a texture of the given size: {}x{}",
                width, height
            ),
        }
    }
}

impl error::Error for NewTextureError {}

impl From<NewTextureError> for LoadTextureError {
    fn from(e: NewTextureError) -> Self {
        match e {
            NewTextureError::InvalidTextureSize { width, height } => {
                LoadTextureError::InvalidTextureSize { width, height }
            }
        }
    }
}

impl From<NewTextureError> for Error {
    fn from(e: NewTextureError) -> Self {
        match e {
            NewTextureError::InvalidTextureSize { width, height } => {
                Error::InvalidTextureSize { width, height }
            }
        }
    }
}
