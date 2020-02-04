//! An implementation of Conway's Game Of Life.
//!
//! To place a cell press the left mouse button,
//! to advance a generation press space.

use crow::{
    glutin::{
        ElementState, Event, EventsLoop, MouseButton, VirtualKeyCode, WindowBuilder, WindowEvent,
    },
    target::Scaled,
    Context, DrawConfig, Texture,
};

const WINDOW_SIZE: (u32, u32) = (1080, 720);
const CELL_SIZE: u32 = 10;

fn mat((r, g, b): (f32, f32, f32)) -> [[f32; 4]; 4] {
    [
        [r, 0.0, 0.0, 0.0],
        [0.0, g, 0.0, 0.0],
        [0.0, 0.0, b, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn main() -> Result<(), crow::Error> {
    let mut ctx = Context::new(
        WindowBuilder::new().with_dimensions(From::from(WINDOW_SIZE)),
        EventsLoop::new(),
    )?;

    let mut texture = Texture::new(&mut ctx, (1, 1))?;
    ctx.clear_color(&mut texture, (1.0, 1.0, 1.0, 1.0))?;
    let mut surface = Scaled::new(ctx.window_surface(), (CELL_SIZE, CELL_SIZE));

    let mut fin = false;

    let mut mouse_position = (0, 0);
    let mut cells =
        [[false; (WINDOW_SIZE.1 / CELL_SIZE) as usize]; (WINDOW_SIZE.0 / CELL_SIZE) as usize];

    loop {
        ctx.events_loop().poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => fin = true,
                    WindowEvent::CursorMoved { position, .. } => mouse_position = position.into(),
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed
                            && input.virtual_keycode == Some(VirtualKeyCode::Space)
                        {
                            step(&mut cells);
                        }
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    } => {
                        let (x, y) = (
                            mouse_position.0 / CELL_SIZE as i32,
                            mouse_position.1 / CELL_SIZE as i32,
                        );
                        if let Some(cell) = cells
                            .get_mut(x as usize)
                            .and_then(|row| row.get_mut(y as usize))
                        {
                            *cell = !*cell;
                        }
                    }
                    _ => (),
                }
            }
        });

        ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0))?;

        for (x, row) in cells.iter().enumerate() {
            for (y, &cell) in row.iter().enumerate() {
                if cell {
                    let color_modulation = match neighbors(&cells, x as isize, y as isize) {
                        2 | 3 => mat((1.0, 1.0, 1.0)),
                        _ => mat((0.0, 0.0, 0.0)),
                    };

                    ctx.draw(
                        &mut surface,
                        &texture,
                        (x as i32, (row.len() - 1 - y) as i32),
                        &DrawConfig {
                            color_modulation,
                            ..Default::default()
                        },
                    )?;
                }
            }
        }

        ctx.finalize_frame()?;

        if fin {
            break;
        }
    }

    Ok(())
}

fn alive(
    cells: &[[bool; (WINDOW_SIZE.1 / CELL_SIZE) as usize]; (WINDOW_SIZE.0 / CELL_SIZE) as usize],
    x: isize,
    y: isize,
) -> u8 {
    if (x >= 0 && y >= 0)
        && cells
            .get(x as usize)
            .and_then(|row| row.get(y as usize).copied())
            .unwrap_or(false)
    {
        1
    } else {
        0
    }
}

fn neighbors(
    cells: &[[bool; (WINDOW_SIZE.1 / CELL_SIZE) as usize]; (WINDOW_SIZE.0 / CELL_SIZE) as usize],
    x: isize,
    y: isize,
) -> u8 {
    alive(cells, x - 1, y - 1)
        + alive(cells, x, y - 1)
        + alive(cells, x + 1, y - 1)
        + alive(cells, x + 1, y)
        + alive(cells, x + 1, y + 1)
        + alive(cells, x, y + 1)
        + alive(cells, x - 1, y + 1)
        + alive(cells, x - 1, y)
}

fn step(
    cells: &mut [[bool; (WINDOW_SIZE.1 / CELL_SIZE) as usize];
             (WINDOW_SIZE.0 / CELL_SIZE) as usize],
) {
    let mut diffs = Vec::new();

    for (x, row) in cells.iter().enumerate() {
        for (y, &cell) in row.iter().enumerate() {
            let n = neighbors(&cells, x as isize, y as isize);
            if cell {
                if n != 2 && n != 3 {
                    diffs.push((x, y));
                }
            } else if n == 3 {
                diffs.push((x, y));
            }
        }
    }

    for (x, y) in diffs {
        cells[x][y] = !cells[x][y];
    }
}
