use std::{
    sync::{LazyLock, Mutex}
};

use rand::RngExt;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

const LEVEL: &[u8] = " .-=co*&@#".as_bytes();

static GRID: LazyLock<Mutex<[[f32; HEIGHT]; WIDTH]>> =
    LazyLock::new(|| Mutex::new([[0.0; HEIGHT]; WIDTH]));

fn main() {
    println!("Hello, Seaman!");

    let mut rng = rand::rng();

    let mut grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            grid[y][x] = rng.random();
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let level: usize = (grid[y][x] * (LEVEL.len() - 1) as f32) as usize;
            print!("{}", LEVEL[level] as char)
        }
        println!()
    }
}
