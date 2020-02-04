use std::{fs, io::ErrorKind, ops::Deref};

use image::RgbaImage;

use rand::prelude::*;

use crow::{
    glutin::{EventsLoop, WindowBuilder},
    target::{Offset, Scaled},
    Context, DrawConfig, Texture,
};

pub fn test(name: &str, f: fn(&mut Context) -> Result<RgbaImage, crow::Error>) -> Result<(), ()> {
    let mut ctx = Context::new(
        WindowBuilder::new()
            .with_dimensions(From::from((720, 480)))
            .with_visibility(false),
        EventsLoop::new(),
    )
    .unwrap();

    let res = f(&mut ctx);
    unsafe {
        ctx.unlock_unchecked();
    }

    let actual_image = match res {
        Ok(image) => image,
        Err(_e) => {
            eprintln!("TEST FAILED (runtime error): {}", name);
            return Err(());
        }
    };

    let expected = if let Ok(image) = image::open(format!("tests/expected/{}.png", name)) {
        image.to_rgba()
    } else {
        eprintln!("TEST FAILED (expected image not found): {}", name);
        return Err(());
    };

    if actual_image.deref() != expected.deref() {
        eprintln!("TEST FAILED (invalid return image): {}", name);
        actual_image
            .save(format!("tests/actual/{}.png", name))
            .unwrap();
        Err(())
    } else {
        Ok(())
    }
}

fn simple(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    ctx.clear_color(&mut a, (1.0, 0.0, 0.0, 1.0))?;
    ctx.clear_color(&mut b, (0.0, 1.0, 0.0, 1.0))?;
    ctx.draw(&mut a, &b, (16, 16), &DrawConfig::default())?;

    Ok(a.get_image_data(ctx))
}

fn color_modulation(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    ctx.clear_color(&mut a, (1.0, 0.0, 0.0, 1.0))?;
    ctx.clear_color(&mut b, (0.5, 0.0, 0.5, 1.0))?;
    ctx.draw(
        &mut a,
        &b,
        (16, 16),
        &DrawConfig {
            color_modulation: [
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ..Default::default()
        },
    )?;

    Ok(a.get_image_data(ctx))
}

fn flip_vertically(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let big = Texture::new(ctx, (48, 16))?;
    let mut a = big.get_section((0, 0), (16, 16));
    let mut b = big.get_section((16, 0), (16, 16));
    let mut c = big.get_section((32, 0), (16, 16));

    ctx.clear_color(&mut a, (1.0, 0.0, 0.0, 1.0))?;
    ctx.clear_color(&mut b, (0.0, 1.0, 0.0, 1.0))?;
    ctx.clear_color(&mut c, (0.0, 0.0, 1.0, 1.0))?;

    ctx.draw(&mut c, &b, (0, 8), &DrawConfig::default())?;
    ctx.draw(
        &mut a,
        &c,
        (8, 0),
        &DrawConfig {
            flip_vertically: true,
            ..Default::default()
        },
    )?;

    Ok(a.get_image_data(ctx))
}

fn section_drawing(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    ctx.draw(&mut target, &source, (3, 5), &DrawConfig::default())?;

    Ok(target.get_image_data(ctx))
}

fn section_offset(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    ctx.draw(
        &mut Offset::new(&mut target, (-2, -3)),
        &source,
        (1, 2),
        &DrawConfig::default(),
    )?;

    Ok(target.get_image_data(ctx))
}

fn section_flipped(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    ctx.draw(
        &mut target,
        &source,
        (3, 5),
        &DrawConfig {
            flip_vertically: true,
            flip_horizontally: true,
            ..Default::default()
        },
    )?;

    Ok(target.get_image_data(ctx))
}

fn section_scaled(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    ctx.draw(
        &mut Scaled::new(&mut target, (2, 3)),
        &source,
        (1, 1),
        &DrawConfig {
            flip_vertically: true,
            flip_horizontally: true,
            ..Default::default()
        },
    )?;

    Ok(target.get_image_data(ctx))
}

fn zero_section(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (0, 0));

    ctx.draw(&mut target, &source, (3, 5), &DrawConfig::default())?;

    Ok(target.get_image_data(ctx))
}

fn debug_lines(ctx: &mut Context) -> Result<RgbaImage, crow::Error> {
    let mut target = Texture::new(ctx, (10, 10))?;
    ctx.clear_color(&mut target, (0.0, 1.0, 0.0, 1.0))?;

    ctx.draw_line(&mut target, (2, 2), (2, 10), (1.0, 0.0, 0.0, 1.0))?;
    ctx.draw_line(&mut target, (4, 9), (9, 9), (1.0, 0.0, 0.0, 1.0))?;

    Ok(target.get_image_data(ctx))
}

#[derive(Default)]
struct TestRunner(
    Vec<(
        &'static str,
        fn(&mut Context) -> Result<RgbaImage, crow::Error>,
    )>,
);

impl TestRunner {
    fn add(&mut self, name: &'static str, f: fn(&mut Context) -> Result<RgbaImage, crow::Error>) {
        self.0.push((name, f))
    }

    fn run(mut self) -> i32 {
        // randomize test order
        self.0.shuffle(&mut rand::thread_rng());

        let mut success = 0;
        let mut failure = 0;

        for (name, f) in self.0 {
            match test(name, f) {
                Ok(()) => success += 1,
                Err(()) => failure += 1,
            }
        }

        if failure > 0 {
            println!(
                "RUN FAILED: total: {}, success: {}, failure: {}",
                success + failure,
                success,
                failure
            );
            1
        } else {
            println!(
                "RUN SUCCESS: total: {}, success: {}, failure: {}",
                success + failure,
                success,
                failure
            );
            0
        }
    }
}

fn main() {
    fs::remove_dir_all("tests/actual")
        .or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .expect("unable to remove 'tests/actual'");

    fs::create_dir("tests/actual").expect("unable to create 'tests/actual'");

    let mut runner = TestRunner::default();
    runner.add("simple", simple);
    runner.add("color_modulation", color_modulation);
    runner.add("flip_vertically", flip_vertically);
    runner.add("section_drawing", section_drawing);
    runner.add("section_offset", section_offset);
    runner.add("section_flipped", section_flipped);
    runner.add("section_scaled", section_scaled);
    runner.add("zero_section", zero_section);
    runner.add("debug_lines", debug_lines);

    std::process::exit(runner.run())
}
