use crow::{
    glutin::{Event, EventsLoop, WindowBuilder, WindowEvent},
    Context, DrawConfig, Texture,
};

fn main() {
    let mut ctx = Context::new(WindowBuilder::new(), EventsLoop::new()).unwrap();

    let texture = Texture::load(&mut ctx, "./textures/player.png").expect("Unable to load texture");
    let mut surface = ctx.window_surface();

    let mut fin = false;
    loop {
        ctx.events_loop().poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => fin = true,
            _ => (),
        });

        ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0)).unwrap();
        ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default())
            .unwrap();

        ctx.finalize_frame().unwrap();

        if fin {
            break;
        }
    }
}
