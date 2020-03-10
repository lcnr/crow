use criterion::{criterion_group, criterion_main, Criterion};

use crow::{
    glutin::{EventsLoop, WindowBuilder},
    Context,
};

fn debug_lines(c: &mut Criterion) {
    let mut ctx = Context::new(
        WindowBuilder::new()
            .with_dimensions(From::from((720, 480)))
            .with_visibility(false),
        EventsLoop::new(),
    )
    .unwrap();

    let mut surface = ctx.window_surface();

    c.bench_function("debug_lines", |b| {
        b.iter(|| {
            ctx.clear_color(&mut surface, (0.0, 0.0, 0.0, 1.0));

            for i in (0..100).map(|i| i * 2) {
                ctx.debug_line(
                    &mut surface,
                    (i, i + 10),
                    (i + 20, i + 10),
                    (1.0, 0.0, 1.0, 1.0),
                );
            }

            ctx.finalize_frame().unwrap();
        })
    });
}

criterion_group!(benches, debug_lines);
criterion_main!(benches);
