use super::ComputeData;
use crate::voxel_pipeline::voxel_world::{VoxelData, VoxelUniforms};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::RenderContext,
    },
};
use std::borrow::Cow;

pub struct AutomataNode;

#[derive(Resource)]
pub struct Pipeline(CachedComputePipelineId);

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();
        let compute_bind_group_layout = world.resource::<ComputeData>().bind_group_layout.clone();
        let shader = world.resource::<AssetServer>().load("compute/automata.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("automata pipeline")),
            layout: Some(vec![voxel_bind_group_layout, compute_bind_group_layout]),
            shader: shader,
            shader_defs: vec![],
            entry_point: Cow::from("automata"),
        });

        Pipeline(update_pipeline)
    }
}

impl render_graph::Node for AutomataNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let voxel_data = world.resource::<VoxelData>();
        let compute_data = world.resource::<ComputeData>();
        let voxel_uniforms = world.resource::<VoxelUniforms>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let dispatch_size = voxel_uniforms.texture_size / 4;

        let pipeline = match pipeline_cache.get_compute_pipeline(world.resource::<Pipeline>().0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &voxel_data.bind_group, &[]);
        pass.set_bind_group(1, &compute_data.bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);

        Ok(())
    }
}
