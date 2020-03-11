//! A simple example drawing a texture.
use crow::{glutin::window::WindowBuilder, Context, DrawConfig, Texture, WindowSurface};

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(WindowBuilder::new())?;

    let texture = Texture::load(&mut ctx, "./textures/player.png")?;

    ctx.run(move |ctx: &mut Context, surface: &mut WindowSurface, _| {
        ctx.clear_color(surface, (0.4, 0.4, 0.8, 1.0));
        ctx.draw(surface, &texture, (100, 150), &DrawConfig::default());
        true
    })
}
