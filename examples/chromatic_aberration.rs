use std::{thread, time::Duration};

use glutin::{
    ControlFlow, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowBuilder, WindowEvent,
};

use crow::{color, BlendMode, DrawConfig, ErrDontCare, GlobalContext, Texture};

fn inner() -> Result<(), ErrDontCare> {
    let mut context = GlobalContext::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut context, "./textures/player.png")?;

    let mut target_texture = Texture::new(&mut context, (100, 100))?;

    let mut fin = false;
    let mut offset = 0;
    loop {
        context.events_loop().poll_events(|event| match event {
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

        target_texture.clear_color(&mut context, (0.0, 0.0, 0.0, 0.0))?;

        texture.draw_to_texture(
            &mut context,
            &mut target_texture,
            (0 - offset, 0 + offset),
            &DrawConfig {
                blend_mode: BlendMode::Additive,
                color_modulation: color::RED,
                ..Default::default()
            },
        )?;
        texture
            .draw_to_texture(
                &mut context,
                &mut target_texture,
                (0, 0),
                &DrawConfig {
                    blend_mode: BlendMode::Additive,
                    color_modulation: color::GREEN,
                    ..Default::default()
                },
            )
            .unwrap();
        texture
            .draw_to_texture(
                &mut context,
                &mut target_texture,
                (0 + offset, 0 - offset),
                &DrawConfig {
                    blend_mode: BlendMode::Additive,
                    color_modulation: color::BLUE,
                    ..Default::default()
                },
            )
            .unwrap();

        target_texture
            .draw(
                &mut context,
                (100, 100),
                &DrawConfig {
                    scale: (4, 4),
                    ..Default::default()
                },
            )
            .unwrap();

        context.finalize_frame().unwrap();
        thread::sleep(Duration::from_millis(1000 / 30));

        if fin {
            break;
        }
    }

    Ok(())
}

fn main() {
    inner().unwrap();
}
