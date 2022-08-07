use super::trace;
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::RenderQueue,
        renderer::{RenderContext, RenderDevice},
        RenderApp, RenderStage,
    },
};
use std::borrow::Cow;

pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GameOfLifePipeline>()
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("game_of_life", GameOfLifeNode::default());
        render_graph
            .add_node_edge(
                "game_of_life",
                bevy::render::main_graph::node::CAMERA_DRIVER,
            )
            .unwrap();
    }
}

enum GameOfLifeState {
    Loading,
    Update,
}

struct GameOfLifeNode {
    state: GameOfLifeState,
}

impl Default for GameOfLifeNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GameOfLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    if let CachedPipelineState::Ok(_) =
                        pipeline_cache.get_compute_pipeline_state(pipeline.rebuild_pipeline)
                    {
                        self.state = GameOfLifeState::Update;
                    }
                }
            }
            GameOfLifeState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();
        let render_queue = world.resource::<RenderQueue>();
        let trace_meta = world.resource::<trace::TraceMeta>();

        let uniforms = world.resource::<trace::ExtractedUniforms>();

        if uniforms.misc_bool != 0 {
            render_queue.write_buffer(
                &trace_meta.storage,
                0,
                bytemuck::cast_slice(&[0u8; 37440]),
            );
        }

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading => {}
            GameOfLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                let rebuild_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.rebuild_pipeline)
                    .unwrap();

                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(128, 128, 128);

                pass.set_pipeline(rebuild_pipeline);
                pass.dispatch_workgroups(128, 128, 128);
            }
        }

        Ok(())
    }
}

pub struct GameOfLifePipeline {
    compute_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
    rebuild_pipeline: CachedComputePipelineId,
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let compute_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(std::mem::size_of::<
                                trace::ExtractedUniforms,
                            >()
                                as u64),
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
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R8Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ],
            });

        let compute_shader = world.resource::<AssetServer>().load("compute.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![compute_bind_group_layout.clone()]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });
        let rebuild_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![compute_bind_group_layout.clone()]),
            shader: compute_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("rebuild_gh"),
        });

        GameOfLifePipeline {
            compute_bind_group_layout,
            update_pipeline,
            rebuild_pipeline,
        }
    }
}

struct GameOfLifeImageBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    compute_pipeline: Res<GameOfLifePipeline>,
    render_device: Res<RenderDevice>,
    trace_meta: Res<trace::TraceMeta>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &compute_pipeline.compute_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: trace_meta.uniform.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: trace_meta.storage.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&trace_meta.texture_view),
            },
        ],
    });
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}
