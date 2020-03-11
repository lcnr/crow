use std::collections::VecDeque;

use crow::{
    glutin::{dpi::LogicalSize, window::WindowBuilder},
    target::{Offset, Scaled},
    Context, DrawConfig, DrawTarget, Texture,
};

use rand::Rng;

const SCALE: u32 = 2;
const WINDOW_SIZE: (u32, u32) = (360, 240);

pub struct Rectangle {
    position: (i32, i32),
    size: (u32, u32),
    color: (f32, f32, f32),
}

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(
                WINDOW_SIZE.0 * SCALE,
                WINDOW_SIZE.1 * SCALE,
            ))
            .with_resizable(false),
    )?;

    let rectangle_vertical = Texture::load(&mut ctx, "textures/rectangle_vertical.png")?;
    let rectangle_horizontal = Texture::load(&mut ctx, "textures/rectangle_horizontal.png")?;

    let mut rng = rand::thread_rng();

    let mut rectangles = VecDeque::new();
    rectangles.push_back(Rectangle {
        position: (-10, -10),
        size: (WINDOW_SIZE.0 * 3 / 5 + 10, WINDOW_SIZE.1 / 3 + 10),
        color: rng.gen(),
    });
    rectangles.push_back(Rectangle {
        position: (WINDOW_SIZE.0 as i32 * 4 / 5, WINDOW_SIZE.1 as i32 / 2),
        size: (150, 100),
        color: rng.gen(),
    });

    let mut position = 0;

    let mut frames_to_next = 0;
    ctx.run(move |ctx: &mut Context, surface: &mut _, _| {
        let mut surface = Scaled::new(surface, (SCALE, SCALE));

        ctx.clear_color(&mut surface, (0.3, 0.3, 0.8, 1.0));

        if frames_to_next == 0 {
            frames_to_next = rng.gen_range(50, 170);
            rectangles.push_back(Rectangle {
                position: (
                    position + WINDOW_SIZE.0 as i32,
                    rng.gen_range(-40, WINDOW_SIZE.1 as i32),
                ),
                size: (rng.gen_range(40, 200), rng.gen_range(40, 200)),
                color: rng.gen(),
            })
        } else {
            frames_to_next -= 1;
        }

        position += 1;

        if let Some(first) = rectangles.front() {
            if (first.position.0 + first.size.0 as i32) < position as i32 {
                rectangles.pop_front();
            }
        }

        draw_rectangles(
            &rectangles,
            &rectangle_vertical,
            &rectangle_horizontal,
            &mut Offset::new(&mut surface, (position, 0)),
            ctx,
        );

        true
    })
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
