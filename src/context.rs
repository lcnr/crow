use std::{
    marker::PhantomData,
    mem,
    sync::atomic::{AtomicBool, Ordering},
};

use glutin::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use image::RgbaImage;

use crate::{
    backend::Backend, Context, DrawConfig, DrawTarget, FinalizeError, NewContextError, Texture,
    WindowSurface,
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Context {
    /// Creates a new `Context`. It is not possible to have more
    /// than one `Context` in a program.
    ///
    /// To create a new `Context` after a previous context was used,
    /// The previous context has to be dropped using the method
    /// `Context::unlock_unchecked()`. This is a workaround and
    /// will probably be fixed in a future release.
    pub fn new<T>(
        window: WindowBuilder,
        event_loop: &EventLoop<T>,
    ) -> Result<Self, NewContextError> {
        if INITIALIZED.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire) != Ok(false) {
            panic!("Tried to initialize a second Context");
        }

        let backend = Backend::initialize(window, &event_loop)?;
        let surface = Some(WindowSurface {
            _marker: PhantomData,
        });
        Ok(Self { backend, surface })
    }

    /// Returns the dimensions of the used window.
    pub fn window_dimensions(&self) -> (u32, u32) {
        self.backend.window_dimensions()
    }

    /// Returns the width of the used window.
    pub fn window_width(&self) -> u32 {
        self.window_dimensions().0
    }

    /// Returns the height of the used window.
    pub fn window_height(&self) -> u32 {
        self.window_dimensions().1
    }

    /// Sets the dimensions of the used window.
    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.backend.resize_window(width, height)
    }

    /// Returns the size of the biggest supported texture.
    ///
    /// Trying to create a texture with a size
    /// greater than `maximum_texture_size` results in an
    /// `InvalidTextureSize` error.
    ///
    /// ```rust, no_run
    /// use crow::{Context, glutin::{window::WindowBuilder, event_loop::EventLoop}};
    ///
    /// let mut ctx = Context::new(WindowBuilder::new(), &EventLoop::new()).unwrap();
    /// println!("maximum supported texture size: {:?}", ctx.maximum_texture_size());
    /// ```
    pub fn maximum_texture_size(&self) -> (u32, u32) {
        self.backend.constants().max_texture_size
    }

    /// Draws the `source` onto `target`.
    ///
    /// To draw to the window, use [`Context::window_surface`] as a target.
    ///
    /// [`Context::window_surface`]: struct.Context.html#method.window_surface
    pub fn draw<T>(
        &mut self,
        target: &mut T,
        source: &Texture,
        position: (i32, i32),
        config: &DrawConfig,
    ) where
        T: DrawTarget,
    {
        target.receive_draw(self, source, position, config)
    }

    /// Draws the a line going from `from` to `to` onto `target` with the given `color`.
    ///
    /// To draw this line to the window, use [`Context::window_surface`] as a target.
    ///
    /// [`Context::window_surface`]: struct.Context.html#method.window_surface
    pub fn debug_line<T>(
        &mut self,
        target: &mut T,
        from: (i32, i32),
        to: (i32, i32),
        color: (f32, f32, f32, f32),
    ) where
        T: DrawTarget,
    {
        target.receive_line(self, from, to, color)
    }

    /// Draws the bounding box of an axis-aligned rectangle specified by
    /// its `lower_left` and `upper_right` corner.
    ///
    /// In case `lower_left` is to the right or above `upper_right`, the two points will be flipped.
    ///
    /// To draw this rectangle to the window, use [`Context::window_surface`] as a target.
    ///
    /// [`Context::window_surface`]: struct.Context.html#method.window_surface
    pub fn debug_rectangle<T>(
        &mut self,
        target: &mut T,
        lower_left: (i32, i32),
        upper_right: (i32, i32),
        color: (f32, f32, f32, f32),
    ) where
        T: DrawTarget,
    {
        target.receive_rectangle(self, lower_left, upper_right, color)
    }

    /// Clears the color of the given [`DrawTarget`], setting each pixel to `color`
    ///
    /// [`DrawTarget`]: trait.DrawTarget.html
    pub fn clear_color<T>(&mut self, target: &mut T, color: (f32, f32, f32, f32))
    where
        T: DrawTarget,
    {
        target.receive_clear_color(self, color)
    }

    /// Resets the depth buffer of the given [`DrawTarget`] to `1.0`.
    ///
    /// [`DrawTarget`]: trait.DrawTarget.html
    pub fn clear_depth<T>(&mut self, target: &mut T)
    where
        T: DrawTarget,
    {
        target.receive_clear_depth(self)
    }

    /// Loads the current state of a [`DrawTarget`] into an image.
    ///
    /// [`DrawTarget`]: trait.DrawTarget.html
    pub fn image_data<T>(&mut self, image: &T) -> RgbaImage
    where
        T: DrawTarget,
    {
        image.get_image_data(self)
    }

    /// Returns the inner window.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crow::{
    ///     glutin::{event_loop::EventLoop, window::WindowBuilder},
    ///     Context,
    /// };
    ///
    /// let context = Context::new(
    ///     WindowBuilder::new().with_title("Starting"),
    ///     &EventLoop::new(),
    /// )?;
    ///
    /// context.window().set_title("Running");
    /// # Ok::<(), crow::Error>(())
    /// ```
    pub fn window(&self) -> &Window {
        self.backend.window()
    }

    /// Returns a handle to the window surface.
    ///
    /// This handle implements `DrawTarget` and can be used to draw to the window.
    ///
    /// Use `fn Context::present` to actually display the resulting image.
    pub fn surface(&mut self) -> WindowSurface {
        if let Some(surface) = self.surface.take() {
            surface
        } else {
            panic!("Called `Context::surface` while the previous surface is still in use");
        }
    }

    /// Presents the current frame to the screen.
    pub fn present(&mut self, surface: WindowSurface) -> Result<(), FinalizeError> {
        self.surface = Some(surface);
        self.backend.finalize_frame()
    }

    /// Drops this context while allowing the initialization of a new one afterwards.
    ///
    /// # Safety
    ///
    /// This method may lead to undefined behavior if a struct, for example a `Texture`, which was created using
    /// the current context, is used with the new context.
    pub unsafe fn unlock_unchecked(self) {
        mem::drop(self);

        let gl_error = gl::GetError();
        if gl_error != gl::NO_ERROR {
            bug!("unexpected error: {}", gl_error);
        }

        INITIALIZED.store(false, Ordering::Release);
    }
}

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

    fn get_image_data(&self, ctx: &mut Context) -> RgbaImage {
        let (width, height) = ctx.window_dimensions();

        let data = ctx.backend.take_screenshot((width, height));

        let reversed_data = data
            .chunks(width as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        RgbaImage::from_vec(width, height, reversed_data).unwrap()
    }
}
