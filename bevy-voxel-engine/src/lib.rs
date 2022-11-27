use bevy::prelude::*;
use physics::PhysicsPlugin;
pub use physics::VOXELS_PER_METER;
use voxel_pipeline::RenderPlugin;
pub use voxel_pipeline::{trace::TraceUniforms, voxelization::VoxelizationMaterial};

mod load;
mod physics;
mod voxel_pipeline;

#[derive(Component)]
pub struct VoxelCamera;

#[derive(Component)]
pub struct Particle {
    pub material: u8,
}

/// normal must be a normalized voxel normal
#[derive(Component)]
pub struct Portal {
    pub half_size: IVec3,
    pub normal: Vec3,
}

#[derive(Component)]
pub struct Edges {
    pub material: u8,
    pub half_size: IVec3,
}

#[derive(Component)]
pub struct Box {
    pub material: u8,
    pub half_size: IVec3,
}

#[derive(Component)]
pub struct Velocity {
    pub velocity: Vec3,
    pub hit_normal: Vec3,
    pub portal_rotation: Mat3,
}

impl Velocity {
    pub fn new(velocity: Vec3) -> Self {
        Self {
            velocity,
            hit_normal: Vec3::ZERO,
            portal_rotation: Mat3::IDENTITY,
        }
    }
}

#[derive(Component)]
pub struct BoxCollider {
    pub half_size: IVec3,
}

#[derive(Bundle, Default)]
pub struct VoxelizationBundle {
    pub mesh_handle: Handle<Mesh>,
    pub voxelization_material: VoxelizationMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

pub struct BevyVoxelEnginePlugin;

impl Plugin for BevyVoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PhysicsPlugin).add_plugin(RenderPlugin);
    }
}

#[derive(Resource)]
pub enum LoadVoxelWorld {
    Empty(u32),
    File(String),
    None,
}
