use crate::{
    animation,
    compute::ExtractedGH,
    load::{Pallete, GH},
    LoadVoxelWorld, VoxelCamera,
};
use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::ViewTarget,
        RenderApp, RenderStage,
    },
};
use std::sync::Arc;

pub mod node;

pub struct TracePlugin;

impl Plugin for TracePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_device = app.world.resource::<RenderDevice>();
        let render_queue = app.world.resource::<RenderQueue>();

        // uniforms
        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ExtractedUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let default_path = "/Users/brian/Documents/Code/Rust/vox/monument/monu9.vox".to_string();
        let gh = if let Ok(file) = std::fs::read(default_path) {
            GH::from_vox(&file).unwrap()
        } else {
            GH::empty(32)
        };
        let buffer_size = gh.get_final_length() as usize / 8;
        // let texture_size = gh.texture_size;

        // texture
        let voxel_world = render_device.create_texture_with_data(
            render_queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: gh.texture_size,
                    height: gh.texture_size,
                    depth_or_array_layers: gh.texture_size,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: TextureFormat::R16Uint,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
            },
            &gh.texture_data.clone(),
        );
        let voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());

        // storage
        let grid_heierachy = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; buffer_size],
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // uniforms
        let uniforms_struct = Uniforms {
            pallete: gh.pallete.clone().into(),
            portals: [ExtractedPortal::default(); 32],
            resolution: Vec4::default(),
            last_camera: Mat4::default(),
            camera: Mat4::default(),
            camera_inverse: Mat4::default(),
            levels: gh.levels,
            offsets: gh.get_offsets(),
            time: 0.0,
            delta_time: 1.0 / 120.0,
            texture_size: gh.texture_size,
            show_ray_steps: false,
            indirect_lighting: false,
            shadows: true,
            accumulation_frames: 1.0,
            fov: 1.0,
            freeze: false,
            enable_compute: true,
            skybox: true,
            misc_bool: false,
            misc_float: 1.0,
        };

        app.insert_resource(uniforms_struct)
            .insert_resource(ShaderTimer(Timer::from_seconds(
                1000.0,
                TimerMode::Repeating,
            )))
            .insert_resource(LastFrameData {
                last_camera: Mat4::default(),
            })
            .insert_resource(LoadVoxelWorld::None)
            .insert_resource(NewGH::None)
            .add_plugin(ExtractResourcePlugin::<ExtractedUniforms>::default())
            .add_plugin(ExtractResourcePlugin::<NewGH>::default())
            .add_system(update_uniforms)
            .add_system(load_voxel_world);

        // setup custom render pipeline
        app.sub_app_mut(RenderApp)
            .init_resource::<TracePipeline>()
            .init_resource::<SpecializedRenderPipelines<TracePipeline>>()
            .insert_resource(TraceData {
                uniform_buffer,
                voxel_world,
                grid_heierachy,
            })
            .add_system_to_stage(RenderStage::Queue, queue_trace_pipeline)
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms)
            .add_system_to_stage(RenderStage::Prepare, load_voxel_world_prepare);
    }
}

#[derive(Resource)]
struct TracePipeline {
    shader: Handle<Shader>,
    bind_group: BindGroupLayout,
}

#[derive(Resource)]
pub struct TraceData {
    pub uniform_buffer: Buffer,
    pub voxel_world: TextureView,
    pub grid_heierachy: Buffer,
}

#[derive(Component)]
struct ViewTracePipeline(CachedRenderPipelineId);

impl SpecializedRenderPipeline for TracePipeline {
    type Key = ();

    fn specialize(&self, _: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("trace pipeline".into()),
            layout: Some(vec![self.bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        }
    }
}

impl FromWorld for TracePipeline {
    fn from_world(render_world: &mut World) -> Self {
        let trace_bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("trace bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<ExtractedUniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::R16Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                ],
            });

        let asset_server = render_world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shader.wgsl");

        TracePipeline {
            shader,
            bind_group: trace_bind_group,
        }
    }
}

fn queue_trace_pipeline(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TracePipeline>>,
    view_pipeline: Res<TracePipeline>,
    view_targets: Query<Entity, With<ViewTarget>>,
) {
    for entity in view_targets.iter() {
        let pipeline = pipelines.specialize(&mut pipeline_cache, &view_pipeline, ());
        commands.entity(entity).insert(ViewTracePipeline(pipeline));
    }
}

fn update_uniforms(
    mut uniforms: ResMut<Uniforms>,
    windows: Res<Windows>,
    main_cam: Query<(&Transform, &Projection), With<VoxelCamera>>,
    mut shader_timer: ResMut<ShaderTimer>,
    time: Res<Time>,
    mut last_frame_data: ResMut<LastFrameData>,
) {
    let window = windows.primary();
    uniforms.resolution = Vec4::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
        0.0,
        0.0,
    );

    let (transform, _perspective) = main_cam.single();

    let transform = Transform {
        translation: animation::world_to_render(transform.translation, uniforms.texture_size),
        ..*transform
    };

    uniforms.camera_inverse = transform.compute_matrix();
    uniforms.camera = uniforms.camera_inverse.inverse();
    uniforms.last_camera = last_frame_data.last_camera;
    if !uniforms.freeze {
        last_frame_data.last_camera = uniforms.camera;
    }

    shader_timer.0.tick(time.delta());
    uniforms.time = shader_timer.0.elapsed_secs();
    uniforms.delta_time = time.delta_seconds();
}

