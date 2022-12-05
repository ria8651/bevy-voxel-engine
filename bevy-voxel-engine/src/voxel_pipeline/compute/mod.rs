use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
    utils::HashMap,
};

pub mod animation;
pub mod automata;
pub mod clear;
pub mod physics;
pub mod rebuild;

const MAX_TYPE_BUFFER_DATA: usize = 1000000; // 4mb

pub struct ComputeResourcesPlugin;

impl Plugin for ComputeResourcesPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let render_queue = app.world.resource::<RenderQueue>();

        let mut uniform_buffer = UniformBuffer::from(ComputeUniforms {
            time: 0.0,
            delta_time: 0.0,
        });
        uniform_buffer.write_buffer(render_device, render_queue);

        let physics_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vec![0u32; MAX_TYPE_BUFFER_DATA]),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        });
        let animation_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vec![0u32; MAX_TYPE_BUFFER_DATA]),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
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
                            min_binding_size: BufferSize::new(ComputeUniforms::SHADER_SIZE.into()),
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

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: physics_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: animation_buffer.as_entire_binding(),
                },
            ],
        });

        app.insert_resource(PhysicsData {
            data: vec![0],
            entities: HashMap::new(),
            physics_buffer,
        })
        .insert_resource(AnimationData {
            distpatch_size: 0,
            animation_buffer,
        })
        .add_plugin(ExtractResourcePlugin::<PhysicsData>::default())
        .add_plugin(ExtractResourcePlugin::<AnimationData>::default());

        app.sub_app_mut(RenderApp)
            .insert_resource(ComputeData {
                bind_group_layout,
                bind_group,
                uniform_buffer,
            })
            .init_resource::<clear::Pipeline>()
            .init_resource::<rebuild::Pipeline>()
            .init_resource::<automata::Pipeline>()
            .init_resource::<physics::Pipeline>()
            .init_resource::<animation::Pipeline>()
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms);
    }
}

fn prepare_uniforms(
    time: Res<Time>,
    mut compute_data: ResMut<ComputeData>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let uniforms = ComputeUniforms {
        time: time.elapsed_seconds_f64() as f32,
        delta_time: time.delta_seconds() as f32,
    };
    compute_data.uniform_buffer.set(uniforms);
    compute_data
        .uniform_buffer
        .write_buffer(&render_device, &render_queue);
}

#[derive(Resource, ShaderType)]
struct ComputeUniforms {
    time: f32,
    delta_time: f32,
}

#[derive(Clone, Resource, ExtractResource)]
pub struct PhysicsData {
    pub data: Vec<u32>,
    pub entities: HashMap<Entity, usize>,
    pub physics_buffer: Buffer,
}

#[derive(Clone, Resource, ExtractResource)]
pub struct AnimationData {
    pub distpatch_size: u32,
    pub animation_buffer: Buffer,
}

#[derive(Resource)]
pub struct ComputeData {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    uniform_buffer: UniformBuffer<ComputeUniforms>,
}
