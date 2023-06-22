use crate::{
    voxel_pipeline::voxel_world::{VoxelData, VoxelUniforms},
    RenderGraphSettings,
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
    },
};
use std::{borrow::Cow, num::NonZeroU32};

pub struct MipNode;

#[derive(Resource)]
pub struct Pipeline {
    pipeline: CachedComputePipelineId,
    mip_bind_group_layout: BindGroupLayout,
}

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mip_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(VoxelUniforms::SHADER_SIZE.into()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::R16Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ],
            });

        // let voxel_bind_group_layout = world.resource::<VoxelData>().bind_group_layout.clone();
        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let mip_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("mip pipeline")),
            layout: vec![mip_bind_group_layout.clone()],
            shader: super::MIP_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: Cow::from("mip"),
            push_constant_ranges: vec![],
        });

        Pipeline {
            pipeline: mip_pipeline,
            mip_bind_group_layout,
        }
    }
}

impl render_graph::Node for MipNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let render_device = world.resource::<RenderDevice>();
        let voxel_data = world.resource::<VoxelData>();
        let voxel_uniforms = world.resource::<VoxelUniforms>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_graph_settings = world.resource::<RenderGraphSettings>();
        let mip_pipeline = world.resource::<Pipeline>();

        if !render_graph_settings.mip {
            return Ok(());
        }

        let pipeline = match pipeline_cache.get_compute_pipeline(mip_pipeline.pipeline) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let texture_view = voxel_data.mip_texture.create_view(&TextureViewDescriptor {
            mip_level_count: NonZeroU32::new(1),
            ..default()
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &mip_pipeline.mip_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: voxel_data.uniform_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&voxel_data.voxel_world),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&texture_view),
                },
            ],
        });

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let dispatch_size = voxel_uniforms.texture_size / 4;
        pass.set_bind_group(0, &bind_group, &[]);

        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);

        Ok(())
    }
}
