use super::{compute, load::GH};
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        event::Events,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        camera::Projection,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, Msaa, NoFrustumCulling},
        RenderApp, RenderStage,
    },
    window::WindowResized,
};

pub struct Tracer;

impl Plugin for Tracer {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let render_queue = app.world.resource::<RenderQueue>();

        // uniforms
        let uniform = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ExtractedUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // screen texture
        let window = app.world.resource::<Windows>().primary();
        let screen_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: window.physical_width(),
                height: window.physical_height(),
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING,
        });
        let screen_texture_view = screen_texture.create_view(&TextureViewDescriptor::default());

        // storage
        let gh = app.world.resource::<GH>();

        let storage = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; gh.get_final_length() as usize / 8],
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
                format: TextureFormat::R16Uint,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
            },
            &gh.texture_data.clone(),
        );
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        // uniforms
        let uniforms_struct = Uniforms {
            pallete: gh.pallete,
            portals: [ExtractedPortal::default(); 32],
            resolution: Vec4::default(),
            last_camera: Mat4::default(),
            camera: Mat4::default(),
            camera_inverse: Mat4::default(),
            levels: gh.levels,
            offsets: gh.get_offsets(),
            time: 0.0,
            texture_size: gh.texture_size,
            show_ray_steps: false,
            indirect_lighting: false,
            shadows: true,
            accumulation_frames: 20.0,
            freeze: false,
            enable_compute: true,
            skybox: false,
            misc_bool: false,
            misc_float: 34.0,
        };

        // println!(
        //     "{:?}",
        //     render_device.get_supported_read_only_binding_type(4)
        // );

        // As the render world can no longer acces the main world we have to add seperate plugins to the main world
        app.add_startup_system(setup)
            .add_system(update_uniforms)
            .add_system(resize_system)
            .insert_resource(uniforms_struct)
            .add_plugin(ExtractComponentPlugin::<TraceMaterial>::default())
            .add_plugin(ExtractResourcePlugin::<ExtractedUniforms>::default())
            .add_plugin(ExtractResourcePlugin::<ResizeEvent>::default());

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .insert_resource(TraceMeta {
                uniform,
                storage,
                screen_texture_view,
                texture_view,
                bind_group: None,
            })
            .init_resource::<TracePipeline>()
            .init_resource::<SpecializedMeshPipelines<TracePipeline>>()
            .add_system_to_stage(RenderStage::Prepare, resize_prepare)
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms)
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Queue, queue_trace_bind_group);
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Plane { size: 2.0 })),
        Transform::from_xyz(0.0, 0.0, 0.001).looking_at(Vec3::Y, Vec3::Z),
        GlobalTransform::default(),
        TraceMaterial,
        Visibility::default(),
        ComputedVisibility::default(),
        NoFrustumCulling,
    ));
}

pub struct TraceMeta {
    pub uniform: Buffer,
    pub storage: Buffer,
    screen_texture_view: TextureView,
    pub texture_view: TextureView,
    bind_group: Option<BindGroup>,
}

pub struct ShaderTimer(pub Timer);

