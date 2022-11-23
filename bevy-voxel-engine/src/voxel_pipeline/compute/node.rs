use crate::voxel_pipeline::{
    compute::{
        ComputeBindGroup, ComputeMeta, ComputePipeline, ExtractedAnimationData, ExtractedGH,
        ExtractedPhysicsData,
    },
    trace::{ExtractedUniforms, TraceData},
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::*,
        renderer::RenderQueue,
        renderer::{RenderContext},
    },
};

#[derive(PartialEq, Eq)]
enum ComputeState {
    Loading,
    Init,
    Update,
}

pub struct ComputeNode {
    state: ComputeState,
}

impl Default for ComputeNode {
    fn default() -> Self {
        Self {
            state: ComputeState::Loading,
        }
    }
}

impl render_graph::Node for ComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            ComputeState::Loading => {
                // if the update pipeline is ready the other's probably are too lol
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = ComputeState::Init;
                }
            }
            ComputeState::Init => {
                self.state = ComputeState::Update;
            }
            ComputeState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<ComputeBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let render_queue = world.resource::<RenderQueue>();
        let trace_data = world.resource::<TraceData>();
        let compute_meta = world.resource::<ComputeMeta>();
        let extracted_gh = world.resource::<ExtractedGH>();
        let extracted_animation_data = world.resource::<ExtractedAnimationData>();
        let extracted_physics_data = world.resource::<ExtractedPhysicsData>();
        let uniforms = world.resource::<ExtractedUniforms>();

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            ComputeState::Loading => {}
            ComputeState::Init | ComputeState::Update => {
                if uniforms.enable_compute != 0 || self.state == ComputeState::Init {
                    render_queue.write_buffer(
                        &compute_meta.physics_data,
                        0,
                        bytemuck::cast_slice(&extracted_physics_data.data),
                    );
                    render_queue.write_buffer(
                        &compute_meta.animation_data,
                        0,
                        bytemuck::cast_slice(&extracted_animation_data.data),
                    );
                    render_queue.write_buffer(
                        &trace_data.grid_heierachy,
                        0,
                        bytemuck::cast_slice(&vec![0u8; extracted_gh.buffer_size]),
                    );

                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();
                    let automata_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.automata_pipeline)
                        .unwrap();
                    let animation_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.animation_pipeline)
                        .unwrap();
                    let rebuild_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.rebuild_pipeline)
                        .unwrap();
                    let physics_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.physics_pipeline)
                        .unwrap();

                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                    );

                    pass.set_pipeline(automata_pipeline);
                    pass.dispatch_workgroups(
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                    );

                    let dispatch_size =
                        (extracted_animation_data.data[0] as f32).cbrt().ceil() as u32;
                    if dispatch_size > 0 {
                        pass.set_pipeline(animation_pipeline);
                        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);
                    }

                    pass.set_pipeline(rebuild_pipeline);
                    pass.dispatch_workgroups(
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                        extracted_gh.texture_size / 4,
                    );

                    let dispatch_size =
                        (extracted_physics_data.data[0] as f32).cbrt().ceil() as u32;
                    if dispatch_size > 0 {
                        pass.set_pipeline(physics_pipeline);
                        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);
                    }
                }
            }
        }

        Ok(())
    }
}
