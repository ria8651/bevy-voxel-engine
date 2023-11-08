use crate::{
    load::GH,
    voxel_pipeline::voxel_world::{VoxelData, VoxelUniforms},
    RenderGraphSettings,
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderQueue},
    },
};
use std::borrow::Cow;

pub struct RebuildNode;

#[derive(Resource)]
pub struct Pipeline(CachedComputePipelineId);

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();

        let asset_server = world.resource_mut::<AssetServer>();
        let shader = asset_server.load("embedded://bevy_voxel_engine/voxel_pipeline/compute/rebuild.wgsl");
        
        let pipeline_cache = world.resource_mut::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("rebuild pipeline")),
            layout: vec![voxel_bind_group_layout],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("rebuild_gh"),
            push_constant_ranges: vec![],
        });

        Pipeline(update_pipeline)
    }
}

impl render_graph::Node for RebuildNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let voxel_data = world.resource::<VoxelData>();
        let voxel_uniforms = world.resource::<VoxelUniforms>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_queue = world.resource::<RenderQueue>();
        let dispatch_size = voxel_uniforms.texture_size / 4;
        let render_graph_settings = world.resource::<RenderGraphSettings>();

        if !render_graph_settings.rebuild {
            return Ok(());
        }

        let mut levels = [0; 8];
        for i in 0..8 {
            levels[i] = voxel_uniforms.levels[i].x;
        }
        let gh_size = GH::get_buffer_size_from_levels(&levels);

        let pipeline = match pipeline_cache.get_compute_pipeline(world.resource::<Pipeline>().0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        // Clear the old grid hierarchy so we can build a new one
        render_queue.write_buffer(
            &voxel_data.grid_hierarchy,
            0,
            bytemuck::cast_slice(&vec![0u8; gh_size]),
        );

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &voxel_data.bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);

        Ok(())
    }
}
