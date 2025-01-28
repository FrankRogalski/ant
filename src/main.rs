use std::array::from_fn;

use anyhow::{anyhow, Error, Result};
use rand::Rng;
use raylib::ffi::TraceLogLevel;
use raylib::prelude::Image;
use raylib::{color::Color, prelude::RaylibDraw};

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;
const CELL_SIZE: usize = 1;
const WIDTH: usize = SCREEN_WIDTH as usize / CELL_SIZE;
const HEIGHT: usize = SCREEN_HEIGHT as usize / CELL_SIZE;
const AREA: usize = WIDTH * HEIGHT;
const ANTS: usize = 100;
const FPS: u32 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Black,
    White,
}

impl Cell {
    fn invert(&mut self) {
        *self = if *self == Cell::Black {
            Cell::White
        } else {
            Cell::Black
        }
    }
}

#[derive(Debug)]
enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Direction {
    fn random() -> Self {
        match rand::random::<u8>() % 4 {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!(),
        }
    }

    fn next(&mut self) {
        *self = match *self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
}

#[derive(Debug)]
struct Ant {
    pos: usize,
    dir: Direction,
}

impl Ant {
    fn step(&mut self) {
        let pos = self.pos as isize;
        self.pos = match self.dir {
            Direction::Up => (pos - WIDTH as isize).rem_euclid(AREA as isize) as usize,
            Direction::Right => {
                if (self.pos + 1) % WIDTH == 0 {
                    self.pos + 1 - WIDTH
                } else {
                    self.pos + 1
                }
            }
            Direction::Down => (self.pos + WIDTH).rem_euclid(AREA),
            Direction::Left => {
                let pos = pos - 1;
                (if pos.rem_euclid(WIDTH as isize) == (WIDTH - 1) as isize {
                    pos + WIDTH as isize
                } else {
                    pos
                }) as usize
            }
        };
    }

    fn turn(&mut self, cell: Cell) {
        if cell == Cell::Black {
            self.dir.next();
        } else {
            self.dir.prev();
        }
    }
}

fn draw_rect(pos: usize, color: Color, image: &mut Image) {
    let x = pos % WIDTH;
    let y = pos / WIDTH;
    image.draw_rectangle(
        (x * CELL_SIZE) as i32,
        (y * CELL_SIZE) as i32,
        CELL_SIZE as i32,
        CELL_SIZE as i32,
        color,
    );
}

fn main() -> Result<(), Error> {
    assert!(SCREEN_WIDTH % CELL_SIZE as i32 == 0);
    assert!(SCREEN_HEIGHT % CELL_SIZE as i32 == 0);
    let mut rng = rand::thread_rng();
    let mut grid: [Cell; AREA] = [Cell::Black; AREA];
    let (mut rl, thread) = raylib::init()
        .title("Langton's ant")
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    let mut ants: [Ant; ANTS] = from_fn(|_| Ant {
        pos: rng.gen_range(0..AREA),
        dir: Direction::random(),
    });
    let mut image = Image::gen_image_color(SCREEN_WIDTH, SCREEN_HEIGHT, Color::BLACK);
    rl.set_target_fps(FPS);

    while !rl.window_should_close() {
        for ant in ants.iter_mut() {
            let color = if grid[ant.pos] == Cell::White {
                Color::WHITE
            } else {
                Color::BLACK
            };
            draw_rect(ant.pos, color, &mut image);
            ant.step();
            ant.turn(grid[ant.pos]);
            grid[ant.pos].invert();
            draw_rect(ant.pos, Color::RED, &mut image);
        }
        let texture = rl
            .load_texture_from_image(&thread, &image)
            .map_err(|err| anyhow!(err))?;
        let mut draw = rl.begin_drawing(&thread);
        draw.draw_texture(&texture, 0, 0, Color::WHITE);
    }
    Ok(())
}
