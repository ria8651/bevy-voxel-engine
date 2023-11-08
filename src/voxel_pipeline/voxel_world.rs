use crate::{
    load::{Pallete, GH},
    LoadVoxelWorld,
};
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use std::sync::Arc;

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let render_device = app.sub_app(RenderApp).world.resource::<RenderDevice>();

        let render_queue = app.sub_app(RenderApp).world.resource::<RenderQueue>();

        let gh = GH::empty(128);
        let buffer_size = gh.get_buffer_size();
        let texture_size = gh.texture_size;
        let gh_offsets = gh.get_offsets();

        let mut levels = [UVec4::ZERO; 8];
        let mut offsets = [UVec4::ZERO; 8];
        for i in 0..8 {
            levels[i] = UVec4::new(gh.levels[i], 0, 0, 0);
            offsets[i] = UVec4::new(gh_offsets[i], 0, 0, 0);
        }

        // Uniforms
        let voxel_uniforms = VoxelUniforms {
            pallete: gh.pallete.into(),
            portals: [ExtractedPortal::default(); 32],
            levels,
            offsets,
            texture_size,
        };
        let mut uniform_buffer = UniformBuffer::from(voxel_uniforms.clone());
        uniform_buffer.write_buffer(&render_device, &render_queue);

        // Texture
        let voxel_world = render_device.create_texture_with_data(
            &render_queue,
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
                view_formats: &[],
            },
            &gh.texture_data.clone(),
        );
        let voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());

        // Storage
        let grid_hierarchy = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; buffer_size],
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Mip texture
        let mip_count = gh.texture_size.trailing_zeros();
        let mip_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: gh.texture_size,
                height: gh.texture_size,
                depth_or_array_layers: gh.texture_size,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mip_texture_view = mip_texture.create_view(&TextureViewDescriptor::default());

        // Sampler
        let texture_sampler = render_device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..default()
        });

        let bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxelization bind group layout"),
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
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R16Uint,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(4),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let bind_group = render_device.create_bind_group(
            None,
            &bind_group_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&voxel_world),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: grid_hierarchy.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&mip_texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
        );

        app.insert_resource(LoadVoxelWorld::None)
            .insert_resource(NewGH::None)
            .insert_resource(voxel_uniforms)
            .add_plugins(ExtractResourcePlugin::<NewGH>::default())
            .add_plugins(ExtractResourcePlugin::<VoxelUniforms>::default())
            .add_systems(Update, load_voxel_world);

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(VoxelData {
                uniform_buffer,
                voxel_world,
                grid_hierarchy,
                mip_texture,
                texture_sampler,
                bind_group_layout,
                bind_group,
            })
            .add_systems(Render, prepare_uniforms.in_set(RenderSet::Prepare))
            .add_systems(Render, load_voxel_world_prepare.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
    }
}

#[derive(Resource)]
pub struct VoxelData {
    pub uniform_buffer: UniformBuffer<VoxelUniforms>,
    pub voxel_world: TextureView,
    pub grid_hierarchy: Buffer,
    pub mip_texture: Texture,
    pub texture_sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

#[derive(Default, Debug, Clone, Copy, ShaderType)]
pub struct PalleteEntry {
    pub colour: Vec4,
}

impl Into<[PalleteEntry; 256]> for Pallete {
    fn into(self) -> [PalleteEntry; 256] {
        let mut pallete = [PalleteEntry::default(); 256];
        for i in 0..256 {
            pallete[i].colour = self[i].into();
        }
        pallete
    }
}

#[derive(Default, Debug, Clone, Copy, ShaderType)]
pub struct ExtractedPortal {
    pub transformation: Mat4,
    pub position: Vec3,
    pub normal: Vec3,
}

#[derive(Resource, ExtractResource, Clone, ShaderType)]
pub struct VoxelUniforms {
    pub pallete: [PalleteEntry; 256],
    pub portals: [ExtractedPortal; 32],
    pub levels: [UVec4; 8],
    pub offsets: [UVec4; 8],
    pub texture_size: u32,
}

#[derive(Resource, ExtractResource, Clone)]
enum NewGH {
    Some(Arc<GH>),
    None,
}

fn prepare_uniforms(
    voxel_uniforms: Res<VoxelUniforms>,
    mut voxel_data: ResMut<VoxelData>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    voxel_data.uniform_buffer.set(voxel_uniforms.clone());
    voxel_data
        .uniform_buffer
        .write_buffer(&render_device, &render_queue);
}

fn load_voxel_world(
    mut load_voxel_world: ResMut<LoadVoxelWorld>,
    mut new_gh: ResMut<NewGH>,
    mut voxel_uniforms: ResMut<VoxelUniforms>,
) {
    match load_voxel_world.as_ref() {
        LoadVoxelWorld::Empty(_) | LoadVoxelWorld::File(_) => {
            let gh = match load_voxel_world.as_ref() {
                LoadVoxelWorld::Empty(size) => GH::empty(*size),
                LoadVoxelWorld::File(path) => {
                    let file = std::fs::read(path).unwrap();
                    GH::from_vox(&file).unwrap()
                }
                LoadVoxelWorld::None => unreachable!(),
            };

            let mut levels = [UVec4::ZERO; 8];
            for i in 0..8 {
                levels[i] = UVec4::new(gh.levels[i], 0, 0, 0);
            }

            voxel_uniforms.pallete = gh.pallete.clone().into();
            voxel_uniforms.levels = levels;
            voxel_uniforms.texture_size = gh.texture_size;

            *new_gh = NewGH::Some(Arc::new(gh));
            *load_voxel_world = LoadVoxelWorld::None;
        }
        LoadVoxelWorld::None => {
            *new_gh = NewGH::None;
        }
    }
}

fn load_voxel_world_prepare(
    mut voxel_data: ResMut<VoxelData>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    new_gh: Res<NewGH>,
) {
    if let NewGH::Some(gh) = new_gh.as_ref() {
        let buffer_size = gh.get_buffer_size();

        // grid hierarchy
        voxel_data.grid_hierarchy = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; buffer_size],
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // voxel world
        let voxel_world = render_device.create_texture_with_data(
            render_queue.as_ref(),
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
                view_formats: &[],
            },
            &gh.texture_data,
        );
        voxel_data.voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());

        // mip texture
        let mip_count = gh.texture_size.trailing_zeros();
        let mip_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: gh.texture_size,
                height: gh.texture_size,
                depth_or_array_layers: gh.texture_size,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        voxel_data.mip_texture = mip_texture;
    }
}

fn queue_bind_group(render_device: Res<RenderDevice>, mut voxel_data: ResMut<VoxelData>) {
    let bind_group = render_device.create_bind_group(
        None,
        &voxel_data.bind_group_layout,
        &[
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
                resource: voxel_data.grid_hierarchy.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(
                    &voxel_data
                        .mip_texture
                        .create_view(&TextureViewDescriptor::default()),
                ),
            },
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::Sampler(&voxel_data.texture_sampler),
            },
        ],
    );
    voxel_data.bind_group = bind_group;
}
