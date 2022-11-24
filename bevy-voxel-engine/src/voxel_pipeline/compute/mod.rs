use super::{voxel_world::VoxelData, trace::{TraceData, ExtractedUniforms}};
use crate::animation;
use bevy::{
    app::CoreStage,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::RenderDevice,
        RenderApp, RenderStage,
    },
};
use std::{borrow::Cow, collections::HashMap};

pub mod node;

const MAX_ANIMATION_DATA: usize = 1000000; // 4mb

pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();

        // compute data buffer
        let physics_data = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vec![0u32; MAX_ANIMATION_DATA]),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        });

        // compute data buffer
        let animation_data = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vec![0u32; MAX_ANIMATION_DATA]),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // setup world
        app.add_system(animation::extract_animation_data)
            .add_system_to_stage(CoreStage::PreUpdate, animation::insert_physics_data)
            .add_system_to_stage(CoreStage::PostUpdate, animation::extract_physics_data)
            .insert_resource(ComputeMeta {
                physics_data,
                animation_data,
            })
            .insert_resource(ExtractedPhysicsData {
                data: vec![0],
                entities: HashMap::new(),
            })
            .add_plugin(ExtractResourcePlugin::<ExtractedAnimationData>::default())
            .add_plugin(ExtractResourcePlugin::<ExtractedPhysicsData>::default())
            .add_plugin(ExtractResourcePlugin::<ComputeMeta>::default());

        // setup render world
        app.sub_app_mut(RenderApp)
            .init_resource::<ComputePipeline>()
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);
    }
}

#[derive(Clone, Resource, ExtractResource)]
pub struct ExtractedAnimationData {
    pub data: Vec<u32>,
}

#[derive(Clone, Resource, ExtractResource)]
pub struct ExtractedPhysicsData {
    pub data: Vec<u32>,
    pub entities: HashMap<Entity, usize>,
}

#[derive(Clone, Resource, ExtractResource)]
pub struct ComputeMeta {
    pub physics_data: Buffer,
    pub animation_data: Buffer,
}

#[derive(Resource)]
pub struct ExtractedGH {
    pub buffer_size: usize,
    pub texture_size: u32,
}

#[derive(Resource)]
pub struct ComputePipeline {
    compute_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
    automata_pipeline: CachedComputePipelineId,
    physics_pipeline: CachedComputePipelineId,
    animation_pipeline: CachedComputePipelineId,
    rebuild_pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let voxel_data = world.resource::<VoxelData>();

        let compute_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("compute bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
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
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                ],
            });
        let voxel_bind_group_layout = voxel_data.bind_group_layout.clone();

        let compute_shader = world.resource::<AssetServer>().load("compute.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("update_pipeline")),
            layout: Some(vec![
                voxel_bind_group_layout.clone(),
                compute_bind_group_layout.clone(),
            ]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });
        let automata_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("automata_pipeline")),
            layout: Some(vec![
                voxel_bind_group_layout.clone(),
                compute_bind_group_layout.clone(),
            ]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("automata"),
        });
        let physics_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("physics_pipeline")),
            layout: Some(vec![
                voxel_bind_group_layout.clone(),
                compute_bind_group_layout.clone(),
            ]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update_physics"),
        });
        let animation_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("animation_pipeline")),
            layout: Some(vec![
                voxel_bind_group_layout.clone(),
                compute_bind_group_layout.clone(),
            ]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update_animation"),
        });
        let rebuild_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("rebuild_pipeline")),
            layout: Some(vec![
                voxel_bind_group_layout.clone(),
                compute_bind_group_layout.clone(),
            ]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("rebuild_gh"),
        });

        ComputePipeline {
            compute_bind_group_layout,
            update_pipeline,
            automata_pipeline,
            physics_pipeline,
            animation_pipeline,
            rebuild_pipeline,
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
struct ComputeBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    compute_pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    compute_meta: Res<ComputeMeta>,
    trace_data: Res<TraceData>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &compute_pipeline.compute_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: trace_data.uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: compute_meta.physics_data.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: compute_meta.animation_data.as_entire_binding(),
            },
        ],
    });
    commands.insert_resource(ComputeBindGroup(bind_group));
}