pub struct LastFrameData {
    pub last_camera: Mat4,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PalleteEntry {
    pub colour: [f32; 4],
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ExtractedPortal {
    pub pos: [f32; 4],
    pub other_pos: [f32; 4],
    pub normal: [f32; 4],
    pub other_normal: [f32; 4],
}

pub struct Uniforms {
    pub pallete: [PalleteEntry; 256],
    pub portals: [ExtractedPortal; 32],
    pub resolution: Vec4,
    pub last_camera: Mat4,
    pub camera: Mat4,
    pub camera_inverse: Mat4,
    pub levels: [u32; 8],
    pub offsets: [u32; 8],
    pub time: f32,
    pub texture_size: u32,
    pub show_ray_steps: bool,
    pub indirect_lighting: bool,
    pub shadows: bool,
    pub accumulation_frames: f32,
    pub freeze: bool,
    pub enable_compute: bool,
    pub skybox: bool,
    pub misc_bool: bool,
    pub misc_float: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ExtractedUniforms {
    pallete: [PalleteEntry; 256],
    portals: [ExtractedPortal; 32],
    resolution: Vec4,
    last_camera: Mat4,
    camera: Mat4,
    camera_inverse: Mat4,
    levels: [u32; 8],
    offsets: [u32; 8],
    time: f32,
    texture_size: u32,
    show_ray_steps: u32,
    indirect_lighting: u32,
    shadows: u32,
    accumulation_frames: f32,
    freeze: u32,
    pub enable_compute: u32,
    skybox: u32,
    misc_bool: u32,
    misc_float: f32,
    padding: [u32; 1],
}

impl ExtractResource for ExtractedUniforms {
    type Source = Uniforms;

    fn extract_resource(uniforms: &Self::Source) -> Self {
        ExtractedUniforms {
            pallete: uniforms.pallete,
            portals: uniforms.portals,
            resolution: uniforms.resolution,
            last_camera: uniforms.last_camera,
            camera: uniforms.camera,
            camera_inverse: uniforms.camera_inverse,
            levels: uniforms.levels,
            offsets: uniforms.offsets,
            time: uniforms.time,
            texture_size: uniforms.texture_size,
            show_ray_steps: uniforms.show_ray_steps as u32,
            indirect_lighting: uniforms.indirect_lighting as u32,
            shadows: uniforms.shadows as u32,
            accumulation_frames: uniforms.accumulation_frames,
            freeze: uniforms.freeze as u32,
            enable_compute: uniforms.enable_compute as u32,
            skybox: uniforms.skybox as u32,
            misc_bool: uniforms.misc_bool as u32,
            misc_float: uniforms.misc_float,
            padding: [0; 1],
        }
    }
}

fn update_uniforms(
    mut uniforms: ResMut<Uniforms>,
    windows: Res<Windows>,
    main_cam: Query<(&Transform, &Projection), With<super::MainCamera>>,
    mut shader_timer: ResMut<ShaderTimer>,
    time: Res<Time>,
    mut last_frame_data: ResMut<LastFrameData>,
) {
    let window = windows.primary();
    uniforms.resolution = Vec4::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
        0.0,
        0.0,
    );

    let (transform, _perspective) = main_cam.single();

    let transform = Transform {
        translation: compute::world_to_render(transform.translation, uniforms.texture_size),
        ..*transform
    };

    uniforms.camera_inverse = transform.compute_matrix();
    uniforms.camera = uniforms.camera_inverse.inverse();
    uniforms.last_camera = last_frame_data.last_camera;
    if !uniforms.freeze {
        last_frame_data.last_camera = uniforms.camera;
    }

    shader_timer.0.tick(time.delta());
    uniforms.time = shader_timer.0.elapsed_secs();
}

// write the extracted time into the corresponding uniform buffer
fn prepare_uniforms(
    extraced_uniforms: Res<ExtractedUniforms>,
    trace_meta: ResMut<TraceMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &trace_meta.uniform,
        0,
        bytemuck::cast_slice(&[*extraced_uniforms.as_ref()]),
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
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&trace_meta.screen_texture_view),
            },
        ],
    });
    trace_meta.bind_group = Some(bind_group);
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
                                std::mem::size_of::<ExtractedUniforms>() as u64,
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
                            format: TextureFormat::R16Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2Array,
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

impl ExtractComponent for TraceMaterial {
    type Query = Read<TraceMaterial>;

    type Filter = ();

    fn extract_component(_: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        TraceMaterial
    }
}

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

fn resize_system(
    mut commands: Commands,
    resize_event: Res<Events<WindowResized>>,
    windows: Res<Windows>,
) {
    let window = windows.primary();
    let mut reader = resize_event.get_reader();
    let mut width = 0.0;
    let mut height = 0.0;

    for e in reader.iter(&resize_event) {
        width = e.width * window.scale_factor() as f32;
        height = e.height * window.scale_factor() as f32;
        println!("resizing window to ({}, {})", width, height);
    }

    commands.insert_resource(ResizeEvent(width, height));
}

fn resize_prepare(
    resize_event: Res<ResizeEvent>,
    render_device: Res<RenderDevice>,
    mut trace_meta: ResMut<TraceMeta>,
) {
    if resize_event.0 == 0.0 || resize_event.1 == 0.0 {
        return;
    }

    let screen_texture = render_device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: resize_event.0 as u32,
            height: resize_event.1 as u32,
            depth_or_array_layers: 2,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::STORAGE_BINDING,
    });
    let screen_texture_view = screen_texture.create_view(&TextureViewDescriptor::default());

    trace_meta.screen_texture_view = screen_texture_view;

    println!("resized window to ({}, {})", resize_event.0, resize_event.1);
}

#[derive(Clone, Copy, ExtractResource)]
struct ResizeEvent(f32, f32);
