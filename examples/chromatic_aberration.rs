use std::{thread, time::Duration};

use glutin::{
    ControlFlow, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowBuilder, WindowEvent,
};

use crow::{color, BlendMode, DrawConfig, GlobalContext};

fn main() {
    let mut context = crow::GlobalContext::new(WindowBuilder::new()).unwrap();

    let texture = context
        .load_texture("./examples/textures/player.png")
        .unwrap();

    let mut target_texture = context.new_texture((100, 100)).unwrap();

    let mut fin = false;
    let mut offset;
    loop {
        offset = 0;
        context.events_loop().poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => fin = true,
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed
                        && input.virtual_keycode == Some(VirtualKeyCode::Space)
                    {
                        offset = 1;
                    }
                }
                _ => (),
            },
            _ => (),
        });

        context
            .clear_texture_color(&mut target_texture, (0.0, 0.0, 0.0, 0.0))
            .unwrap();

        context
            .draw_to_texture(
                &mut target_texture,
                &texture,
                (0 - offset, 0 + offset),
                &DrawConfig {
                    blend_mode: BlendMode::Additive,
                    color_modulation: color::RED,
                    ..Default::default()
                },
            )
            .unwrap();
        context
            .draw_to_texture(
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
        context
            .draw_to_texture(
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

        context
            .draw(
                &target_texture,
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
}
