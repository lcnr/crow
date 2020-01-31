# Crow

A simple and fairly efficient pixel based 2D graphics library. **crow** is designed to be easy to use and
should allow users to do nearly everything they want without requiring custom renderers or unsafe code.

## Examples

```rust
use crow::{
    glutin::{Event, EventsLoop, WindowBuilder, WindowEvent},
    Context, DrawConfig, Texture,
};

fn main() {
    let mut ctx = Context::new(WindowBuilder::new(), EventsLoop::new()).unwrap();

    let texture = Texture::load(&mut ctx, "path/to/texture.png").expect("Unable to load texture");
    let mut surface = ctx.window_surface();

    let mut fin = false;
    loop {
        ctx.events_loop().poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => fin = true,
            _ => (),
        });

        ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0)).unwrap();
        ctx.draw(&mut surface, &texture, (100, 150), &DrawConfig::default())
            .unwrap();

        ctx.finalize_frame().unwrap();

        if fin {
            break;
        }
    }
}
```

## Features

- [x] basic pixel perfect rendering
- [x] image scaling
- [x] depth
- [x] color modulation
  - [x] gray scale
  - [x] invert colors
- [x] implement screenshots + getting texture data
- [x] flip textures
- [x] subtextures (spritesheets)
- [ ] flip textures diagonally
- [x] different drawing modes
- [x] change draw to accept a generic target (allows for easy cameras/scaling etc etc)
- [ ] actual error handling
- [ ] custom renderer support
