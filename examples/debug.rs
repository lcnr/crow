//! A simple showcase of debug lines and rectangles.
use crow::{
    glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    Context, DrawConfig, Texture,
};

fn main() -> Result<(), crow::Error> {
    let event_loop = EventLoop::new();
    let mut ctx = Context::new(WindowBuilder::new(), &event_loop)?;

    event_loop.run(
        move |event: Event<()>, _window_target: _, control_flow: &mut ControlFlow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => ctx.window().request_redraw(),
            Event::RedrawRequested(_) => {
                let mut surface = ctx.surface();
                ctx.clear_color(&mut surface, (0.3, 0.3, 0.8, 1.0));

                ctx.debug_line(&mut surface, (50, 50), (150, 100), (1.0, 0.0, 0.0, 1.0));
                ctx.debug_line(&mut surface, (150, 200), (50, 150), (1.0, 0.0, 0.0, 1.0));

                ctx.debug_rectangle(&mut surface, (50, 250), (150, 300), (1.0, 0.0, 0.0, 1.0));
                ctx.debug_rectangle(&mut surface, (150, 400), (50, 350), (1.0, 0.0, 0.0, 1.0));
                ctx.present(surface).unwrap();
            }
            _ => (),
        },
    )
}
