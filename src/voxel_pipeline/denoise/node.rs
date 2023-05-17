use super::{super::RenderGraphSettings, DenoisePassData, DenoisePipeline};
use crate::TraceSettings;
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_resource::*,
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

pub struct DenoiseNode {
    query: QueryState<(&'static ViewTarget, &'static TraceSettings), With<ExtractedView>>,
}

impl DenoiseNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for DenoiseNode {
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
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity("view")?;
        let pipeline_cache = world.resource::<PipelineCache>();
        let denoise_pipeline = world.resource::<DenoisePipeline>();
        let render_graph_settings = world.get_resource::<RenderGraphSettings>().unwrap();

        let (target, trace_uniforms) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        if !render_graph_settings.denoise || !trace_uniforms.indirect_lighting {
            return Ok(());
        }

        let pipeline = match pipeline_cache.get_render_pipeline(denoise_pipeline.pipeline_id) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        let accumulation = graph.get_input_texture("accumulation")?;
        let normal = graph.get_input_texture("normal")?;
        let position = graph.get_input_texture("position")?;

        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &denoise_pipeline.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: denoise_pipeline.uniform_buffer.as_entire_binding(),
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
        let source_bind_group =
            render_context
                .render_device()
                .create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &denoise_pipeline.pass_data_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: denoise_pipeline.pass_data.binding().unwrap(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(source),
                        },
                    ],
                });
        let destination_bind_group =
            render_context
                .render_device()
                .create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &denoise_pipeline.pass_data_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: denoise_pipeline.pass_data.binding().unwrap(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(destination),
                        },
                    ],
                });

        let source_descriptor = RenderPassDescriptor {
            label: Some("denoise pass"),
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
            label: Some("denoise pass"),
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

        let offset_size = u64::from(DenoisePassData::SHADER_SIZE) as u32;

        {
            let mut render_pass = render_context
                .command_encoder()
                .begin_render_pass(&destination_descriptor);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_bind_group(1, &source_bind_group, &[0]);
            render_pass.draw(0..3, 0..1);
        }
        {
            let mut render_pass = render_context
                .command_encoder()
                .begin_render_pass(&source_descriptor);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_bind_group(1, &destination_bind_group, &[offset_size]);
            render_pass.draw(0..3, 0..1);
        }
        {
            let mut render_pass = render_context
                .command_encoder()
                .begin_render_pass(&destination_descriptor);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_bind_group(1, &source_bind_group, &[2 * offset_size]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