fn prepare_uniforms(
    extraced_uniforms: Res<ExtractedUniforms>,
    trace_data: ResMut<TraceData>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &trace_data.uniform_buffer,
        0,
        bytemuck::cast_slice(&[*extraced_uniforms.as_ref()]),
    );
}

fn load_voxel_world(
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut new_gh: ResMut<NewGH>,
    mut uniforms: ResMut<Uniforms>,
) {
    match load_voxel_world.as_ref() {
        LoadVoxelWorld::Empty(_) | LoadVoxelWorld::File(_) => {
            let gh = match load_voxel_world.as_ref() {
                LoadVoxelWorld::Empty(size) => GH::empty(*size),
                LoadVoxelWorld::File(path) => {
                    let file = std::fs::read(path).unwrap();
                    GH::from_vox(&file).unwrap()
                }
                LoadVoxelWorld::None => unreachable!(),
            };

            uniforms.pallete = gh.pallete.clone().into();
            uniforms.levels = gh.levels;
            uniforms.texture_size = gh.texture_size;

            *new_gh = NewGH::Some(Arc::new(gh));
            *load_voxel_world = LoadVoxelWorld::None;
        }
        LoadVoxelWorld::None => {
            *new_gh = NewGH::None;
        }
    }
}

fn load_voxel_world_prepare(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut trace_meta: ResMut<TraceData>,
    new_gh: Res<NewGH>,
) {
    if let NewGH::Some(gh) = new_gh.as_ref() {
        let buffer_size = gh.get_final_length() as usize / 8;
        let texture_size = gh.texture_size;

        // grid hierarchy
        trace_meta.grid_heierachy = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; buffer_size],
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // voxel world
        let voxel_world = render_device.create_texture_with_data(
            render_queue.as_ref(),
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: gh.texture_size,
                    height: gh.texture_size,
                    depth_or_array_layers: gh.texture_size,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: TextureFormat::R16Uint,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
            },
            &gh.texture_data,
        );
        trace_meta.voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());

        commands.insert_resource(ExtractedGH {
            buffer_size,
            texture_size,
        });
    }
}

#[derive(Resource, ExtractResource, Clone)]
enum NewGH {
    Some(Arc<GH>),
    None,
}

#[derive(Resource)]
pub struct ShaderTimer(pub Timer);

#[derive(Resource)]
pub struct LastFrameData {
    pub last_camera: Mat4,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PalleteEntry {
    pub colour: [f32; 4],
}

impl Into<[PalleteEntry; 256]> for Pallete {
    fn into(self) -> [PalleteEntry; 256] {
        let mut pallete = [PalleteEntry::default(); 256];
        for i in 0..256 {
            pallete[i].colour = self[i].into();
        }
        pallete
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ExtractedPortal {
    pub pos: [f32; 4],
    pub other_pos: [f32; 4],
    pub normal: [f32; 4],
    pub other_normal: [f32; 4],
    pub half_size: [i32; 4],
}

#[derive(Resource)]
pub struct Uniforms {
    pub pallete: [PalleteEntry; 256],
    pub portals: [ExtractedPortal; 32],
    pub resolution: Vec4,
    pub last_camera: Mat4,
    pub camera: Mat4,
    pub camera_inverse: Mat4,
    pub levels: [u32; 8],
    pub offsets: [u32; 8],
    pub time: f32,
    pub delta_time: f32,
    pub texture_size: u32,
    pub show_ray_steps: bool,
    pub indirect_lighting: bool,
    pub shadows: bool,
    pub accumulation_frames: f32,
    pub fov: f32,
    pub freeze: bool,
    pub enable_compute: bool,
    pub skybox: bool,
    pub misc_bool: bool,
    pub misc_float: f32,
}

#[repr(C)]
#[derive(Resource, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ExtractedUniforms {
    pallete: [PalleteEntry; 256],
    portals: [ExtractedPortal; 32],
    resolution: Vec4,
    last_camera: Mat4,
    camera: Mat4,
    camera_inverse: Mat4,
    levels: [u32; 8],
    offsets: [u32; 8],
    time: f32,
    delta_time: f32,
    texture_size: u32,
    show_ray_steps: u32,
    indirect_lighting: u32,
    shadows: u32,
    accumulation_frames: f32,
    fov: f32,
    freeze: u32,
    pub enable_compute: u32,
    skybox: u32,
    misc_bool: u32,
    misc_float: f32,
    padding: [u32; 3],
}

impl ExtractResource for ExtractedUniforms {
    type Source = Uniforms;

    fn extract_resource(uniforms: &Self::Source) -> Self {
        ExtractedUniforms {
            pallete: uniforms.pallete,
            portals: uniforms.portals,
            resolution: uniforms.resolution,
            last_camera: uniforms.last_camera,
            camera: uniforms.camera,
            camera_inverse: uniforms.camera_inverse,
            levels: uniforms.levels,
            offsets: uniforms.offsets,
            time: uniforms.time,
            delta_time: uniforms.delta_time,
            texture_size: uniforms.texture_size,
            show_ray_steps: uniforms.show_ray_steps as u32,
            indirect_lighting: uniforms.indirect_lighting as u32,
            shadows: uniforms.shadows as u32,
            accumulation_frames: uniforms.accumulation_frames,
            fov: uniforms.fov,
            freeze: uniforms.freeze as u32,
            enable_compute: uniforms.enable_compute as u32,
            skybox: uniforms.skybox as u32,
            misc_bool: uniforms.misc_bool as u32,
            misc_float: uniforms.misc_float,
            padding: [0; 3],
        }
    }
}
