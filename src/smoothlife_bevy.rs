use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::WindowResolution,
};
use rand::RngExt;

use crate::smoothlife_core::{compute_grid_diff, recompute_grid};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SmoothLife".to_string(),
                resolution: WindowResolution::new(WIDTH, HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, update_grid)
        .run();
}

#[derive(Component)]
struct Grid {
    current: Box<[[f32; WIDTH as usize]; HEIGHT as usize]>,
    next: Box<[[f32; WIDTH as usize]; HEIGHT as usize]>,
}

impl Grid {
    fn new() -> Self {
        let mut rng = rand::rng();

        let mut grid_current = Box::new([[0.0; WIDTH as usize]; HEIGHT as usize]);
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                grid_current[y][x] = rng.random();
            }
        }

        Self {
            current: grid_current,
            next: Box::new([[0.0; WIDTH as usize]; HEIGHT as usize]),
        }
    }
}

#[derive(Component)]
struct GridImage(Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    };

    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image);

    let grid = Grid::new();
    let new_image = create_image(&grid);
    let _ = images.insert(image_handle.id(), new_image);

    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
            ..default()
        },
        Transform::default(),
        grid,
        GridImage(image_handle),
    ));
}

fn update_grid(
    mut query: Query<(&mut Grid, &GridImage)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut grid, grid_image) in &mut query {
        let Grid { current, next } = &mut *grid;
        compute_grid_diff(current, next);
        recompute_grid(current, next);

        if let Some(image) = images.get_mut(&grid_image.0) {
            update_image_pixels(image, current);
        }
    }
}

fn create_image(grid: &Grid) -> Image {
    let mut pixel_data = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);

    for y in 0..HEIGHT as usize {
        for x in 0..WIDTH as usize {
            let current = grid.current[y][x];
            let (r, g, b, a) = pixel_color(current);
            pixel_data.push(r);
            pixel_data.push(g);
            pixel_data.push(b);
            pixel_data.push(a);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();
    image
}

fn update_image_pixels(image: &mut Image, grid: &[[f32; WIDTH as usize]; HEIGHT as usize]) {
    if let Some(data) = image.data.as_mut() {
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                let idx = (y * WIDTH as usize + x) * 4;
                let (r, g, b, a) = pixel_color(grid[y][x]);
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = a;
            }
        }
    }
}

fn pixel_color(current: f32) -> (u8, u8, u8, u8) {
    if current > 0.0 {
        let hue = (1.0 - current) * 240.0; // 240°(蓝) ~ 0°(红)
        let (r, g, b) = hsl_to_rgb_u8(hue, 0.9, 0.3 + current * 0.4);
        (r, g, b, 255u8)
    } else {
        (0u8, 0u8, 0u8, 255u8)
    }
}

fn hsl_to_rgb_u8(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h / 60.0) as i32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
