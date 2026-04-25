use std::sync::{LazyLock, Mutex};

use rand::RngExt;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

const RA: usize = 21;
const RI: usize = RA / 3;
const ALPHA_N: f32 = 0.028;
const ALPHA_M: f32 = 0.147;
const B1: f32 = 0.257;
const D1: f32 = 0.365;
const B2: f32 = 0.336;
const D2: f32 = 0.549;
const DT: f32 = 0.05;

const LEVEL: &[u8] = " .-=co*&@#".as_bytes();

static GRID: LazyLock<Mutex<[[f32; HEIGHT]; WIDTH]>> =
    LazyLock::new(|| Mutex::new([[0.0; HEIGHT]; WIDTH]));
static GRID_DIFF: LazyLock<Mutex<[[f32; HEIGHT]; WIDTH]>> =
    LazyLock::new(|| Mutex::new([[0.0; HEIGHT]; WIDTH]));

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
            let level: usize = (grid[y][x] * (LEVEL.len() - 1) as f32).rem_euclid(LEVEL.len() as f32)  as usize;
            print!("{}", LEVEL[level] as char)
        }
        println!();
    }
}

fn recompute_grid() {
    let grid_diff = GRID_DIFF.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            grid[y][x] += DT * grid_diff[y][x];
            grid[y][x] = grid[y][x].clamp(0.0, 1.0);
        }
    }
}

fn sigam1_n(x: f32, a: f32) -> f32 {
    1.0 / (1.0 + (-(x - a) * 4.0 / ALPHA_N).exp())
}

fn sigam1_m(x: f32, a: f32) -> f32 {
    1.0 / (1.0 + (-(x - a) * 4.0 / ALPHA_M).exp())
}

fn sigam2(x: f32, a: f32, b: f32) -> f32 {
    sigam1_n(x, a) * (1.0 - sigam1_n(x, b))
}

fn sigamm(x: f32, y: f32, m: f32) -> f32 {
    x * (1.0 - sigam1_m(m, 0.5)) + y * sigam1_m(m, 0.5)
}

fn s(n: f32, m: f32) -> f32 {
    sigam2(n, sigamm(B1, D1, m), sigamm(B2, D2, m))
}

fn compute_grid_diff() {
    let grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut grid_diff = GRID_DIFF.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for cy in 0..HEIGHT {
        for cx in 0..WIDTH {
            let mut m: f32 = 0.0;
            let mut M: f32 = 0.0;
            let mut n: f32 = 0.0;
            let mut N: f32 = 0.0;

            let ra = RA as i32 - 1;

            for dy in -ra..=ra {
                for dx in -ra..=ra {
                    let x = (cx as i32 + dx).rem_euclid(WIDTH as i32);
                    let y = (cy as i32 + dy).rem_euclid(HEIGHT as i32);

                    if (dx * dx + dy * dy) <= (RI * RI) as i32 {
                        m += grid[y as usize][x as usize];
                        M += 1.0;
                    } else if (dx * dx + dy * dy) <= (RA * RA) as i32 {
                        n += grid[y as usize][x as usize];
                        N += 1.0;
                    }
                }
            }

            grid_diff[cy][cx] = 2.0 * s(n / N, m / M) - 1.0;
        }
    }
}

fn main() {
    println!("Hello, Seaman!");
    random_grid();
    display_grid();

    loop {
        println!();
        compute_grid_diff();
        recompute_grid();
        display_grid();
    }
}
