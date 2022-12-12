use super::voxel_world::{VoxelData, VoxelUniforms};
use crate::{RenderGraphSettings, VOXELS_PER_METER};
use bevy::{
    asset::load_internal_asset,
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Transparent3d},
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{RenderTarget, ScalingMode},
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};

const VOXELIZATION_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1975691635883203525);

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            VOXELIZATION_SHADER_HANDLE,
            "shaders/voxelization.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractResourcePlugin::<FallbackImage>::default())
            .add_plugin(ExtractComponentPlugin::<VoxelizationMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_cameras);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<VoxelizationPipeline>()
            .init_resource::<SpecializedMeshPipelines<VoxelizationPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_custom);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct VoxelizationImage(Handle<Image>);

#[derive(Component)]
struct VoxelizationCamera;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // create fallback image
    let mut image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[10, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);
    commands.insert_resource(FallbackImage(image));

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

    // priorities of -3, -2 and -1 so that they are rendered before the main pass
    for i in 0..3 {
        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    priority: -3 + i,
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
    }
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
        debug!(
            "Updating {} voxelization cameras to as resolution of {}",
            voxelization_cameras.iter().len(),
            voxel_uniforms.texture_size
        );
        let mut i = 0;
        for (mut transform, mut projection) in voxelization_cameras.iter_mut() {
            // resize image
            let size = voxel_uniforms.texture_size;
            voxelization_image.resize(Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            });

            // update camera
            *transform = match i {
                0 => Transform::from_translation(Vec3::ZERO).looking_at(Vec3::X, Vec3::Y),
                1 => Transform::from_translation(Vec3::ZERO).looking_at(Vec3::Y, Vec3::Z),
                2 => Transform::from_translation(Vec3::ZERO).looking_at(Vec3::Z, Vec3::Y),
                _ => panic!("Too many voxelization cameras"),
            };

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

            i += 1;
        }
    }
}

#[derive(Component, Default, Clone, Debug)]
pub struct VoxelizationMaterial {
    pub texture: Handle<Image>,
}

impl ExtractComponent for VoxelizationMaterial {
    type Query = Read<VoxelizationMaterial>;

    type Filter = ();

    fn extract_component(material: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        material.clone()
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetVoxelWorldBindGroup<2>,
    SetVoxelizationBindGroup<3>,
    DrawMesh,
);

#[derive(Resource)]
pub struct VoxelizationPipeline {
    mesh_pipeline: MeshPipeline,
    world_bind_group_layout: BindGroupLayout,
    voxelization_bind_group_layout: BindGroupLayout,
}

#[derive(Resource, Clone, ExtractResource, Deref, DerefMut)]
struct FallbackImage(Handle<Image>);

impl FromWorld for VoxelizationPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let voxel_world_data = world.resource::<VoxelData>();

        let world_bind_group_layout = voxel_world_data.bind_group_layout.clone();
        let voxelization_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        VoxelizationPipeline {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            world_bind_group_layout,
            voxelization_bind_group_layout,
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
        descriptor.vertex.shader = VOXELIZATION_SHADER_HANDLE.typed();
        descriptor.fragment.as_mut().unwrap().shader = VOXELIZATION_SHADER_HANDLE.typed();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.world_bind_group_layout.clone(),
            self.voxelization_bind_group_layout.clone(),
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
    render_graph_settings: Res<RenderGraphSettings>,
) {
    if !render_graph_settings.voxelization {
        return;
    }

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

#[derive(Component, Deref, DerefMut)]
struct VoxelizationBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    voxelization_materials: Query<(Entity, &VoxelizationMaterial)>,
    gpu_images: Res<RenderAssets<Image>>,
    voxelization_pipeline: Res<VoxelizationPipeline>,
    fallback_image: Res<FallbackImage>,
) {
    for (entity, voxelization_material) in voxelization_materials.iter() {
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let image_handle = voxelization_material.texture.clone();
        let image_view = gpu_images
            .get(&image_handle)
            .unwrap_or(gpu_images.get(&fallback_image).unwrap());

        let voxelization_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &voxelization_pipeline.voxelization_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&image_view.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        commands
            .entity(entity)
            .insert(VoxelizationBindGroup(voxelization_bind_group));
    }
}

struct SetVoxelWorldBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetVoxelWorldBindGroup<I> {
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

struct SetVoxelizationBindGroup<const I: usize>;

impl<const I: usize> EntityRenderCommand for SetVoxelizationBindGroup<I> {
    type Param = SQuery<Read<VoxelizationBindGroup>>;

    fn render<'w>(
        _view: Entity,
        item: Entity,
        query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let voxelization_bind_group = query.get_inner(item).unwrap();

        pass.set_bind_group(I, voxelization_bind_group, &[]);

        RenderCommandResult::Success
    }
}
