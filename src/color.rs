//! A collection of useful color matrices.

/// Converts an ordinary image into grayscale.
pub const GREYSCALE: [[f32; 4]; 4] = [
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Uses only the red and alpha channel of an image.
pub const RED: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Uses only the green and alpha channel of an image.
pub const GREEN: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Uses only the green blue and alpha channel of an image.
pub const BLUE: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
