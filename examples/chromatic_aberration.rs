//! A simple implementation of chromatic aberration.
//!
//! Press space to split and offset the 3 color channels.
use crow::{
    color,
    glutin::{
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
        window::WindowBuilder,
    },
    BlendMode, Context, DrawConfig, Texture,
};

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;
    let mut target_texture = Texture::new(&mut ctx, (100, 100))?;

    let mut offset = 0;
    ctx.run(move |ctx: &mut Context, surface: &mut _, events| {
        for event in events {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed
                            && input.virtual_keycode == Some(VirtualKeyCode::Space)
                        {
                            offset = 1;
                        } else if input.state == ElementState::Released
                            && input.virtual_keycode == Some(VirtualKeyCode::Space)
                        {
                            offset = 0;
                        }
                    }
                    _ => (),
                }
            }
        }

        ctx.clear_color(surface, (0.3, 0.3, 0.8, 1.0));
        ctx.clear_color(&mut target_texture, (0.0, 0.0, 0.0, 0.0));

        ctx.draw(
            &mut target_texture,
            &texture,
            (0 - offset, offset),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::RED,
                ..Default::default()
            },
        );
        ctx.draw(
            &mut target_texture,
            &texture,
            (0, 0),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::GREEN,
                ..Default::default()
            },
        );
        ctx.draw(
            &mut target_texture,
            &texture,
            (offset, 0 - offset),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::BLUE,
                ..Default::default()
            },
        );

        ctx.draw(
            surface,
            &target_texture,
            (100, 100),
            &DrawConfig {
                scale: (4, 4),
                ..Default::default()
            },
        );
        true
    })
}
