# Crow

[![Documentation][di]][dl] [![Crates.io][ri]][rl] [![License: MIT][li]][ll]

[di]: https://docs.rs/crow/badge.svg
[dl]: https://docs.rs/crow

[ri]: https://img.shields.io/crates/v/crow.svg
[rl]: https://crates.io/crates/crow/

[li]: https://img.shields.io/badge/License-MIT-blue.svg
[ll]: ./LICENSE

**This crate is still early in development and both highly unstable and not yet feature complete**

A simple and fairly efficient pixel based 2D graphics library. **crow** is designed to be easy to use and
should allow users to do nearly everything they want without requiring custom renderers or unsafe code.

The most recent documentation can be found [here](https://docs.rs/crow).

The latest release can be viewed at the [0.7.2 tag](https://github.com/lcnr/crow/tree/v0.7.2).

You may also want to consider looking at [akari](https://github.com/lcnr/akari), a WIP showcase project.

This crate requires a GPU supporting OpenGL Version **3.3**.

## Examples

```rust
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

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;

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
                ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default());
                ctx.present(surface).unwrap();
            }
            _ => (),
        },
    )
}
```
