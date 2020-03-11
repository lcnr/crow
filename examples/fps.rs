//! A showcase of `Context::set_framerate`.
//!
//! # Controls
//!
//! - `u`: set framerate to unlimited
//! - `d`: set framerate to 120
//! - `n`: set framerate to 60
//! - `h`: set framerate to 30
use crow::{
    glutin::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        window::WindowBuilder,
    },
    Context, DrawConfig, Texture, WindowSurface,
};

const MAX_OFFSET: i32 = 100;

fn main() -> Result<(), crow::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::max())
        .init();

    let mut ctx = Context::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;

    let mut offset = MAX_OFFSET;
    ctx.run(
        move |ctx: &mut Context, surface: &mut WindowSurface, events| {
            for event in events {
                if let Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        },
                    ..
                } = event
                {
                    match key {
                        VirtualKeyCode::U => ctx.set_framerate(0),
                        VirtualKeyCode::D => ctx.set_framerate(120),
                        VirtualKeyCode::N => ctx.set_framerate(60),
                        VirtualKeyCode::H => ctx.set_framerate(30),
                        _ => (),
                    }
                }
            }

            ctx.clear_color(surface, (0.4, 0.4, 0.8, 1.0));
            ctx.draw(
                surface,
                &texture,
                (100 + (MAX_OFFSET - offset).abs(), 150),
                &DrawConfig::default(),
            );

            offset = (offset + 1) % (MAX_OFFSET * 2);
            true
        },
    )
}
