//! A collect of useful draw modifiers.
use crate::{Context, DrawConfig, DrawTarget, ErrDontCare, Texture};

/// Can be used as a [`DrawTarget`] which modifies the scale of each draw call.
/// This should be identical to drawing to a temporary buffer and drawing this buffer
/// with the given `scale` onto the target.
///
/// [`DrawTarget`]: ../trait.DrawTarget.html
#[derive(Debug, Clone)]
pub struct Scaled<T> {
    inner: T,
    scale: (u32, u32),
}

impl<T: DrawTarget> Scaled<T> {
    pub fn new(inner: T, scale: (u32, u32)) -> Self {
        Self { inner, scale }
    }
}

impl<T: DrawTarget> DrawTarget for Scaled<T> {
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        self.inner.receive_draw(
            ctx,
            texture,
            (
                position.0 * self.scale.0 as i32,
                position.1 * self.scale.1 as i32,
            ),
            &DrawConfig {
                scale: (config.scale.0 * self.scale.0, config.scale.1 * self.scale.1),
                ..config.clone()
            },
        )
    }

    fn receive_clear_color(
        &mut self,
        ctx: &mut Context,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        self.inner.receive_clear_color(ctx, color)
    }
}

/// Can be used as a [`DrawTarget`] which offsets the `position` of each draw call by a given `offset`.
///
/// [`DrawTarget`]: ../trait.DrawTarget.html
#[derive(Debug, Clone)]
pub struct Offset<T> {
    inner: T,
    offset: (i32, i32),
}

impl<T: DrawTarget> Offset<T> {
    pub fn new(inner: T, offset: (i32, i32)) -> Self {
        Self { inner, offset }
    }
}

impl<T: DrawTarget> DrawTarget for Offset<T> {
    fn receive_draw(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) -> Result<(), ErrDontCare> {
        self.inner.receive_draw(
            ctx,
            texture,
            (position.0 + self.offset.0, position.1 + self.offset.1),
            config,
        )
    }

    fn receive_clear_color(
        &mut self,
        ctx: &mut Context,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ErrDontCare> {
        self.inner.receive_clear_color(ctx, color)
    }
}
