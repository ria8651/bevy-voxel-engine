use bevy::{
    asset::load_internal_asset,
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::ViewTarget,
        RenderApp, RenderSet,
    },
};
pub use node::DenoiseNode;

mod node;

pub const DENOISE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3278741884048607584);

pub struct DenoisePlugin;

impl Plugin for DenoisePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            DENOISE_SHADER_HANDLE,
            "../shaders/denoise.wgsl",
            Shader::from_wgsl
        );

        let pass_settings = [
            DenoisePassData::new(1.0, 0.08, 0.5, 0.1),
            DenoisePassData::new(2.0, 0.025, 0.5, 0.1),
            DenoisePassData::new(4.0, 0.015, 0.5, 0.1),
            // DenoisePassData::new(1.0, 0.15, 0.5, 0.1),
            // DenoisePassData::new(2.0, 0.10, 0.5, 0.1),
            // DenoisePassData::new(0.0, 1.0, 1.0, 1.0),
        ];

        app.insert_resource(DenoiseSettings { pass_settings })
            .add_plugin(ExtractResourcePlugin::<DenoiseSettings>::default());

        app.sub_app_mut(RenderApp)
            .init_resource::<DenoisePipeline>()
            .add_system(prepare_pass_data.in_set(RenderSet::Prepare));
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct DenoiseSettings {
    pub pass_settings: [DenoisePassData; 3],
}

fn prepare_pass_data(
    denoise_settings: Res<DenoiseSettings>,
    mut denoise_pipeline: ResMut<DenoisePipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    denoise_pipeline.pass_data.clear();
    denoise_pipeline
        .pass_data
        .push(denoise_settings.pass_settings[0]);
    denoise_pipeline
        .pass_data
        .push(denoise_settings.pass_settings[1]);
    denoise_pipeline
        .pass_data
        .push(denoise_settings.pass_settings[2]);
    denoise_pipeline
        .pass_data
        .write_buffer(&render_device, &render_queue);
}

#[derive(Resource)]
struct DenoisePipeline {
    bind_group_layout: BindGroupLayout,
    pass_data_bind_group_layout: BindGroupLayout,
    pipeline_id: CachedRenderPipelineId,
    uniform_buffer: Buffer,
    pass_data: DynamicUniformBuffer<DenoisePassData>,
}

#[derive(Component)]
struct ViewDenoisePipeline(CachedRenderPipelineId);

impl FromWorld for DenoisePipeline {
    fn from_world(render_world: &mut World) -> Self {
        let bind_group_layout = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("denoise bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                get_uniform_buffer_data().len() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let pass_data_bind_group_layout = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("denoise bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: BufferSize::new(DenoisePassData::SHADER_SIZE.into()),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("denoise pipeline".into()),
            layout: vec![
                bind_group_layout.clone(),
                pass_data_bind_group_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: DENOISE_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: ViewTarget::TEXTURE_FORMAT_HDR,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
        };

        let cache = render_world.resource_mut::<PipelineCache>();
        let pipeline_id = cache.queue_render_pipeline(pipeline_descriptor);

        let uniform_buffer = render_world
            .resource::<RenderDevice>()
            .create_buffer_with_data(&BufferInitDescriptor {
                label: Some("denoise uniform buffer"),
                contents: &get_uniform_buffer_data(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let pass_data = DynamicUniformBuffer::default();

        DenoisePipeline {
            bind_group_layout,
            pass_data_bind_group_layout,
            pipeline_id,
            uniform_buffer,
            pass_data,
        }
    }
}

#[derive(Clone, Copy, ShaderType)]
pub struct DenoisePassData {
    pub denoise_strength: f32,
    pub colour_phi: f32,
    pub normal_phi: f32,
    pub position_phi: f32,
    padding4: [UVec4; 15],
}
impl DenoisePassData {
    fn new(denoise_strength: f32, colour_phi: f32, normal_phi: f32, position_phi: f32) -> Self {
        Self {
            denoise_strength,
            colour_phi,
            normal_phi,
            position_phi,
            padding4: [UVec4::ZERO; 15],
        }
    }
}

fn get_uniform_buffer_data() -> Vec<u8> {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let offsets: [(f32, f32); 25] = [
        (-2.0, -2.0), (-1.0, -2.0), (0.0, -2.0), (1.0, -2.0), (2.0, -2.0),
        (-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0), (2.0, -1.0),
        (-2.0, 0.0),  (-1.0, 0.0),  (0.0, 0.0),  (1.0, 0.0),  (2.0, 0.0),
        (-2.0, 1.0),  (-1.0, 1.0),  (0.0, 1.0),  (1.0, 1.0),  (2.0, 1.0),
        (-2.0, 2.0),  (-1.0, 2.0),  (0.0, 2.0),  (1.0, 2.0),  (2.0, 2.0),
    ];

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let kernel: [f32; 25] = [
        1.0/256.0, 1.0/64.0, 3.0/128.0, 1.0/64.0, 1.0/256.0,
        1.0/64.0,  1.0/16.0, 3.0/32.0,  1.0/16.0, 1.0/64.0,
        3.0/128.0, 3.0/32.0, 9.0/64.0,  3.0/32.0, 3.0/128.0,
        1.0/64.0,  1.0/16.0, 3.0/32.0,  1.0/16.0, 1.0/64.0,
        1.0/256.0, 1.0/64.0, 3.0/128.0, 1.0/64.0, 1.0/256.0,
    ];

    let mut data = Vec::new();
    for i in 0..25 {
        data.extend_from_slice(&offsets[i].0.to_le_bytes());
        data.extend_from_slice(&offsets[i].1.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
    }
    for i in 0..25 {
        data.extend_from_slice(&kernel[i].to_le_bytes());
        data.extend_from_slice(&1f32.to_le_bytes());
        data.extend_from_slice(&1f32.to_le_bytes());
        data.extend_from_slice(&1f32.to_le_bytes());
    }

    data
}
