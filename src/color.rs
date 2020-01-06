//! A collection of useful color matrices.

/// Converts an ordinary image into grayscale
pub const GREYSCALE: [[f32; 4]; 4] = [
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
