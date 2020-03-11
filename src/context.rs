use std::{
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};

use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use image::RgbaImage;

use crate::{
    backend::Backend, Application, Context, DrawConfig, DrawTarget, NewContextError, Texture,
    UnwrapBug, WindowSurface,
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Context {
    /// Creates a new `Context`. It is not possible to have more
    /// than one `Context` in a program.
    pub fn new(window: WindowBuilder) -> Result<Self, NewContextError> {
        if INITIALIZED.compare_and_swap(false, true, Ordering::AcqRel) {
            panic!("Tried to initialize a second Context");
        }

        let event_loop = EventLoop::new();
        let backend = Backend::initialize(window, &event_loop)?;
        Ok(Self {
            backend,
            event_loop: Some(event_loop),
        })
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
    /// use crow::{Context, glutin::WindowBuilder};
    ///
    /// let mut ctx = Context::new(WindowBuilder::new()).unwrap();
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

    /// Stores the current state of the window in an image.
    /// This function is fairly slow and should not be used carelessly.
    ///
    /// It is currently not possible to screenshot a part of the screen.
    pub fn take_screenshot(&mut self) -> RgbaImage {
        let (width, height) = self.window_dimensions();

        let data = self.backend.take_screenshot((width, height));

        let reversed_data = data
            .chunks(width as usize * 4)
            .rev()
            .flat_map(|row| row.iter())
            .copied()
            .collect();

        RgbaImage::from_vec(width, height, reversed_data).unwrap()
    }

    /// Returns the inner window.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crow::{Context, glutin::WindowBuilder};
    ///
    /// let context = Context::new(WindowBuilder::new().with_title("Starting"))
    ///     .expect("unable to create a context");
    ///
    /// context.window().set_title("Running");
    /// ```
    pub fn window(&self) -> &Window {
        self.backend.window()
    }

    /// Hijacks the calling thread and runs the given `frame` in a loop.
    ///
    /// Since the closure is `'static`, it must be a `move` closure if it needs to
    /// access any data from the calling context.
    pub fn run<T>(mut self, application: T) -> !
    where
        T: Application + 'static,
    {
        let mut application = Some(application);
        let event_loop = self.event_loop.take().unwrap();
        let mut surface = WindowSurface {
            _marker: PhantomData,
        };

        let closure = move |event: Event<()>,
                            _window_target: &EventLoopWindowTarget<()>,
                            control_flow: &mut ControlFlow| {
            match event {
                // TODO: when is redraw requested called, is it better to use main events cleared
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(new_size) => {
                        let (width, height) = self.backend.convert_size(new_size);
                        self.backend.resize_window(width, height);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    if !application.as_mut().unwrap().frame(&mut self, &mut surface) {
                        *control_flow = ControlFlow::Exit;
                    } else {
                        self.backend.finalize_frame().unwrap_bug();
                    }
                }
                Event::LoopDestroyed => application.take().unwrap().shutdown(),
                _ => (),
            }
        };

        event_loop.run(closure)
    }
}
