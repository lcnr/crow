use std::{path::Path, rc::Rc};

use image::RgbaImage;

use crate::{
    backend::tex::RawTexture, Context, DrawConfig, DrawTarget, LoadTextureError, NewTextureError,
    Texture, UnwrapBug,
};

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
