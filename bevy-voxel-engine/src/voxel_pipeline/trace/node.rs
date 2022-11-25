use crate::voxel_pipeline::voxel_world::VoxelData;

use super::{TraceData, TracePipeline, ViewTracePipeline};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        render_graph::{self, SlotInfo, SlotType},
        render_resource::*,
        view::{ExtractedView, ViewTarget},
    },
};

pub struct TraceNode {
    query: QueryState<
        (
            &'static ViewTarget,
            &'static ViewTracePipeline,
            &'static Camera3d,
        ),
        With<ExtractedView>,
    >,
}

impl TraceNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for TraceNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new("view", SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let view_entity = graph.get_input_entity("view")?;
        let pipeline_cache = world.resource::<PipelineCache>();
        let voxel_data = world.get_resource::<VoxelData>().unwrap();
        let trace_pipeline = world.get_resource::<TracePipeline>().unwrap();
        let trace_data = world.get_resource::<TraceData>().unwrap();

        let (target, pipeline, camera_3d) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let pipeline = match pipeline_cache.get_render_pipeline(pipeline.0) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let bind_group = render_context
            .render_device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &trace_pipeline.trace_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: trace_data.uniform_buffer.as_entire_binding(),
                }],
            });

        let pass_descriptor = RenderPassDescriptor {
            label: Some("trace pass"),
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: match camera_3d.clear_color {
                    ClearColorConfig::Default => {
                        LoadOp::Clear(world.resource::<ClearColor>().0.into())
                    }
                    ClearColorConfig::Custom(color) => LoadOp::Clear(color.into()),
                    ClearColorConfig::None => LoadOp::Load,
                },
                store: true,
            }))],
            depth_stencil_attachment: None,
        };

        let mut render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        render_pass.set_bind_group(0, &voxel_data.bind_group, &[]);
        render_pass.set_bind_group(1, &bind_group, &[]);

        render_pass.set_pipeline(pipeline);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}