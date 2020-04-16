//! A simple implementation of chromatic aberration.
//!
//! Press space to split and offset the 3 color channels.
use crow::{
    color,
    glutin::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    BlendMode, Context, DrawConfig, Texture,
};

fn main() -> Result<(), crow::Error> {
    std::env::set_var("RUST_LOG", "warn");
    let event_loop = EventLoop::new();
    let mut ctx = Context::new(WindowBuilder::new(), &event_loop)?;

    let texture = Texture::load(&mut ctx, ".assets/textures/player.png")?;
    let mut target_texture = Texture::new(&mut ctx, (100, 100))?;

    let mut offset = 0;
    event_loop.run(
        move |event: Event<()>, _window_target: _, control_flow: &mut ControlFlow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if state == ElementState::Pressed {
                    offset = 1;
                } else {
                    offset = 0;
                }
            }
            Event::MainEventsCleared => ctx.window().request_redraw(),
            Event::RedrawRequested(_) => {
                let mut surface = ctx.surface();
                ctx.clear_color(&mut surface, (0.3, 0.3, 0.8, 1.0));
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
                    &mut surface,
                    &target_texture,
                    (100, 100),
                    &DrawConfig {
                        scale: (4, 4),
                        ..Default::default()
                    },
                );
                ctx.present(surface).unwrap();
            }
            _ => (),
        },
    )
}
