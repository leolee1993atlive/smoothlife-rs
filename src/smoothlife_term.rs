use std::sync::{LazyLock, Mutex};

use rand::RngExt;

use crate::smoothlife_core::{compute_grid_diff, recompute_grid};

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

const LEVEL: &[u8] = " .-=co*&@#".as_bytes();

static GRID: LazyLock<Mutex<[[f32; WIDTH]; HEIGHT]>> =
    LazyLock::new(|| Mutex::new([[0.0; WIDTH]; HEIGHT]));
static GRID_DIFF: LazyLock<Mutex<[[f32; WIDTH]; HEIGHT]>> =
    LazyLock::new(|| Mutex::new([[0.0; WIDTH]; HEIGHT]));

fn random_grid() {
    let mut rng = rand::rng();

    let mut grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            grid[y][x] = rng.random();
        }
    }
}

fn display_grid() {
    let grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let level: usize =
                (grid[y][x] * (LEVEL.len() - 1) as f32).rem_euclid(LEVEL.len() as f32) as usize;
            print!("{}", LEVEL[level] as char)
        }
        println!();
    }
}

pub fn run() {
    println!("Hello, Seaman!");
    random_grid();
    display_grid();

    loop {
        println!();
        {
            let grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut grid_diff = GRID_DIFF
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            compute_grid_diff(&grid, &mut grid_diff);
        }
        {
            let mut grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let grid_diff = GRID_DIFF
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            recompute_grid(&mut grid, &grid_diff);
        }

        display_grid();
    }
}
