//! An implementation of Conway's Game Of Life.
//!
//! To place a cell press the left mouse button,
//! to advance a generation press space.

use crow::{
    glutin::{
        dpi::LogicalSize,
        event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
        window::WindowBuilder,
    },
    target::Scaled,
    Context, DrawConfig, Texture,
};

const WINDOW_WIDTH: u32 = 1080;
const WINDOW_HEIGHT: u32 = 720;
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
        WindowBuilder::new().with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
    )?;

    let mut texture = Texture::new(&mut ctx, (1, 1))?;
    ctx.clear_color(&mut texture, (1.0, 1.0, 1.0, 1.0));

    let mut mouse_position = (0, 0);
    let mut cells =
        [[false; (WINDOW_HEIGHT / CELL_SIZE) as usize]; (WINDOW_WIDTH / CELL_SIZE) as usize];

    ctx.run(move |ctx: &mut Context, surface: &mut _, events| {
        let surface = &mut Scaled::new(surface, (CELL_SIZE, CELL_SIZE));
        for event in events {
            if let Event::WindowEvent { event, .. } = event {
                match event {
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
        }

        ctx.clear_color(surface, (0.4, 0.4, 0.8, 1.0));

        for (x, row) in cells.iter().enumerate() {
            for (y, &cell) in row.iter().enumerate() {
                if cell {
                    let color_modulation = match neighbors(&cells, x as isize, y as isize) {
                        2 | 3 => mat((1.0, 1.0, 1.0)),
                        _ => mat((0.0, 0.0, 0.0)),
                    };

                    ctx.draw(
                        surface,
                        &texture,
                        (x as i32, (row.len() - 1 - y) as i32),
                        &DrawConfig {
                            color_modulation,
                            ..Default::default()
                        },
                    );
                }
            }
        }
        true
    })
}

fn alive(
    cells: &[[bool; (WINDOW_HEIGHT / CELL_SIZE) as usize]; (WINDOW_WIDTH / CELL_SIZE) as usize],
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
    cells: &[[bool; (WINDOW_HEIGHT / CELL_SIZE) as usize]; (WINDOW_WIDTH / CELL_SIZE) as usize],
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
    cells: &mut [[bool; (WINDOW_HEIGHT / CELL_SIZE) as usize]; (WINDOW_WIDTH / CELL_SIZE) as usize],
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
