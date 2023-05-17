use super::voxel_world::VoxelData;
use bevy::{
    asset::load_internal_asset,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderSet,
    },
    utils::HashMap,
};
pub use node::TraceNode;

mod node;

const TRACE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3541867952248261868);
const REPROJECTION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 18296058895300425745);
const COMMON_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1874948457211004189);
const RAYTRACING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10483863284569474370);

pub struct TracePlugin;

impl Plugin for TracePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TRACE_SHADER_HANDLE,
            "../shaders/trace.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            REPROJECTION_SHADER_HANDLE,
            "../shaders/reprojection.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            COMMON_SHADER_HANDLE,
            "../shaders/common.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            RAYTRACING_SHADER_HANDLE,
            "../shaders/raytracing.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<TraceSettings>::default());

        // setup custom render pipeline
        app.sub_app_mut(RenderApp)
            .init_resource::<TracePipelineData>()
            .insert_resource(LastCameras(HashMap::new()))
            .add_system(prepare_uniforms.in_set(RenderSet::Prepare));
    }
}

#[derive(Resource)]
struct TracePipelineData {
    trace_pipeline_id: CachedRenderPipelineId,
    reprojection_pipeline_id: CachedRenderPipelineId,
    accumulation_pipeline_id: CachedRenderPipelineId,
    trace_bind_group_layout: BindGroupLayout,
    reprojection_bind_group_layout: BindGroupLayout,
}

#[derive(Component, Clone, ExtractComponent)]
pub struct TraceSettings {
    pub show_ray_steps: bool,
    pub indirect_lighting: bool,
    pub samples: u32,
    pub reprojection_factor: f32,
    pub shadows: bool,
    pub misc_bool: bool,
    pub misc_float: f32,
}

impl Default for TraceSettings {
    fn default() -> Self {
        Self {
            show_ray_steps: false,
            indirect_lighting: false,
            samples: 1,
            reprojection_factor: 0.75,
            shadows: true,
            misc_bool: false,
            misc_float: 1.0,
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
    pub indirect_lighting: u32,
    pub samples: u32,
    pub reprojection_factor: f32,
    pub shadows: u32,
    pub misc_bool: u32,
    pub misc_float: f32,
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
            indirect_lighting: settings.indirect_lighting as u32,
            samples: settings.samples,
            reprojection_factor: settings.reprojection_factor,
            shadows: settings.shadows as u32,
            misc_bool: settings.misc_bool as u32,
            misc_float: settings.misc_float,
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
        let voxel_data = render_world.get_resource::<VoxelData>().unwrap();

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
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
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
        let reprojection_bind_group_layout = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("reprojection bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let trace_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("trace pipeline".into()),
            layout: vec![
                voxel_bind_group_layout.clone(),
                trace_bind_group_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: TRACE_SHADER_HANDLE.typed(),
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
        let reprojection_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("reprojection pipeline".into()),
            layout: vec![
                trace_bind_group_layout.clone(),
                reprojection_bind_group_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: REPROJECTION_SHADER_HANDLE.typed(),
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
        let accumulation_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("accumulation pipeline".into()),
            layout: vec![
                trace_bind_group_layout.clone(),
                reprojection_bind_group_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: REPROJECTION_SHADER_HANDLE.typed(),
                shader_defs: Vec::new(),
                entry_point: "accumulation".into(),
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
        let reprojection_pipeline_id =
            cache.queue_render_pipeline(reprojection_pipeline_descriptor);
        let accumulation_pipeline_id =
            cache.queue_render_pipeline(accumulation_pipeline_descriptor);

        TracePipelineData {
            trace_pipeline_id,
            reprojection_pipeline_id,
            accumulation_pipeline_id,
            trace_bind_group_layout,
            reprojection_bind_group_layout,
        }
    }
}
