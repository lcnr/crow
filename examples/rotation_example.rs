//! A simple example drawing a texture.
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

    let texture = Texture::load(&mut ctx, "./textures/cat.png")?;

    let mut a = 0;

    event_loop.run(
        move |event: Event<()>, _window_target: _, control_flow: &mut ControlFlow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => ctx.window().request_redraw(),
            Event::RedrawRequested(_) => {
                let mut surface = ctx.surface();
                ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));
                ctx.draw(
                    &mut surface,
                    &texture,
                    (0, 0),
                    &DrawConfig {
                        rotation: a,
                        ..DrawConfig::default()
                    },
                );
                ctx.present(surface).unwrap();
                a += 3;
            }
            _ => (),
        },
    )
}
