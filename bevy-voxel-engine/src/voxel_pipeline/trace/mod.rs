use super::voxel_world::VoxelData;
use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewTarget},
        RenderApp, RenderStage,
    },
};
pub use node::TraceNode;

mod node;

pub struct TracePlugin;

impl Plugin for TracePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<TraceSettings>::default());

        // setup custom render pipeline
        app.sub_app_mut(RenderApp)
            .init_resource::<TracePipeline>()
            .init_resource::<SpecializedRenderPipelines<TracePipeline>>()
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms)
            .add_system_to_stage(RenderStage::Queue, queue_trace_pipeline);
    }
}

#[derive(Resource)]
struct TracePipeline {
    shader: Handle<Shader>,
    voxel_bind_group_layout: BindGroupLayout,
    trace_bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
struct ViewTracePipeline(CachedRenderPipelineId);

#[derive(Component, Clone)]
pub struct TraceSettings {
    pub show_ray_steps: bool,
    pub indirect_lighting: bool,
    pub shadows: bool,
    pub samples: u32,
    pub misc_bool: bool,
    pub misc_float: f32,
}

impl ExtractComponent for TraceSettings {
    type Query = &'static TraceSettings;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Self {
        item.clone()
    }
}

#[repr(C)]
#[derive(Clone, ShaderType)]
pub struct TraceUniforms {
    pub camera: Mat4,
    pub camera_inverse: Mat4,
    pub time: f32,
    pub show_ray_steps: u32,
    pub indirect_lighting: u32,
    pub shadows: u32,
    pub samples: u32,
    pub misc_bool: u32,
    pub misc_float: f32,
}

#[derive(Component, Deref, DerefMut)]
struct ViewTraceUniformBuffer(UniformBuffer<TraceUniforms>);

fn prepare_uniforms(
    mut commands: Commands,
    query: Query<(Entity, &TraceSettings, &ExtractedView)>,
    time: Res<Time>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let elapsed = time.elapsed_seconds_f64();

    for (entity, settings, view) in query.iter() {
        let projection = view.projection;
        let inverse_projection = projection.inverse();
        let view = view.transform.compute_matrix();
        let inverse_view = view.inverse();

        let uniforms = TraceUniforms {
            camera: projection * inverse_view,
            camera_inverse: view * inverse_projection,
            time: elapsed as f32,
            show_ray_steps: settings.show_ray_steps as u32,
            indirect_lighting: settings.indirect_lighting as u32,
            shadows: settings.shadows as u32,
            samples: settings.samples,
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

impl FromWorld for TracePipeline {
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
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<TraceUniforms>() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
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
                ],
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