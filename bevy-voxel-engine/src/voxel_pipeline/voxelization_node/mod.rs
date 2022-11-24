use super::trace::ExtractedUniforms;
use bevy::{
    core_pipeline::{
        clear_color::ClearColorConfig, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    prelude::*,
    render::{
        camera::CameraRenderGraph, render_resource::*, renderer::RenderDevice, view::ViewTarget,
        RenderApp, RenderStage,
    },
};

pub mod node;

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);

        app.sub_app_mut(RenderApp)
            .init_resource::<VoxelizationPipeline>()
            .init_resource::<SpecializedRenderPipelines<VoxelizationPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_main_pipeline);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform1 =
        Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Z);

    commands.spawn(Camera3dBundle {
        transform: transform1,
        camera_render_graph: CameraRenderGraph::new("voxelization"),
        camera: Camera {
            hdr: true,
            priority: 1,
            ..default()
        },
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0).looking_at(Vec3::splat(1.0), Vec3::Y),
        ..default()
    });
}

#[derive(Resource)]
struct VoxelizationPipeline {
    shader: Handle<Shader>,
    bind_group: BindGroupLayout,
}

#[derive(Component)]
struct ViewVoxelizationPipeline(CachedRenderPipelineId);

impl SpecializedRenderPipeline for VoxelizationPipeline {
    type Key = ();

    fn specialize(&self, _: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("voxelization pipeline".into()),
            layout: Some(vec![self.bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: Vec::new(),
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
        }
    }
}

impl FromWorld for VoxelizationPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxelization bind group layout"),
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
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R16Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                ],
            });

        let asset_server = render_world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("voxelization.wgsl");

        VoxelizationPipeline {
            shader,
            bind_group: bind_group,
        }
    }
}

fn queue_main_pipeline(
    mut commands: Commands,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<VoxelizationPipeline>>,
    voxelization_pipeline: Res<VoxelizationPipeline>,
    view_targets: Query<Entity, With<ViewTarget>>,
) {
    for entity in view_targets.iter() {
        let pipeline = pipelines.specialize(&mut pipeline_cache, &voxelization_pipeline, ());
        commands
            .entity(entity)
            .insert(ViewVoxelizationPipeline(pipeline));
    }
}
