# Crow

[![Documentation][di]][dl] [![Crates.io][ri]][rl] [![License: MIT][li]][ll]

[di]: https://docs.rs/crow/badge.svg
[dl]: https://docs.rs/crow

[ri]: https://img.shields.io/crates/v/crow.svg
[rl]: https://crates.io/crates/crow/

[li]: https://img.shields.io/badge/License-MIT-blue.svg
[ll]: ./LICENSE

A simple and fairly efficient pixel based 2D graphics library. **crow** is designed to be easy to use and
should allow users to do nearly everything they want without requiring custom renderers or unsafe code.

The most recent documentation can be found [here](https://docs.rs/crow).

The latest release can be viewed in the [0.4.0 branch](https://github.com/lcnr/crow/tree/0.4.0).

You may also want to consider looking at [akari](https://github.com/lcnr/akari), a WIP showcase project.

This crate requires a GPU supporting OpenGL Version **3.3**.

## Examples

```rust
use crow::{
    glutin::{Event, WindowBuilder, WindowEvent},
    Context, DrawConfig, Texture,
};

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;
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

        ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));
        ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default());

        ctx.finalize_frame()?;

        if fin {
            break;
        }
    }

    Ok(())
}
```
