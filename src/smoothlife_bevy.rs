use bevy::{asset::RenderAssetUsages, image::ImageSampler, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, window::WindowResolution};
use rand::RngExt;

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
        .run();
}

#[derive(Component)]
struct Grid {
    current: Box<[[f32; WIDTH as usize]; HEIGHT as usize]>,
    next: Box<[[f32; WIDTH as usize]; HEIGHT as usize]>,
}

impl Grid  {
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

    let mut image = Image::new_fill(
        size, 
        TextureDimension::D2, &[0,0,0,255], 
        TextureFormat::Rgba8UnormSrgb, 
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image);

    let grid = Grid::new();
    let new_image = create_image(&grid);
    images.insert(image_handle.id(), new_image);

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

fn create_image(grid: &Grid) -> Image {
    let mut pixel_data = Vec::with_capacity((WIDTH * HEIGHT) as usize); 
    
    for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                let current = grid.current[y][x];

                let (r, g, b, a) = if current > 0.0 {
                    let hue = (1.0 - current) * 240.0; // 240°(蓝) ~ 0°(红)
                    let (r, g, b) = hsl_to_rgb_u8(hue, 0.9, 0.3 + current * 0.4);
                    (r, g, b, 255u8)
                } else {
                    (0u8, 0u8, 0u8, 255u8)
                };

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

fn idx(x: i32, y: i32) -> usize {
    let wrapped_x = ((x % WIDTH as i32 + WIDTH as i32) % WIDTH as i32) as u32;
    let wrapped_y = ((y % HEIGHT as i32 + HEIGHT as i32) % HEIGHT as i32) as u32;
    (wrapped_y * WIDTH + wrapped_x) as usize
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
