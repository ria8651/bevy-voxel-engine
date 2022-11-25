use crate::VOXELS_PER_METER;

use super::voxel_world::{VoxelData, VoxelUniforms};
use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Transparent3d},
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<VoxelizationMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_cameras)
            .add_system(update);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<VoxelizationPipeline>()
            .init_resource::<SpecializedMeshPipelines<VoxelizationPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct VoxelizationImage(Handle<Image>);

#[derive(Component)]
struct VoxelizationCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // image that is the size of the render world to create the correct ammount of fragments
    let size = Extent3d {
        width: 1,
        height: 1,
        ..default()
    };
    let image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    let image_handle = images.add(image);
    commands.insert_resource(VoxelizationImage(image_handle.clone()));

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                priority: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        VoxelizationCamera,
    ));

    commands.spawn((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::splat(1.0), Vec3::Y)
            .with_scale(Vec3::splat(10.0)),
        GlobalTransform::default(),
        VoxelizationMaterial,
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}

fn update_cameras(
    voxelization_image: Res<VoxelizationImage>,
    mut images: ResMut<Assets<Image>>,
    mut voxelization_cameras: Query<(&mut Transform, &mut Projection), With<VoxelizationCamera>>,
    voxel_uniforms: Res<VoxelUniforms>,
) {
    let voxelization_image = images.get_mut(&voxelization_image).unwrap();
    if voxelization_image.size().x as u32 != voxel_uniforms.texture_size {
        // update cameras
        info!(
            "Updating voxelization cameras to {}",
            voxel_uniforms.texture_size
        );
        for (mut transform, mut projection) in voxelization_cameras.iter_mut() {
            // resize image
            let size = voxel_uniforms.texture_size;
            voxelization_image.resize(Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            });

            // update camera
            *transform = Transform::from_translation(Vec3::ZERO).looking_at(Vec3::Y, Vec3::Z);

            let side = size as f32 / VOXELS_PER_METER / 2.0;
            *projection = Projection::Orthographic(OrthographicProjection {
                near: -side,
                far: side,
                left: side,
                right: -side,
                top: side,
                bottom: -side,
                scaling_mode: ScalingMode::None,
                ..default()
            });

            // Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Z);
        }
    }
}

fn update(time: Res<Time>, mut cube: Query<&mut Transform, With<VoxelizationMaterial>>) {
    for mut transform in cube.iter_mut() {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}

#[derive(Component)]
struct VoxelizationMaterial;

impl ExtractComponent for VoxelizationMaterial {
    type Query = Read<VoxelizationMaterial>;

    type Filter = ();

    fn extract_component(_: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        VoxelizationMaterial
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetVoxelizationBindGroup<2>,
    DrawMesh,
);

#[derive(Resource)]
pub struct VoxelizationPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for VoxelizationPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("voxelization.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        let voxel_world_data = world.resource::<VoxelData>();
        let bind_group_layout = voxel_world_data.bind_group_layout.clone();

        VoxelizationPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for VoxelizationPipeline {
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
            self.bind_group_layout.clone(),
        ]);
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}

fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<VoxelizationPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<VoxelizationPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<VoxelizationMaterial>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let key = MeshPipelineKey::from_msaa_samples(msaa.samples)
        | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);

    for (view, mut transparent_phase) in &mut views {
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

struct SetVoxelizationBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetVoxelizationBindGroup<I> {
    type Param = SRes<VoxelData>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let voxel_world_data = query.into_inner();

        pass.set_bind_group(I, &voxel_world_data.bind_group, &[]);

        RenderCommandResult::Success
    }
}
