use std::{
    array::from_fn,
    thread::sleep,
    time::{Duration, SystemTime},
};

use anyhow::Error;
use rand::Rng;
use raylib::{
    color::Color,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;
const CELL_SIZE: usize = 10;
const WIDTH: usize = SCREEN_WIDTH as usize / CELL_SIZE;
const HEIGHT: usize = SCREEN_HEIGHT as usize / CELL_SIZE;
const AREA: usize = WIDTH * HEIGHT;
const ANTS: usize = 1;
const FPS: f64 = 6.0;
const FRAME_TIME: Duration = Duration::from_nanos((1e9 / FPS) as u64);

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

struct Ant {
    pos: isize,
    dir: Direction,
}

impl Ant {
    fn step(&mut self, cell: Cell) {
        self.pos = match self.dir {
            Direction::Up => (self.pos - WIDTH as isize).rem_euclid(AREA as isize),
            Direction::Right => (self.pos + 1).rem_euclid(WIDTH as isize),
            Direction::Down => (self.pos + WIDTH as isize).rem_euclid(AREA as isize),
            Direction::Left => (self.pos - 1).rem_euclid(WIDTH as isize),
        };
        if cell == Cell::Black {
            self.dir.next();
        } else {
            self.dir.prev();
        }
    }
}

fn draw_cell(pos: isize, cell: Cell, draw: &mut RaylibDrawHandle) {
    let color = if cell == Cell::White {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let x = (pos % WIDTH as isize) as usize;
    let y = (pos / WIDTH as isize) as usize;
    draw.draw_rectangle(
        (x * CELL_SIZE) as i32,
        (y * CELL_SIZE) as i32,
        CELL_SIZE as i32,
        CELL_SIZE as i32,
        color,
    );
}

fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let mut grid: [Cell; AREA] = [Cell::Black; AREA];
    let (mut rl, thread) = raylib::init()
        .title("Ant simulation thing. i forgot the name :(")
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .build();
    let mut ants: [Ant; ANTS] = from_fn(|_| Ant {
        pos: rng.gen_range(0..AREA) as isize,
        dir: Direction::random(),
    });

    while !rl.window_should_close() {
        let start = SystemTime::now();
        let mut draw = rl.begin_drawing(&thread);
        for ant in ants.iter_mut() {
            draw_cell(ant.pos, grid[ant.pos as usize], &mut draw);
            ant.step(grid[ant.pos as usize]);
            grid[ant.pos as usize].invert();
        }
        let frame_time = start.elapsed()?;
        if frame_time < FRAME_TIME {
            sleep(FRAME_TIME - frame_time);
        }
    }
    Ok(())
}
