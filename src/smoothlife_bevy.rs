use std::borrow::Cow;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType,
            StorageTextureAccess, TextureFormat, TextureUsages, UniformBuffer,
            binding_types::{texture_storage_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    shader::PipelineCacheError,
};

const SHADER_ASSET_PATH: &str = "shaders/smooth_life.wgsl";

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const DISPLAY_FACTOR: u32 = 4;
const WORKGROUP_SIZE: u32 = 8;

const SIZE: UVec2 = UVec2::new(WIDTH / DISPLAY_FACTOR, HEIGHT / DISPLAY_FACTOR);

pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SmoothLife".to_string(),
                        resolution: (SIZE * DISPLAY_FACTOR).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            SmoothLifeComputePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (switch_textures, update_brush))
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_target_texture(SIZE.x, SIZE.y, TextureFormat::Rgba32Float, None);
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    // 显式清零，避免 GPU 未定义内存导致 NaN/垃圾值污染卷积
    let pixel_bytes = 16u32; // Rgba32Float = 4 × 4 bytes
    image.data = Some(vec![0u8; (SIZE.x * SIZE.y * pixel_bytes) as usize]);
    let image0 = images.add(image.clone());
    let image1 = images.add(image);

    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            image: image0.clone(),
            custom_size: Some(SIZE.as_vec2()),
            ..default()
        },
        Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
    ));
    commands.insert_resource(SmoothLifeImages {
        texture_a: image0,
        texture_b: image1,
    });

    commands.insert_resource(SmoothLifeUniforms {
        alive_color: LinearRgba::RED,
        brush_x: 0.0,
        brush_y: 0.0,
        brush_radius: 12.0, // 纹理像素半径
        brush_state: -1.0,
    });
}

fn update_brush(
    mut uniforms: ResMut<SmoothLifeUniforms>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    let left = mouse_buttons.pressed(MouseButton::Left);
    let right = mouse_buttons.pressed(MouseButton::Right);

    if left || right {
        if let Some(pos) = window.cursor_position() {
            // 窗口坐标 / DISPLAY_FACTOR = 纹理坐标
            uniforms.brush_x = pos.x / DISPLAY_FACTOR as f32;
            uniforms.brush_y = pos.y / DISPLAY_FACTOR as f32;
            uniforms.brush_state = if left { 1.0 } else { 0.0 };
        }
    } else {
        uniforms.brush_state = -1.0;
    }
}

fn switch_textures(images: Res<SmoothLifeImages>, mut sprite: Single<&mut Sprite>) {
    if sprite.image == images.texture_a {
        sprite.image = images.texture_b.clone();
    } else {
        sprite.image = images.texture_a.clone();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct SmmothLifeLabel;

#[derive(Resource, Clone, ExtractResource)]
struct SmoothLifeImages {
    texture_a: Handle<Image>,
    texture_b: Handle<Image>,
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
struct SmoothLifeUniforms {
    alive_color: LinearRgba,
    brush_x: f32,
    brush_y: f32,
    brush_radius: f32,
    brush_state: f32, // -1.0=不活跃, 0.0=消除, 1.0=种植
}

enum SmoothLifeState {
    Loading,
    Init,
    Update(usize),
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<SmoothLifePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_images: Res<SmoothLifeImages>,
    game_of_life_uniforms: Res<SmoothLifeUniforms>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let view_a = gpu_images.get(&game_of_life_images.texture_a).unwrap();
    let view_b = gpu_images.get(&game_of_life_images.texture_b).unwrap();

    // Uniform buffer is used here to demonstrate how to set up a uniform in a compute shader
    // Alternatives such as storage buffers or push constants may be more suitable for your use case
    let mut uniform_buffer = UniformBuffer::from(game_of_life_uniforms.into_inner());
    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group_0 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
        &BindGroupEntries::sequential((
            &view_a.texture_view,
            &view_b.texture_view,
            &uniform_buffer,
        )),
    );
    let bind_group_1 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
        &BindGroupEntries::sequential((
            &view_b.texture_view,
            &view_a.texture_view,
            &uniform_buffer,
        )),
    );
    commands.insert_resource(SmoothLifeImageBindGroups([bind_group_0, bind_group_1]));
}

#[derive(Resource)]
struct SmoothLifePipeline {
    texture_bind_group_layout: BindGroupLayoutDescriptor,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

fn init_smooth_life_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let texture_bind_group_layout = BindGroupLayoutDescriptor::new(
        "SmoothLifeImages",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                uniform_buffer::<SmoothLifeUniforms>(false),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("init")),
        ..default()
    });
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(SmoothLifePipeline {
        texture_bind_group_layout,
        init_pipeline,
        update_pipeline,
    });
}

struct SmoothLifeNode {
    state: SmoothLifeState,
}

impl Default for SmoothLifeNode {
    fn default() -> Self {
        Self {
            state: SmoothLifeState::Loading,
        }
    }
}

#[derive(Resource)]
struct SmoothLifeImageBindGroups([BindGroup; 2]);

impl render_graph::Node for SmoothLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<SmoothLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            SmoothLifeState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = SmoothLifeState::Init;
                    }
                    CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => {}
                    CachedPipelineState::Err(err) => {
                        panic!("Failed to get init pipeline state: {:?}", err);
                    }
                    _ => {}
                }
            }
            SmoothLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = SmoothLifeState::Update(1);
                }
            }
            SmoothLifeState::Update(0) => {
                self.state = SmoothLifeState::Update(1);
            }

            SmoothLifeState::Update(1) => {
                self.state = SmoothLifeState::Update(0);
            }
            SmoothLifeState::Update(_) => unreachable!(),
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_groups = &world.resource::<SmoothLifeImageBindGroups>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<SmoothLifePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        // select the pipeline based on the current state
        match self.state {
            SmoothLifeState::Loading => {}
            SmoothLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_bind_group(0, &bind_groups[0], &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    (SIZE.x + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                    (SIZE.y + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                    1,
                );
            }
            SmoothLifeState::Update(index) => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_bind_group(0, &bind_groups[index], &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    (SIZE.x + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                    (SIZE.y + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}

struct SmoothLifeComputePlugin;

impl Plugin for SmoothLifeComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<SmoothLifeImages>::default(),
            ExtractResourcePlugin::<SmoothLifeUniforms>::default(),
        ));
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_smooth_life_pipeline)
            .add_systems(
                Render,
                prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(SmmothLifeLabel, SmoothLifeNode::default());
        render_graph.add_node_edge(SmmothLifeLabel, CameraDriverLabel);
    }
}
