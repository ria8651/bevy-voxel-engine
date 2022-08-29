use super::{
    compute,
    trace::{ExtractedPortal, Uniforms},
};
use bevy::{prelude::*, render::renderer::RenderDevice};
use std::collections::HashMap;

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
pub struct Velocity {
    pub velocity: Vec3,
}

#[derive(Clone)]
struct TypeBuffer {
    header: Vec<u32>,
    data: Vec<u32>,
}

impl TypeBuffer {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            header: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<u32> {
        // move all the pointers based on the header length
        let offset = self.header.len() + 1;
        for i in 0..self.header.len() {
            self.header[i] += offset as u32;
        }

        // combine the header and animation data
        let mut data = vec![self.header.len() as u32];
        data.extend(self.header);
        data.extend(self.data);

        return data;
    }

    fn push_object<F>(&mut self, object_type: u32, function: F)
    where
        // The closure takes an `i32` and returns an `i32`.
        F: Fn(&mut Self),
    {
        self.header
            .push(self.data.len() as u32 | (object_type << 24));
        function(self);
    }

    fn push_u32(&mut self, value: u32) {
        self.data.push(bytemuck::cast(value));
    }

    fn push_vec3(&mut self, value: Vec3) {
        self.data.push(bytemuck::cast(value.x));
        self.data.push(bytemuck::cast(value.y));
        self.data.push(bytemuck::cast(value.z));
    }

    fn push_ivec3(&mut self, value: IVec3) {
        self.data.push(bytemuck::cast(value.x));
        self.data.push(bytemuck::cast(value.y));
        self.data.push(bytemuck::cast(value.z));
    }
}

const VOXELS_PER_METER: u32 = 4;

pub fn world_to_voxel(world_pos: Vec3, voxel_world_size: u32) -> IVec3 {
    let world_pos = world_pos * VOXELS_PER_METER as f32;
    world_pos.as_ivec3() + IVec3::splat(voxel_world_size as i32 / 2)
}

pub fn world_to_render(world_pos: Vec3, voxel_world_size: u32) -> Vec3 {
    2.0 * world_pos * VOXELS_PER_METER as f32 / voxel_world_size as f32
}

pub fn extract_animation_data(
    mut commands: Commands,
    particle_query: Query<(&Transform, &Particle)>,
    portal_query: Query<(&Transform, &Portal)>,
    edges_query: Query<(&Transform, &Edges)>,
    mut uniforms: ResMut<Uniforms>,
) {
    let mut type_buffer = TypeBuffer::new();

    let voxel_world_size = uniforms.texture_size;

    // add particles
    for (transform, particle) in particle_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(0, |type_buffer| {
            type_buffer.push_u32(particle.material as u32);
            type_buffer.push_ivec3(pos);
        });
    }

    // add portals
    let mut i = 0;
    for (transform, portal) in portal_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(1, |type_buffer| {
            type_buffer.push_u32(1); // portal material doesn't matter atm
            type_buffer.push_ivec3(pos);
            type_buffer.push_u32(i);
            type_buffer.push_ivec3(portal.half_size);
        });
        i += 1;
    }

    // add edges
    for (transform, edges) in edges_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(2, |type_buffer| {
            type_buffer.push_u32(edges.material as u32);
            type_buffer.push_ivec3(pos);
            type_buffer.push_ivec3(edges.half_size);
        });
    }

    // grab all the poratls in pairs
    uniforms.portals = [ExtractedPortal::default(); 32];
    let mut i = 0;
    let mut first: Option<(&Transform, &Portal)> = None;
    for (transform, portal) in portal_query.iter() {
        if i % 2 == 1 {
            let first = first.unwrap();
            let second = (transform, portal);

            let first_normal = first.1.normal;
            let second_normal = second.1.normal;

            let voxel_size = 2.0 / uniforms.texture_size as f32;
            let first_pos =
                world_to_render(first.0.translation, uniforms.texture_size) + voxel_size / 2.0;
            let second_pos =
                world_to_render(second.0.translation, uniforms.texture_size) + voxel_size / 2.0;

            uniforms.portals[i - 1] = ExtractedPortal {
                pos: [first_pos.x, first_pos.y, first_pos.z, 0.0],
                other_pos: [second_pos.x, second_pos.y, second_pos.z, 0.0],
                normal: [first_normal.x, first_normal.y, first_normal.z, 0.0],
                other_normal: [second_normal.x, second_normal.y, second_normal.z, 0.0],
                half_size: [
                    first.1.half_size.x,
                    first.1.half_size.y,
                    first.1.half_size.z,
                    0,
                ],
            };
            uniforms.portals[i] = ExtractedPortal {
                pos: [second_pos.x, second_pos.y, second_pos.z, 0.0],
                other_pos: [first_pos.x, first_pos.y, first_pos.z, 0.0],
                normal: [second_normal.x, second_normal.y, second_normal.z, 0.0],
                other_normal: [first_normal.x, first_normal.y, first_normal.z, 0.0],
                half_size: [
                    second.1.half_size.x,
                    second.1.half_size.y,
                    second.1.half_size.z,
                    0,
                ],
            };
        }
        first = Some((transform, portal));
        i += 1;
    }

    commands.insert_resource(compute::ExtractedAnimationData {
        data: type_buffer.finish(),
    });
}

pub fn extract_physics_data(
    bullet_query: Query<(&Transform, &Velocity, Entity)>,
    mut extracted_physics_data: ResMut<compute::ExtractedPhysicsData>,
) {
    let mut type_buffer = TypeBuffer::new();
    let mut entities = HashMap::new();

    // add velocity components
    for (transform, velocity, entity) in bullet_query.iter() {
        entities.insert(entity, type_buffer.header.len());

        type_buffer.push_object(0, |type_buffer| {
            type_buffer.push_vec3(transform.translation);
            type_buffer.push_vec3(velocity.velocity);
        });
    }

    extracted_physics_data.data = type_buffer.finish();
    extracted_physics_data.entities = entities;
}

pub fn insert_physics_data(
    mut bullet_query: Query<(&mut Transform, &mut Velocity, Entity)>,
    extracted_physics_data: Res<compute::ExtractedPhysicsData>,
    compute_meta: Res<compute::ComputeMeta>,
    render_device: Res<RenderDevice>,
) {
    // process last frames physics data
    if extracted_physics_data.data.len() > 1 {
        let buffer_slice = compute_meta
            .physics_data
            .slice(..extracted_physics_data.data.len() as u64 * 4);

        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

        render_device.poll(wgpu::Maintain::Wait);

        let data = buffer_slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        compute_meta.physics_data.unmap();

        for (mut transform, mut velocity, entity) in bullet_query.iter_mut() {
            if let Some(index) = extracted_physics_data.entities.get(&entity) {
                let data_index = result[index + 1] as usize;
                transform.translation = Vec3::new(
                    bytemuck::cast(result[data_index + 0]),
                    bytemuck::cast(result[data_index + 1]),
                    bytemuck::cast(result[data_index + 2]),
                );
                velocity.velocity = Vec3::new(
                    bytemuck::cast(result[data_index + 3]),
                    bytemuck::cast(result[data_index + 4]),
                    bytemuck::cast(result[data_index + 5]),
                );
            }
        }
    }
}
