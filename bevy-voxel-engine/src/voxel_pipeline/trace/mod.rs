use super::voxel_world::{VoxelData, VoxelUniforms};
use crate::{load::Pallete, LoadVoxelWorld, VoxelCamera, physics};
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

pub mod node;

pub struct TracePlugin;

impl Plugin for TracePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_device = app.world.resource::<RenderDevice>();

        let uniforms_struct = TraceUniforms {
            resolution: Vec4::default(),
            last_camera: Mat4::default(),
            camera: Mat4::default(),
            camera_inverse: Mat4::default(),
            time: 0.0,
            delta_time: 1.0 / 120.0,
            show_ray_steps: false,
            indirect_lighting: false,
            shadows: true,
            accumulation_frames: 1.0,
            fov: 1.0,
            freeze: false,
            skybox: true,
            misc_bool: false,
            misc_float: 1.0,
        };

        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ExtractedUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        app.insert_resource(uniforms_struct)
            .insert_resource(ShaderTimer(Timer::from_seconds(
                1000.0,
                TimerMode::Repeating,
            )))
            .insert_resource(LastFrameData {
                last_camera: Mat4::default(),
            })
            .insert_resource(LoadVoxelWorld::None)
            .add_plugin(ExtractResourcePlugin::<ExtractedUniforms>::default())
            .add_system(update_uniforms);

        // setup custom render pipeline
        app.sub_app_mut(RenderApp)
            .init_resource::<TracePipeline>()
            .init_resource::<SpecializedRenderPipelines<TracePipeline>>()
            .insert_resource(TraceData { uniform_buffer })
            .add_system_to_stage(RenderStage::Queue, queue_trace_pipeline)
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms);
    }
}

#[derive(Resource)]
pub struct TraceData {
    pub uniform_buffer: Buffer,
}

#[derive(Resource)]
struct TracePipeline {
    shader: Handle<Shader>,
    voxel_bind_group_layout: BindGroupLayout,
    trace_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
struct ViewTracePipeline(CachedRenderPipelineId);

impl SpecializedRenderPipeline for TracePipeline {
    type Key = ();

    fn specialize(&self, _: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("trace pipeline".into()),
            layout: Some(vec![
                self.voxel_bind_group_layout.clone(),
                self.trace_bind_group_layout.clone(),
            ]),
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
        let voxel_data = render_world.get_resource::<VoxelData>().unwrap();

        let voxel_bind_group_layout = voxel_data.bind_group_layout.clone();
        let trace_bind_group_layout = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("trace bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            std::mem::size_of::<ExtractedUniforms>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let asset_server = render_world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shader.wgsl");

        TracePipeline {
            shader,
            voxel_bind_group_layout,
            trace_bind_group_layout,
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
    mut uniforms: ResMut<TraceUniforms>,
    windows: Res<Windows>,
    main_cam: Query<(&Transform, &Projection), With<VoxelCamera>>,
    mut shader_timer: ResMut<ShaderTimer>,
    time: Res<Time>,
    mut last_frame_data: ResMut<LastFrameData>,
    voxel_world_uniforms: Res<VoxelUniforms>,
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
        translation: physics::world_to_render(
            transform.translation,
            voxel_world_uniforms.texture_size,
        ),
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
pub struct TraceUniforms {
    pub resolution: Vec4,
    pub last_camera: Mat4,
    pub camera: Mat4,
    pub camera_inverse: Mat4,
    pub time: f32,
    pub delta_time: f32,
    pub show_ray_steps: bool,
    pub indirect_lighting: bool,
    pub shadows: bool,
    pub accumulation_frames: f32,
    pub fov: f32,
    pub freeze: bool,
    pub skybox: bool,
    pub misc_bool: bool,
    pub misc_float: f32,
}

#[repr(C)]
#[derive(Resource, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ExtractedUniforms {
    resolution: Vec4,
    last_camera: Mat4,
    camera: Mat4,
    camera_inverse: Mat4,
    time: f32,
    delta_time: f32,
    show_ray_steps: u32,
    indirect_lighting: u32,
    shadows: u32,
    accumulation_frames: f32,
    fov: f32,
    freeze: u32,
    skybox: u32,
    misc_bool: u32,
    misc_float: f32,
    padding: [u32; 1],
}

impl ExtractResource for ExtractedUniforms {
    type Source = TraceUniforms;

    fn extract_resource(uniforms: &Self::Source) -> Self {
        ExtractedUniforms {
            resolution: uniforms.resolution,
            last_camera: uniforms.last_camera,
            camera: uniforms.camera,
            camera_inverse: uniforms.camera_inverse,
            time: uniforms.time,
            delta_time: uniforms.delta_time,
            show_ray_steps: uniforms.show_ray_steps as u32,
            indirect_lighting: uniforms.indirect_lighting as u32,
            shadows: uniforms.shadows as u32,
            accumulation_frames: uniforms.accumulation_frames,
            fov: uniforms.fov,
            freeze: uniforms.freeze as u32,
            skybox: uniforms.skybox as u32,
            misc_bool: uniforms.misc_bool as u32,
            misc_float: uniforms.misc_float,
            padding: [0; 1],
        }
    }
}
