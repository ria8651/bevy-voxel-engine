use super::{AnimationData, ComputeData};
use crate::{voxel_pipeline::voxel_world::VoxelData, RenderGraphSettings};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::RenderContext,
    },
};
use std::borrow::Cow;

pub struct AnimationNode;

#[derive(Resource)]
pub struct Pipeline(CachedComputePipelineId);

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();
        let compute_bind_group_layout = world.resource::<ComputeData>().bind_group_layout.clone();

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("animation pipeline")),
            layout: Some(vec![voxel_bind_group_layout, compute_bind_group_layout]),
            shader: super::ANIMATION_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: Cow::from("animation"),
        });

        Pipeline(update_pipeline)
    }
}

impl render_graph::Node for AnimationNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let voxel_data = world.resource::<VoxelData>();
        let compute_data = world.resource::<ComputeData>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let animation_data = world.resource::<AnimationData>();
        let render_graph_settings = world.get_resource::<RenderGraphSettings>().unwrap();

        if !render_graph_settings.animation {
            return Ok(());
        }

        let pipeline = match pipeline_cache.get_compute_pipeline(world.resource::<Pipeline>().0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &voxel_data.bind_group, &[]);
        pass.set_bind_group(1, &compute_data.bind_group, &[]);

        let dispatch_size = (animation_data.dispatch_size as f32).cbrt().ceil() as u32;
        if dispatch_size > 0 {
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);
        }

        Ok(())
    }
}
