use std::{thread, time::Duration};

use crow::{
    color,
    glutin::{ElementState, Event, VirtualKeyCode, WindowBuilder, WindowEvent},
    BlendMode, Context, DrawConfig, ErrDontCare, Texture,
};

fn main() -> Result<(), ErrDontCare> {
    let mut ctx = Context::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;

    let mut target_texture = Texture::new(&mut ctx, (100, 100))?;

    let mut fin = false;
    let mut offset = 0;
    loop {
        ctx.events_loop().poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => fin = true,
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
            },
            _ => (),
        });

        target_texture.clear_color(&mut ctx, (0.0, 0.0, 0.0, 0.0))?;

        ctx.draw(
            &mut target_texture,
            &texture,
            (0 - offset, 0 + offset),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::RED,
                ..Default::default()
            },
        )?;
        ctx.draw(
            &mut target_texture,
            &texture,
            (0, 0),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::GREEN,
                ..Default::default()
            },
        )
        .unwrap();
        ctx.draw(
            &mut target_texture,
            &texture,
            (0 + offset, 0 - offset),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::BLUE,
                ..Default::default()
            },
        )
        .unwrap();

        ctx.draw(
            &mut ctx.window_surface(),
            &target_texture,
            (100, 100),
            &DrawConfig {
                scale: (4, 4),
                ..Default::default()
            },
        )
        .unwrap();

        ctx.finalize_frame().unwrap();
        thread::sleep(Duration::from_millis(1000 / 30));

        if fin {
            break;
        }
    }

    Ok(())
}
