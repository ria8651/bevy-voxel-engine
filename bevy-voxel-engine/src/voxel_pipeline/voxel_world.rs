use super::trace::{ExtractedPortal, PalleteEntry};
use crate::{load::GH, LoadVoxelWorld};
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};
use std::sync::Arc;

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let render_queue = app.world.resource::<RenderQueue>();

        // uniforms
        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<VoxelUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let default_path = "/Users/brian/Documents/Code/Rust/vox/monument/monu9.vox".to_string();
        let gh = if let Ok(file) = std::fs::read(default_path) {
            GH::from_vox(&file).unwrap()
        } else {
            GH::empty(32)
        };
        let buffer_size = gh.get_buffer_size();
        let texture_size = gh.texture_size;
        let offsets = gh.get_offsets();

        // texture
        let voxel_world = render_device.create_texture_with_data(
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
        let voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());

        // storage
        let grid_heierachy = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &vec![0; buffer_size],
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
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
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<VoxelUniforms>() as u64
                            ),
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
                ],
            });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&voxel_world),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: grid_heierachy.as_entire_binding(),
                },
            ],
        });

        app.insert_resource(NewGH::None)
            .insert_resource(VoxelUniforms {
                pallete: gh.pallete.into(),
                portals: [ExtractedPortal::default(); 32],
                levels: gh.levels,
                offsets: offsets,
                texture_size,
                _padding: [0; 3],
            })
            .add_plugin(ExtractResourcePlugin::<NewGH>::default())
            .add_plugin(ExtractResourcePlugin::<VoxelUniforms>::default())
            .add_system(load_voxel_world);

        app.sub_app_mut(RenderApp)
            .insert_resource(VoxelData {
                uniform_buffer,
                voxel_world,
                grid_heierachy,
                bind_group_layout,
                bind_group,
            })
            .add_system_to_stage(RenderStage::Prepare, prepare_uniforms)
            .add_system_to_stage(RenderStage::Prepare, load_voxel_world_prepare)
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);
    }
}

#[derive(Resource)]
pub struct VoxelData {
    pub uniform_buffer: Buffer,
    pub voxel_world: TextureView,
    pub grid_heierachy: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

#[repr(C)]
#[derive(Resource, ExtractResource, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VoxelUniforms {
    pub pallete: [PalleteEntry; 256],
    pub portals: [ExtractedPortal; 32],
    pub levels: [u32; 8],
    pub offsets: [u32; 8],
    pub texture_size: u32,
    pub _padding: [u32; 3],
}

#[derive(Resource, ExtractResource, Clone)]
enum NewGH {
    Some(Arc<GH>),
    None,
}

fn prepare_uniforms(
    extraced_uniforms: Res<VoxelUniforms>,
    voxel_data: Res<VoxelData>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &voxel_data.uniform_buffer,
        0,
        bytemuck::cast_slice(&[*extraced_uniforms.as_ref()]),
    );
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

            voxel_uniforms.pallete = gh.pallete.clone().into();
            voxel_uniforms.levels = gh.levels;
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
        voxel_data.grid_heierachy = render_device.create_buffer_with_data(&BufferInitDescriptor {
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
            },
            &gh.texture_data,
        );
        voxel_data.voxel_world = voxel_world.create_view(&TextureViewDescriptor::default());
    }
}

fn queue_bind_group(render_device: Res<RenderDevice>, mut voxel_data: ResMut<VoxelData>) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &voxel_data.bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: voxel_data.uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&voxel_data.voxel_world),
            },
            BindGroupEntry {
                binding: 2,
                resource: voxel_data.grid_heierachy.as_entire_binding(),
            },
        ],
    });
    voxel_data.bind_group = bind_group;
}