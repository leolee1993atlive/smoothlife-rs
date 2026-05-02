use std::{
    io::{self, Write},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use rand::RngExt;

use crate::smoothlife_core::{compute_grid_diff, recompute_grid};

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
const TARGET_FPS: u64 = 20;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

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

fn display_grid(frame: u64) {
    let grid = GRID.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // 光标移到 (0,0) 原位覆写，不滚屏
    write!(out, "\x1b[H").unwrap();

    write!(out, "SmoothLife  frame: {frame}  (q to quit)\r\n").unwrap();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let level =
                (grid[y][x] * (LEVEL.len() - 1) as f32).rem_euclid(LEVEL.len() as f32) as usize;
            out.write_all(&[LEVEL[level]]).unwrap();
        }
        out.write_all(b"\r\n").unwrap();
    }
    out.flush().unwrap();
}

pub fn run() {
    random_grid();

    // 进入备用屏幕，隐藏光标
    print!("\x1b[?1049h\x1b[?25l\x1b[2J");

    let mut frame = 0u64;
    loop {
        let frame_start = Instant::now();

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

        display_grid(frame);
        frame += 1;

        if let Some(remaining) = FRAME_DURATION.checked_sub(frame_start.elapsed()) {
            std::thread::sleep(remaining);
        }
    }
}
