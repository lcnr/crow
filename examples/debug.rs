//! A simple showcase of debug lines and rectangles.

use std::{thread, time::Duration};

use crow::{
    glutin::{Event, EventsLoop, WindowBuilder, WindowEvent},
    Context,
};

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(WindowBuilder::new(), EventsLoop::new())?;

    let mut surface = ctx.window_surface();

    let mut fin = false;
    loop {
        ctx.events_loop().poll_events(|event| {
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                fin = true
            }
        });
        ctx.clear_color(&mut surface, (0.3, 0.3, 0.8, 1.0));

        ctx.debug_line(&mut surface, (50, 50), (150, 100), (1.0, 0.0, 0.0, 1.0));
        ctx.debug_line(&mut surface, (150, 200), (50, 150), (1.0, 0.0, 0.0, 1.0));

        ctx.debug_rectangle(&mut surface, (50, 250), (150, 300), (1.0, 0.0, 0.0, 1.0));
        ctx.debug_rectangle(&mut surface, (150, 400), (50, 350), (1.0, 0.0, 0.0, 1.0));

        ctx.finalize_frame()?;
        thread::sleep(Duration::from_millis(1000 / 30));

        if fin {
            break;
        }
    }

    Ok(())
}
