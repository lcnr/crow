//! A collection of useful color matrices.

/// Converts an ordinary image into grayscale.
pub const GREYSCALE: [[f32; 4]; 4] = [
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Ignores all but the red part of the greyscale version of the image.
pub const RED: [[f32; 4]; 4] = [
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Ignores all but the green part of the greyscale version of the image.
pub const GREEN: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Ignores all but the blue part of the greyscale version of the image.
pub const BLUE: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0],
    [0.299, 0.587, 0.114, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
