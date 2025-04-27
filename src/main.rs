use core::panic;
use std::sync::LazyLock;

use anyhow::{anyhow, Error};
use bitvec::bitbox;
use bitvec::boxed::BitBox;
use clap::Parser;
use rand::Rng;
use raylib::ffi::{KeyboardKey, TraceLogLevel};
use raylib::prelude::Image;
use raylib::{color::Color, prelude::RaylibDraw};

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up = 0,
    Right,
    Down,
    Left,
}

impl From<i8> for Direction {
    fn from(value: i8) -> Self {
        match value {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => panic!("Not a valid number!"),
        }
    }
}

impl Direction {
    fn random() -> Self {
        rand::random::<i8>().rem_euclid(4).into()
    }

    fn next(self) -> Self {
        ((self as i8 + 1) % 4).into()
    }

    fn prev(self) -> Self {
        (self as i8 - 1).rem_euclid(4).into()
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
            Direction::Up => {
                (pos - GLOBALS.width as isize).rem_euclid(GLOBALS.area as isize) as usize
            }
            Direction::Right => {
                if (self.pos + 1) % GLOBALS.width == 0 {
                    self.pos + 1 - GLOBALS.width
                } else {
                    self.pos + 1
                }
            }
            Direction::Down => (self.pos + GLOBALS.width).rem_euclid(GLOBALS.area),
            Direction::Left => {
                let pos = pos - 1;
                (if pos.rem_euclid(GLOBALS.width as isize) == (GLOBALS.width - 1) as isize {
                    pos + GLOBALS.width as isize
                } else {
                    pos
                }) as usize
            }
        };
    }

    fn turn(&mut self, cell: bool) {
        if !cell {
            self.dir = self.dir.next();
        } else {
            self.dir = self.dir.prev();
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value_t = 20)]
    ants: usize,

    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    #[arg(short, long, default_value_t = 1)]
    steps: u32,

    #[arg(long, default_value_t = 1280)]
    screen_width: u32,

    #[arg(long, default_value_t = 720)]
    screen_height: u32,

    #[arg(short, long, default_value_t = 5)]
    cell_size: u32,
}

struct Globals {
    ants: usize,
    fps: u32,
    steps: u32,
    screen_width: i32,
    screen_height: i32,
    cell_size: i32,
    area: usize,
    width: usize,
}

static GLOBALS: LazyLock<Globals> = LazyLock::new(|| {
    let args = Arguments::parse();
    assert!(args.screen_width > 0);
    assert!(args.screen_height > 0);
    assert!(args.cell_size > 0);
    assert!(args.ants > 0);
    assert!(args.fps > 0);
    assert!(args.steps > 0);
    assert!(args.screen_width % args.cell_size == 0);
    assert!(args.screen_height % args.cell_size == 0);
    let width = (args.screen_width / args.cell_size) as usize;
    let height = (args.screen_height / args.cell_size) as usize;
    Globals {
        ants: args.ants,
        fps: args.fps,
        steps: args.steps,
        screen_width: args.screen_width as i32,
        screen_height: args.screen_height as i32,
        cell_size: args.cell_size as i32,
        width,
        area: width * height,
    }
});

fn draw_rect(pos: usize, color: Color, image: &mut Image) {
    let x = pos % GLOBALS.width;
    let y = pos / GLOBALS.width;
    image.draw_rectangle(
        (x * GLOBALS.cell_size as usize) as i32,
        (y * GLOBALS.cell_size as usize) as i32,
        GLOBALS.cell_size,
        GLOBALS.cell_size,
        color,
    );
}

fn get_ants() -> Box<[Ant]> {
    let mut rng = rand::thread_rng();
    (0..GLOBALS.ants)
        .map(|_| Ant {
            pos: rng.gen_range(0..GLOBALS.area),
            dir: Direction::random(),
        })
        .collect()
}

fn main() -> Result<(), Error> {
    let mut grid: BitBox = bitbox![0; GLOBALS.area];
    let mut ants: Box<[Ant]> = get_ants();
    let (mut rl, thread) = raylib::init()
        .title("Langton's ant")
        .size(GLOBALS.screen_width, GLOBALS.screen_height)
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut image =
        Image::gen_image_color(GLOBALS.screen_width, GLOBALS.screen_height, Color::BLACK);
    rl.set_target_fps(GLOBALS.fps);
    let mut fps = GLOBALS.fps;

    while !rl.window_should_close() {
        if rl.is_key_down(KeyboardKey::KEY_R) {
            grid = bitbox![0; GLOBALS.area];
            ants = get_ants();
            image =
                Image::gen_image_color(GLOBALS.screen_width, GLOBALS.screen_height, Color::BLACK);
        }
        if rl.is_key_down(KeyboardKey::KEY_W) && rl.is_key_pressed(KeyboardKey::KEY_LEFT_SUPER) {
            break;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            fps += 5;
            rl.set_target_fps(fps);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            if fps - 5 > 0 {
                fps -= 5;
                rl.set_target_fps(fps);
            } else if fps > 1 {
                fps = 1;
                rl.set_target_fps(fps);
            }
        }
        for _ in 0..GLOBALS.steps {
            for ant in ants.iter_mut() {
                let color = if grid[ant.pos] {
                    Color::WHITE
                } else {
                    Color::BLACK
                };
                draw_rect(ant.pos, color, &mut image);
                ant.step();
                ant.turn(grid[ant.pos]);
                let next = !grid[ant.pos];
                grid.set(ant.pos, next);
                draw_rect(ant.pos, Color::RED, &mut image);
            }
        }
        let texture = rl
            .load_texture_from_image(&thread, &image)
            .map_err(|err| anyhow!(err))?;
        let mut draw = rl.begin_drawing(&thread);
        draw.draw_texture(&texture, 0, 0, Color::WHITE);
    }
    Ok(())
}
