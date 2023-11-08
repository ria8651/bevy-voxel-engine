use crate::{
    voxel_pipeline::voxel_world::{VoxelData, VoxelUniforms},
    RenderGraphSettings,
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::RenderContext,
    },
};
use std::borrow::Cow;

pub struct ClearNode;

#[derive(Resource)]
pub struct Pipeline(CachedComputePipelineId);

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("clear pipeline")),
            layout: vec![voxel_bind_group_layout],
            shader: super::CLEAR_SHADER_HANDLE,
            shader_defs: vec![],
            entry_point: Cow::from("clear"),
            push_constant_ranges: vec![],
        });

        Pipeline(update_pipeline)
    }
}

impl render_graph::Node for ClearNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let voxel_data = world.resource::<VoxelData>();
        let voxel_uniforms = world.resource::<VoxelUniforms>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let dispatch_size = voxel_uniforms.texture_size / 4;
        let render_graph_settings = world.resource::<RenderGraphSettings>();

        if !render_graph_settings.clear {
            return Ok(());
        }

        let pipeline = match pipeline_cache.get_compute_pipeline(world.resource::<Pipeline>().0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &voxel_data.bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);

        Ok(())
    }
}
