use super::{TracePipelineData, ViewTraceUniformBuffer};
use crate::voxel_pipeline::{voxel_world::VoxelData, RenderGraphSettings};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, SlotInfo, SlotType},
        render_resource::*,
        view::{ExtractedView, ViewTarget},
    },
};

pub struct TraceNode {
    query: QueryState<(&'static ViewTarget, &'static ViewTraceUniformBuffer), With<ExtractedView>>,
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
        vec![
            SlotInfo::new("view", SlotType::Entity),
            SlotInfo::new("accumulation", SlotType::TextureView),
            SlotInfo::new("normal", SlotType::TextureView),
            SlotInfo::new("position", SlotType::TextureView),
        ]
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
        let trace_pipeline_data = world.get_resource::<TracePipelineData>().unwrap();
        let render_graph_settings = world.get_resource::<RenderGraphSettings>().unwrap();

        if !render_graph_settings.trace {
            return Ok(());
        }

        let (target, trace_uniform_buffer) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => panic!("Voxel camera missing component!"),
        };

        let trace_pipeline =
            match pipeline_cache.get_render_pipeline(trace_pipeline_data.trace_pipeline_id) {
                Some(pipeline) => pipeline,
                None => return Ok(()),
            };
        let reprojection_pipeline = match pipeline_cache
            .get_render_pipeline(trace_pipeline_data.reprojection_pipeline_id)
        {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        let accumulation = graph.get_input_texture("accumulation")?;
        let normal = graph.get_input_texture("normal")?;
        let position = graph.get_input_texture("position")?;

        let trace_bind_group = render_context
            .render_device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &trace_pipeline_data.trace_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: trace_uniform_buffer.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&accumulation),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&normal),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&position),
                    },
                ],
            });
        let reprojection_bind_group = render_context
            .render_device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &trace_pipeline_data.reprojection_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&source),
                    },
                ],
            });

        let source_descriptor = RenderPassDescriptor {
            label: Some("trace pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: source,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        let destination_descriptor = RenderPassDescriptor {
            label: Some("trace pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        {
            let mut render_pass = render_context
                .command_encoder
                .begin_render_pass(&source_descriptor);

            render_pass.set_bind_group(0, &voxel_data.bind_group, &[]);
            render_pass.set_bind_group(1, &trace_bind_group, &[]);

            render_pass.set_pipeline(trace_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        {
            let mut render_pass = render_context
                .command_encoder
                .begin_render_pass(&destination_descriptor);

            render_pass.set_bind_group(0, &trace_bind_group, &[]);
            render_pass.set_bind_group(1, &reprojection_bind_group, &[]);

            render_pass.set_pipeline(reprojection_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
