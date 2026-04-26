use rayon::prelude::*;

pub const RA: usize = 21;
pub const RI: usize = RA / 3;
pub const ALPHA_N: f32 = 0.028;
pub const ALPHA_M: f32 = 0.147;
pub const B1: f32 = 0.257;
pub const D1: f32 = 0.365;
pub const B2: f32 = 0.336;
pub const D2: f32 = 0.549;
pub const DT: f32 = 0.05;

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

pub fn compute_grid_diff<const W: usize, const H: usize>(
    grid: &[[f32; W]; H],
    grid_diff: &mut [[f32; W]; H],
) {
    grid_diff.par_iter_mut().enumerate().for_each(|(cy, row)| {
        for cx in 0..W {
            let mut m: f32 = 0.0;
            let mut m_count: f32 = 0.0;
            let mut n: f32 = 0.0;
            let mut n_count: f32 = 0.0;

            let ra = RA as i32 - 1;

            for dy in -ra..=ra {
                for dx in -ra..=ra {
                    let x = (cx as i32 + dx).rem_euclid(W as i32);
                    let y = (cy as i32 + dy).rem_euclid(H as i32);

                    if (dx * dx + dy * dy) <= (RI * RI) as i32 {
                        m += grid[y as usize][x as usize];
                        m_count += 1.0;
                    } else if (dx * dx + dy * dy) <= (RA * RA) as i32 {
                        n += grid[y as usize][x as usize];
                        n_count += 1.0;
                    }
                }
            }

            row[cx] = 2.0 * s(n / n_count, m / m_count) - 1.0;
        }
    });
}

pub fn recompute_grid<const W: usize, const H: usize>(
    grid: &mut [[f32; W]; H],
    grid_diff: &[[f32; W]; H],
) {
    for y in 0..H {
        for x in 0..W {
            grid[y][x] += DT * grid_diff[y][x];
            grid[y][x] = grid[y][x].clamp(0.0, 1.0);
        }
    }
}
