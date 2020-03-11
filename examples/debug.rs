//! A simple showcase of debug lines and rectangles.
use crow::{glutin::window::WindowBuilder, Context};

fn main() -> Result<(), crow::Error> {
    let ctx = Context::new(WindowBuilder::new())?;

    ctx.run(move |ctx: &mut Context, surface: &mut _, _| {
        ctx.clear_color(surface, (0.3, 0.3, 0.8, 1.0));

        ctx.debug_line(surface, (50, 50), (150, 100), (1.0, 0.0, 0.0, 1.0));
        ctx.debug_line(surface, (150, 200), (50, 150), (1.0, 0.0, 0.0, 1.0));

        ctx.debug_rectangle(surface, (50, 250), (150, 300), (1.0, 0.0, 0.0, 1.0));
        ctx.debug_rectangle(surface, (150, 400), (50, 350), (1.0, 0.0, 0.0, 1.0));
        true
    })
}
