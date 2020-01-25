use std::ops::{Deref, AddAssign};

use glutin::WindowBuilder;

use image::RgbaImage;

use crow::{ErrDontCare, GlobalContext, Texture, DrawConfig};

pub fn test(name: &str, f: fn(&mut GlobalContext) -> Result<RgbaImage, ErrDontCare>) -> Result<(), ()> {
    let mut ctx = GlobalContext::new(WindowBuilder::new().with_dimensions(From::from((720, 480))).with_visibility(false)).unwrap();

    let actual_image = match f(&mut ctx) {
        Ok(image) => image,
        Err(_e) => {
            eprintln!("TEST FAILED (runtime error): {}", name);
            return Err(())
        }  
    };
    
    unsafe {
        ctx.unlock_unchecked();
    }

    if actual_image.deref() != image::open(format!("tests/expected/{}.png", name)).unwrap().to_rgba().deref() {
        eprintln!("TEST FAILED (invalid return image): {}", name);
        actual_image.save(format!("tests/actual/{}.png", name)).unwrap();
        Err(())
    } else {
        Ok(())
    }
}

fn simple(ctx: &mut GlobalContext) -> Result<RgbaImage, ErrDontCare> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    a.clear_color(ctx, (1.0, 0.0, 0.0, 1.0))?;
    b.clear_color(ctx, (0.0, 1.0, 0.0, 1.0))?;
    b.draw_to_texture(ctx, &mut a, (16, 16), &DrawConfig::default())?;

    Ok(a.get_image_data(&ctx))
}

fn color_modulation(ctx: &mut GlobalContext) -> Result<RgbaImage, ErrDontCare> {
    let mut a = Texture::new(ctx, (32, 32))?;
    let mut b = Texture::new(ctx, (32, 32))?;
    a.clear_color(ctx, (1.0, 0.0, 0.0, 1.0))?;
    b.clear_color(ctx, (0.5, 0.0, 0.5, 1.0))?;
    b.draw_to_texture(ctx, &mut a, (16, 16), &DrawConfig {
        color_modulation: [
            [0.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        ..Default::default()
    })?;

    Ok(a.get_image_data(&ctx))
}

struct TestCounter(u32, u32);

impl AddAssign<Result<(), ()>> for TestCounter {
    fn add_assign(&mut self, rhs: Result<(), ()>) {
        self.1 += 1;
        if rhs.is_ok() {
            self.0 += 1;
        }
    }
}

#[test]
fn test_runner() {
    let mut counter = TestCounter(0, 0);
    counter += test("simple", simple);
    counter += test("color_modulation", color_modulation);

    if counter.0 != counter.1 {
        panic!("RUN FAILED: total: {}, success: {}, failure: {}", counter.1, counter.0, counter.1 - counter.0);
    }
}