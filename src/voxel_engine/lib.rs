pub use animation::VOXELS_PER_METER;
use bevy::prelude::*;

mod animation;
mod compute;
mod fps_counter;
mod load;
pub mod trace;

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

pub struct VoxelWorld;

impl Plugin for VoxelWorld {
    fn build(&self, app: &mut App) {
        app.insert_resource(trace::ShaderTimer(Timer::from_seconds(1000.0, true)))
            .insert_resource(trace::LastFrameData {
                last_camera: Mat4::default(),
            })
            .add_plugin(trace::Tracer)
            .add_plugin(compute::ComputePlugin)
            .add_plugin(fps_counter::FpsCounter);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum LoadVoxelWorld {
    Empty(u32),
    File(String),
    None,
}
