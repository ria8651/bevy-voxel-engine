use super::trace::{ExtractedUniforms, TraceData};
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::RenderDevice,
        RenderApp,
    },
    utils::HashMap,
};

pub mod automata;
pub mod clear;
pub mod physics;
pub mod rebuild;

const MAX_TYPE_BUFFER_DATA: usize = 1000000; // 4mb

pub struct ComputeResourcesPlugin;

impl Plugin for ComputeResourcesPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let trace_data = app.sub_app(RenderApp).world.resource::<TraceData>();

        let physics_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vec![0u32; MAX_TYPE_BUFFER_DATA]),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        });

        let bind_group_layout =
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
                ],
            });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: trace_data.uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: physics_buffer.as_entire_binding(),
                },
            ],
        });

        app.insert_resource(ExtractedPhysicsData {
            data: vec![0],
            entities: HashMap::new(),
            physics_buffer,
        })
        .add_plugin(ExtractResourcePlugin::<ExtractedPhysicsData>::default());

        app.sub_app_mut(RenderApp)
            .insert_resource(ComputeData {
                bind_group_layout,
                bind_group,
            })
            .init_resource::<clear::Pipeline>()
            .init_resource::<rebuild::Pipeline>()
            .init_resource::<automata::Pipeline>()
            .init_resource::<physics::Pipeline>();
    }
}

#[derive(Clone, Resource, ExtractResource)]
pub struct ExtractedPhysicsData {
    pub data: Vec<u32>,
    pub entities: HashMap<Entity, usize>,
    pub physics_buffer: Buffer,
}

#[derive(Resource)]
pub struct ComputeData {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}
