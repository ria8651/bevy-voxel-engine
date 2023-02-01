use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{camera::CameraRenderGraph, primitives::Frustum, view::VisibleEntities},
};
use physics::PhysicsPlugin;
pub use physics::VOXELS_PER_METER;
use voxel_pipeline::RenderPlugin;
pub use voxel_pipeline::{
    denoise::DenoiseSettings, trace::TraceSettings, voxelization::VoxelizationMaterial,
    voxelization::VoxelizationMaterialType, RenderGraphSettings,
};

mod load;
mod physics;
mod voxel_pipeline;

#[derive(Component)]
pub struct Particle {
    pub material: u8,
    pub flags: u8,
}

/// normal must be a normalized voxel normal
#[derive(Component)]
pub struct Portal;

#[derive(Component)]
pub struct Edges {
    pub material: u8,
    pub flags: u8,
    pub half_size: IVec3,
}

#[derive(Component)]
pub struct Box {
    pub material: u8,
    pub flags: u8,
    pub half_size: IVec3,
}

#[derive(Component)]
pub struct VoxelPhysics {
    pub velocity: Vec3,
    pub gravity: Vec3,
    pub collision_effect: CollisionEffect,
    pub hit_normal: Vec3,
    pub portal_rotation: Mat3,
}

impl VoxelPhysics {
    pub fn new(velocity: Vec3, gravity: Vec3, collision_effect: CollisionEffect) -> Self {
        Self {
            velocity,
            gravity,
            collision_effect,
            hit_normal: Vec3::ZERO,
            portal_rotation: Mat3::IDENTITY,
        }
    }
}

pub enum CollisionEffect {
    None,
    Destroy {
        radius: f32,
    },
    Place {
        radius: f32,
        material: u8,
        flags: u8,
    },
    SetFlags {
        radius: f32,
        flags: u8,
    },
}

impl CollisionEffect {
    pub fn to_vec3(&self) -> Vec3 {
        let mut vec = Vec3::ZERO;
        vec.x = match self {
            CollisionEffect::None => 0u32 as f32,
            CollisionEffect::Destroy { .. } => 1u32 as f32,
            CollisionEffect::Place { .. } => 2u32 as f32,
            CollisionEffect::SetFlags { .. } => 3u32 as f32,
        };
        vec.y = match self {
            CollisionEffect::Destroy { radius }
            | CollisionEffect::Place { radius, .. }
            | CollisionEffect::SetFlags { radius, .. } => *radius,
            _ => 0.0,
        };
        vec.z = match self {
            CollisionEffect::Place {
                material, flags, ..
            } => bytemuck::cast(*material as u32 | ((*flags as u32) << 8)),
            CollisionEffect::SetFlags { flags, .. } => bytemuck::cast(*flags as u32),
            _ => 0.0,
        };

        vec
    }
}

#[derive(Component)]
pub struct BoxCollider {
    pub half_size: IVec3,
}

#[derive(Bundle)]
pub struct VoxelCameraBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: Projection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: Tonemapping,
    pub trace_settings: TraceSettings,
}

impl Default for VoxelCameraBundle {
    fn default() -> Self {
        Self {
            camera_render_graph: CameraRenderGraph::new("voxel"),
            tonemapping: Tonemapping::Enabled {
                deband_dither: true,
            },
            camera: Camera {
                hdr: true,
                ..default()
            },
            projection: default(),
            visible_entities: default(),
            frustum: default(),
            transform: default(),
            global_transform: default(),
            camera_3d: default(),
            trace_settings: default(),
        }
    }
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
        app.insert_resource(Msaa { samples: 1 })
            .add_plugin(PhysicsPlugin)
            .add_plugin(RenderPlugin);
    }
}

#[derive(Resource)]
pub enum LoadVoxelWorld {
    Empty(u32),
    File(String),
    None,
}

#[allow(non_snake_case, dead_code)]
pub mod Flags {
    pub const AUTOMATA_FLAG: u8 = 128; // 0b10000000
    pub const PORTAL_FLAG: u8 = 64; // 0b01000000
    pub const ANIMATION_FLAG: u8 = 32; // 0b00100000
    pub const COLLISION_FLAG: u8 = 16; // 0b00010000
    pub const SAND_FLAG: u8 = 8; // 0b00001000
    pub const NONE: u8 = 0; // 0b00000000
}
