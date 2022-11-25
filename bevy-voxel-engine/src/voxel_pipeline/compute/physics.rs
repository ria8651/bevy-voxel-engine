use super::{ComputeData, ExtractedPhysicsData};
use crate::voxel_pipeline::voxel_world::VoxelData;
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderQueue},
    },
};
use std::borrow::Cow;

pub struct PhysicsNode;

#[derive(Resource)]
pub struct Pipeline(CachedComputePipelineId);

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();
        let compute_bind_group_layout = world.resource::<ComputeData>().bind_group_layout.clone();
        let shader = world.resource::<AssetServer>().load("compute/physics.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("physics pipeline")),
            layout: Some(vec![voxel_bind_group_layout, compute_bind_group_layout]),
            shader: shader,
            shader_defs: vec![],
            entry_point: Cow::from("physics"),
        });

        Pipeline(update_pipeline)
    }
}

impl render_graph::Node for PhysicsNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let voxel_data = world.resource::<VoxelData>();
        let compute_data = world.resource::<ComputeData>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_queue = world.resource::<RenderQueue>();
        let extracted_physics_data = world.resource::<ExtractedPhysicsData>();

        let pipeline = match pipeline_cache.get_compute_pipeline(world.resource::<Pipeline>().0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        // copy physics data to the buffer
        render_queue.write_buffer(
            &extracted_physics_data.physics_buffer,
            0,
            bytemuck::cast_slice(&extracted_physics_data.data),
        );

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &voxel_data.bind_group, &[]);
        pass.set_bind_group(1, &compute_data.bind_group, &[]);

        let dispatch_size = (extracted_physics_data.data[0] as f32).cbrt().ceil() as u32;
        if dispatch_size > 0 {
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);
        }

        Ok(())
    }
}
