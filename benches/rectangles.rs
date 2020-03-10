use std::{collections::VecDeque, mem};

use criterion::{criterion_group, criterion_main, Criterion};

use crow::{
    glutin::{EventsLoop, WindowBuilder},
    target::Scaled,
    Context, DrawConfig, DrawTarget, Texture,
};

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

const SCALE: u32 = 2;
const WINDOW_SIZE: (u32, u32) = (360, 240);

pub struct Rectangle {
    position: (i32, i32),
    size: (u32, u32),
    color: (f32, f32, f32),
}

fn rectangles(c: &mut Criterion) {
    let mut ctx = Context::new(
        WindowBuilder::new()
            .with_dimensions(From::from((WINDOW_SIZE.0 * SCALE, WINDOW_SIZE.1 * SCALE)))
            .with_visibility(false)
            .with_resizable(false),
        EventsLoop::new(),
    )
    .unwrap();

    let rectangle_vertical = Texture::load(&mut ctx, "textures/rectangle_vertical.png").unwrap();
    let rectangle_horizontal =
        Texture::load(&mut ctx, "textures/rectangle_horizontal.png").unwrap();

    let mut surface = Scaled::new(ctx.window_surface(), (SCALE, SCALE));

    let mut rng = XorShiftRng::from_seed(Default::default());

    let mut rectangles = VecDeque::new();
    rectangles.push_back(Rectangle {
        position: (-10, -10),
        size: (WINDOW_SIZE.0 * 3 / 5 + 10, WINDOW_SIZE.1 / 3 + 10),
        color: rng.gen(),
    });
    rectangles.push_back(Rectangle {
        position: (WINDOW_SIZE.0 as i32 * 2 / 5, WINDOW_SIZE.1 as i32 / 2),
        size: (150, 100),
        color: rng.gen(),
    });
    rectangles.push_back(Rectangle {
        position: (WINDOW_SIZE.0 as i32 * 3 / 5, WINDOW_SIZE.1 as i32 / 2),
        size: (50, 130),
        color: rng.gen(),
    });
    rectangles.push_back(Rectangle {
        position: (300, 200),
        size: (100, 300),
        color: rng.gen(),
    });

    c.bench_function("rectangles", |b| {
        b.iter(|| {
            ctx.events_loop().poll_events(mem::drop);

            ctx.clear_color(&mut surface, (0.3, 0.3, 0.8, 1.0));

            draw_rectangles(
                &rectangles,
                &rectangle_vertical,
                &rectangle_horizontal,
                &mut surface,
                &mut ctx,
            );

            ctx.finalize_frame();
        })
    });
}

fn mat((r, g, b): (f32, f32, f32)) -> [[f32; 4]; 4] {
    [
        [r, 0.0, 0.0, 0.0],
        [0.0, g, 0.0, 0.0],
        [0.0, 0.0, b, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn draw_rectangles(
    rectangles: &VecDeque<Rectangle>,
    vertical: &Texture,
    horizontal: &Texture,
    surface: &mut impl DrawTarget,
    ctx: &mut Context,
) {
    for rectangle in rectangles.iter() {
        let right_pos = rectangle.position.0 + rectangle.size.0 as i32 - vertical.width() as i32;
        let mut height = rectangle.size.1;

        while let Some(h) = height.checked_sub(vertical.height()) {
            height = h;

            ctx.draw(
                surface,
                vertical,
                (rectangle.position.0, height as i32 + rectangle.position.1),
                &DrawConfig {
                    color_modulation: mat(rectangle.color),
                    ..Default::default()
                },
            );

            ctx.draw(
                surface,
                vertical,
                (right_pos, height as i32 + rectangle.position.1),
                &DrawConfig {
                    color_modulation: mat(rectangle.color),
                    flip_horizontally: true,
                    ..Default::default()
                },
            );
        }

        let vertical_section =
            vertical.get_section((0, vertical.height() - height), (vertical.width(), height));
        ctx.draw(
            surface,
            &vertical_section,
            (rectangle.position.0, rectangle.position.1),
            &DrawConfig {
                color_modulation: mat(rectangle.color),
                ..Default::default()
            },
        );

        ctx.draw(
            surface,
            &vertical_section,
            (right_pos, rectangle.position.1),
            &DrawConfig {
                color_modulation: mat(rectangle.color),
                flip_horizontally: true,
                ..Default::default()
            },
        );

        let horizontal_height =
            rectangle.position.1 + rectangle.size.1 as i32 - horizontal.height() as i32;
        let mut horizontal_pos = rectangle.size.0;
        while let Some(p) = horizontal_pos.checked_sub(horizontal.width()) {
            horizontal_pos = p;
            ctx.draw(
                surface,
                horizontal,
                (
                    rectangle.position.0 + horizontal_pos as i32,
                    horizontal_height,
                ),
                &DrawConfig {
                    color_modulation: mat(rectangle.color),
                    ..Default::default()
                },
            );
            ctx.draw(
                surface,
                horizontal,
                (
                    rectangle.position.0 + horizontal_pos as i32,
                    rectangle.position.1,
                ),
                &DrawConfig {
                    color_modulation: mat(rectangle.color),
                    flip_vertically: true,
                    ..Default::default()
                },
            );
        }

        let horizontal_section = horizontal.get_section(
            (horizontal.width() - horizontal_pos, 0),
            (horizontal_pos, horizontal.height()),
        );
        ctx.draw(
            surface,
            &horizontal_section,
            (rectangle.position.0, horizontal_height),
            &DrawConfig {
                color_modulation: mat(rectangle.color),
                ..Default::default()
            },
        );
        ctx.draw(
            surface,
            &horizontal_section,
            (rectangle.position.0, rectangle.position.1),
            &DrawConfig {
                color_modulation: mat(rectangle.color),
                flip_vertically: true,
                ..Default::default()
            },
        );
    }
}

criterion_group!(benches, rectangles);
criterion_main!(benches);
