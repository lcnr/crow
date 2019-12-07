extern crate crow;

use glium::glutin::{Event, WindowEvent};

fn main() {
    let mut global_context = crow::GlobalContext::new().unwrap();

    let texture = global_context.load_texture("test.png").unwrap();
    let copy = global_context.clone_texture(&texture).unwrap();

    let mut closed = false;
    let mut t: f32 = 0.0;
    let mut now = std::time::Instant::now();
    let mut frames = 0;
    while !closed {
        frames += 1;
        t = t + 0.02;

        global_context
            .draw_to_screen(&texture, (t as i32, 17), &crow::DrawConfig::default())
            .unwrap();
        global_context
            .draw_to_screen(&copy, (t as i32, 200), &crow::DrawConfig::default())
            .unwrap();
        global_context.finalize_frame().unwrap();

        // listing the events produced by application and waiting to be received
        global_context.events_loop().poll_events(|ev| match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
        if now.elapsed() > std::time::Duration::from_secs(1) {
            println!("fps: {}", frames);
            frames = 0;
            now = std::time::Instant::now();
        }
    }
}
