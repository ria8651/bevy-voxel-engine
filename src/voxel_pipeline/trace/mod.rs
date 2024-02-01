use super::voxel_world::VoxelData;
use bevy::{
    asset::{embedded_asset, load_internal_asset},
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewTarget},
        Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};
pub use node::TraceNode;

mod node;

const COMMON_HANDLE: Handle<Shader> = Handle::weak_from_u128(1874948457211004189);
const BINDINGS_HANDLE: Handle<Shader> = Handle::weak_from_u128(1874948457211004188);
const RAYTRACING_HANDLE: Handle<Shader> = Handle::weak_from_u128(10483863284569474370);

pub struct TracePlugin;

impl Plugin for TracePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(any(not(target_family = "windows"), target_env = "gnu"))]
        {
            embedded_asset!(app, "src/", "trace.wgsl");
        }
        #[cfg(all(target_family = "windows", not(target_env = "gnu")))]
        {
            embedded_asset!(app, "src\\", "trace.wgsl");
        }

        load_internal_asset!(app, COMMON_HANDLE, "common.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, BINDINGS_HANDLE, "bindings.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, RAYTRACING_HANDLE, "raytracing.wgsl", Shader::from_wgsl);

        app.add_plugins(ExtractComponentPlugin::<TraceSettings>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        // Setup custom render pipeline
        render_app
            .init_resource::<TracePipelineData>()
            .insert_resource(LastCameras(HashMap::new()))
            .add_systems(Render, prepare_uniforms.in_set(RenderSet::Prepare));
    }
}

#[derive(Resource)]
struct TracePipelineData {
    trace_pipeline_id: CachedRenderPipelineId,
    trace_bind_group_layout: BindGroupLayout,
}

#[derive(Component, Clone, ExtractComponent)]
pub struct TraceSettings {
    pub show_ray_steps: bool,
    pub samples: u32,
    pub shadows: bool,
}

impl Default for TraceSettings {
    fn default() -> Self {
        Self {
            show_ray_steps: false,
            samples: 1,
            shadows: true,
        }
    }
}

#[derive(Clone, ShaderType)]
pub struct TraceUniforms {
    pub camera: Mat4,
    pub camera_inverse: Mat4,
    pub last_camera: Mat4,
    pub projection: Mat4,
    pub time: f32,
    pub show_ray_steps: u32,
    pub samples: u32,
    pub shadows: u32,
}

#[derive(Component, Deref, DerefMut)]
struct ViewTraceUniformBuffer(UniformBuffer<TraceUniforms>);

#[derive(Resource, Deref, DerefMut)]
struct LastCameras(HashMap<Entity, Mat4>);

fn prepare_uniforms(
    mut commands: Commands,
    query: Query<(Entity, &TraceSettings, &ExtractedView)>,
    time: Res<Time>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut last_cameras: ResMut<LastCameras>,
) {
    let elapsed = time.elapsed_seconds_f64();

    for (entity, settings, view) in query.iter() {
        let projection = view.projection;
        let inverse_projection = projection.inverse();
        let view = view.transform.compute_matrix();
        let inverse_view = view.inverse();

        let camera = projection * inverse_view;
        let camera_inverse = view * inverse_projection;

        let last_camera = *last_cameras.get(&entity).unwrap_or(&camera);
        last_cameras.insert(entity, camera);

        let uniforms = TraceUniforms {
            camera,
            camera_inverse,
            last_camera,
            projection,
            time: elapsed as f32,
            show_ray_steps: settings.show_ray_steps as u32,
            samples: settings.samples,
            shadows: settings.shadows as u32,
        };

        let mut uniform_buffer = UniformBuffer::from(uniforms);
        uniform_buffer.write_buffer(&render_device, &render_queue);

        commands
            .entity(entity)
            .insert(ViewTraceUniformBuffer(uniform_buffer));
    }
}

impl FromWorld for TracePipelineData {
    fn from_world(render_world: &mut World) -> Self {
        let voxel_data = render_world.resource::<VoxelData>();
        let asset_server = render_world.resource::<AssetServer>();

        let voxel_bind_group_layout = voxel_data.bind_group_layout.clone();

        let trace_bind_group_layout = render_world
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
                            min_binding_size: BufferSize::new(TraceUniforms::SHADER_SIZE.into()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let trace_shader_handle =
            asset_server.load("embedded://bevy_voxel_engine/voxel_pipeline/trace/trace.wgsl");
        let trace_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("trace pipeline".into()),
            layout: vec![
                voxel_bind_group_layout.clone(),
                trace_bind_group_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: trace_shader_handle,
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
            push_constant_ranges: vec![],
        };

        let cache = render_world.resource::<PipelineCache>();
        let trace_pipeline_id = cache.queue_render_pipeline(trace_pipeline_descriptor);

        TracePipelineData {
            trace_pipeline_id,
            trace_bind_group_layout,
        }
    }
}
