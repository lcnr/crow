use std::{fs, io::ErrorKind, ops::Deref};

use glutin::WindowBuilder;

use image::RgbaImage;

use rand::prelude::*;

use crow::{Context, DrawConfig, ErrDontCare, Texture};

pub fn test(name: &str, f: fn(&mut Context) -> Result<RgbaImage, ErrDontCare>) -> Result<(), ()> {
    let mut ctx = Context::new(
        WindowBuilder::new()
            .with_dimensions(From::from((720, 480)))
            .with_visibility(false),
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

fn simple(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    a.clear_color(ctx, (1.0, 0.0, 0.0, 1.0))?;
    b.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;
    b.draw_to_texture(ctx, &mut a, (16, 16), &DrawConfig::default())?;

    Ok(a.get_image_data(&ctx))
}

fn color_modulation(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    a.clear_color(ctx, (1.0, 0.0, 0.0, 1.0))?;
    b.clear_color(ctx, (0.5, 0.0, 0.5, 1.0))?;
    b.draw_to_texture(
        ctx,
        &mut a,
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

    Ok(a.get_image_data(&ctx))
}

fn flip_vertically(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let big = Texture::new(ctx, (48, 16))?;
    let mut a = big.get_section((0, 0), (16, 16));
    let mut b = big.get_section((16, 0), (16, 16));
    let mut c = big.get_section((32, 0), (16, 16));

    a.clear_color(ctx, (1.0, 0.0, 0.0, 1.0))?;
    b.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;
    c.clear_color(ctx, (0.0, 0.0, 1.0, 1.0))?;

    b.draw_to_texture(ctx, &mut c, (0, 8), &DrawConfig::default())?;
    c.draw_to_texture(
        ctx,
        &mut a,
        (8, 0),
        &DrawConfig {
            flip_vertically: true,
            ..Default::default()
        },
    )?;

    Ok(a.get_image_data(&ctx))
}

fn section_drawing(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let mut target = Texture::new(ctx, (10, 10))?;
    target.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    source.draw_to_texture(ctx, &mut target, (3, 5), &DrawConfig::default())?;

    Ok(target.get_image_data(&ctx))
}

fn section_flipped(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let mut target = Texture::new(ctx, (10, 10))?;
    target.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (3, 2));

    source.draw_to_texture(
        ctx,
        &mut target,
        (3, 5),
        &DrawConfig {
            flip_vertically: true,
            flip_horizontally: true,
            ..Default::default()
        },
    )?;

    Ok(target.get_image_data(&ctx))
}

fn zero_section(ctx: &mut Context) -> Result<RgbaImage, ErrDontCare> {
    let mut target = Texture::new(ctx, (10, 10))?;
    target.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;

    let object = Texture::load(ctx, "textures/section_test.png")?;
    let source = object.get_section((3, 4), (0, 0));

    source.draw_to_texture(ctx, &mut target, (3, 5), &DrawConfig::default())?;

    Ok(target.get_image_data(&ctx))
}

#[derive(Default)]
struct TestRunner(
    Vec<(
        &'static str,
        fn(&mut Context) -> Result<RgbaImage, ErrDontCare>,
    )>,
);

impl TestRunner {
    fn add(&mut self, name: &'static str, f: fn(&mut Context) -> Result<RgbaImage, ErrDontCare>) {
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
    runner.add("section_flipped", section_flipped);
    runner.add("zero_section", zero_section);

    std::process::exit(runner.run())
}
