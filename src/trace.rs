use super::load::GH;
use bevy::{
    core_pipeline::Transparent3d,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, Msaa},
        RenderApp, RenderStage,
    },
};

pub struct Tracer;

impl Plugin for Tracer {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let render_queue = app.world.resource::<RenderQueue>();

        // uniforms
        let uniforms = Uniforms {
            resolution: Vec4::default(),
            camera: Mat4::default(),
            camera_inverse: Mat4::default(),
            time: 0.0,
            levels: [0; 8],
            offsets: [0; 8],
            texture_size: 0,
            pallete: [PalleteEntry::default(); 256],
            padding: [0; 2],
        };
        let uniform = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniforms]),
            // contents: resolution.as_std140().as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // storage
        let gh = app.world.resource::<GH>();

        let storage = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &gh.data,
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // texture
        let texture = render_device.create_texture_with_data(
            render_queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: gh.texture_size,
                    height: gh.texture_size,
                    depth_or_array_layers: gh.texture_size,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: TextureFormat::R8Uint,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
            },
            &gh.texture_data,
        );
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        // println!("{}", render_device.limits().max_storage_buffer_binding_size);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .insert_resource(TraceMeta {
                uniform,
                storage,
                _texture: texture,
                texture_view,
                bind_group: None,
            })
            .init_resource::<TracePipeline>()
            .init_resource::<SpecializedMeshPipelines<TracePipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_uniforms)
            .add_system_to_stage(RenderStage::Extract, extract_trace_material)
            .add_system_to_stage(RenderStage::Prepare, prepare_time)
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Queue, queue_trace_bind_group);
        // .insert_resource(Buffers { uniform, storage })
        // .add_plugin(MaterialPlugin::<TraceMaterial>::default());
    }
}

pub struct ShaderTimer(pub Timer);

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PalleteEntry {
    pub colour: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
    resolution: Vec4,
    camera: Mat4,
    camera_inverse: Mat4,
    time: f32,
    levels: [u32; 8],
    offsets: [u32; 8],
    texture_size: u32,
    pallete: [PalleteEntry; 256],
    padding: [u32; 2],
}

// extract the passed time into a resource in the render world
fn extract_uniforms(
    mut commands: Commands,
    windows: Res<Windows>,
    main_cam: Query<(&Transform, &PerspectiveProjection), With<super::MainCamera>>,
    gh: Res<GH>,
    mut shader_timer: ResMut<ShaderTimer>,
    time: Res<Time>,
) {
    let window = windows.primary();
    let resolution = Vec4::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
        0.0,
        0.0,
    );

    let (transform, _perspective) = main_cam.single();

    let camera = Mat4::IDENTITY;
    let camera_inverse = transform.compute_matrix();

    shader_timer.0.tick(time.delta());

    commands.insert_resource(Uniforms {
        resolution,
        camera,
        camera_inverse,
        time: shader_timer.0.elapsed_secs(),
        levels: gh.levels,
        offsets: gh.get_offsets(),
        texture_size: gh.texture_size,
        pallete: gh.pallete,
        padding: [0; 2],
    });
}

// extract the `CustomMaterial` component into the render world
fn extract_trace_material(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    mut query: Query<Entity, With<TraceMaterial>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for entity in query.iter_mut() {
        values.push((entity, (TraceMaterial,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

// write the extracted time into the corresponding uniform buffer
fn prepare_time(
    uniforms: Res<Uniforms>,
    time_meta: ResMut<TraceMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &time_meta.uniform,
        0,
        bytemuck::cast_slice(&[*uniforms.as_ref()]),
    );
}

fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<TracePipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<TracePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<TraceMaterial>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let key = MeshPipelineKey::from_msaa_samples(msaa.samples)
        | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);

    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, mesh_uniform, mesh_handle) in material_meshes.iter() {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                });
            }
        }
    }
}

// create a bind group for the time uniform buffer
fn queue_trace_bind_group(
    render_device: Res<RenderDevice>,
    mut trace_meta: ResMut<TraceMeta>,
    pipeline: Res<TracePipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.trace_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: trace_meta.uniform.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: trace_meta.storage.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&trace_meta.texture_view),
            },
        ],
    });
    trace_meta.bind_group = Some(bind_group);
}

struct TraceMeta {
    uniform: Buffer,
    storage: Buffer,
    _texture: Texture,
    texture_view: TextureView,
    bind_group: Option<BindGroup>,
}

pub struct TracePipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    trace_bind_group_layout: BindGroupLayout,
}

impl FromWorld for TracePipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shader.wgsl");

        let render_device = world.get_resource_mut::<RenderDevice>().unwrap();
        let trace_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R8Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ],
            });

        let mesh_pipeline = world.get_resource::<MeshPipeline>().unwrap();

        TracePipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            trace_bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for TracePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.trace_bind_group_layout.clone(),
        ]);
        Ok(descriptor)
    }
}

// This is the struct that will be passed to your shader
#[derive(Component)]
pub struct TraceMaterial;

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetTraceBindGroup<2>,
    DrawMesh,
);

struct SetTraceBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTraceBindGroup<I> {
    type Param = SRes<TraceMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        trace_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let trace_bind_group = trace_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, trace_bind_group, &[]);

        RenderCommandResult::Success
    }
}
