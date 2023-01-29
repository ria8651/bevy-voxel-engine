use crate::{
    voxel_pipeline::{
        compute::{AnimationData, PhysicsData},
        voxel_world::{ExtractedPortal, VoxelUniforms},
    },
    Box, BoxCollider, Edges, Particle, Portal, RenderGraphSettings, VoxelPhysics,
    VoxelizationMaterial, VoxelizationMaterialType,
};
use bevy::{
    prelude::*,
    render::renderer::{RenderDevice, RenderQueue},
    utils::HashMap,
};

pub const VOXELS_PER_METER: f32 = 4.0;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, insert_physics_data)
            .add_system_to_stage(CoreStage::PostUpdate, extract_physics_data)
            .add_system_to_stage(CoreStage::PostUpdate, extract_animation_data);
    }
}

pub fn extract_physics_data(
    particle_query: Query<(&Transform, &VoxelPhysics, Entity), Without<BoxCollider>>,
    box_query: Query<(&Transform, &VoxelPhysics, &BoxCollider, Entity)>,
    mut physics_data: ResMut<PhysicsData>,
    render_queue: Res<RenderQueue>,
) {
    let mut type_buffer = TypeBuffer::new();
    let mut entities = HashMap::new();

    // add points
    for (transform, voxel_physics, entity) in particle_query.iter() {
        entities.insert(entity, type_buffer.header.len());

        type_buffer.push_object(0, |type_buffer| {
            type_buffer.push_vec3(transform.translation);
            type_buffer.push_vec3(voxel_physics.velocity);
            type_buffer.push_vec3(voxel_physics.gravity);
            type_buffer.push_vec3(voxel_physics.collision_effect.to_vec3());
            type_buffer.push_vec3(Vec3::ZERO); // space to recieve hit data
            type_buffer.push_mat3(Mat3::IDENTITY); // space to recieve portal rotation
        });
    }

    // add boxes
    for (transform, voxel_physics, box_collider, entity) in box_query.iter() {
        entities.insert(entity, type_buffer.header.len());

        type_buffer.push_object(1, |type_buffer| {
            type_buffer.push_vec3(transform.translation);
            type_buffer.push_vec3(voxel_physics.velocity);
            type_buffer.push_vec3(voxel_physics.gravity);
            type_buffer.push_vec3(voxel_physics.collision_effect.to_vec3());
            type_buffer.push_vec3(Vec3::ZERO); // space to recieve hit data
            type_buffer.push_mat3(Mat3::IDENTITY); // space to recieve portal rotation
            type_buffer.push_ivec3(box_collider.half_size);
        });
    }

    physics_data.dispatch_size = type_buffer.header.len() as u32;
    physics_data.buffer_length = (type_buffer.header.len() + type_buffer.data.len() + 1) as u64;

    // copy physics data to the buffer
    render_queue.write_buffer(
        &physics_data.physics_buffer_gpu,
        0,
        bytemuck::cast_slice(&type_buffer.finish()),
    );

    physics_data.entities = entities;
}

pub fn insert_physics_data(
    mut voxel_physics_query: Query<(&mut Transform, &mut VoxelPhysics, Entity)>,
    physics_data: Res<PhysicsData>,
    render_device: Res<RenderDevice>,
    render_graph_settings: Res<RenderGraphSettings>,
) {
    if !render_graph_settings.physics {
        return;
    }

    // process last frames physics data
    if physics_data.dispatch_size > 0 {
        let physics_buffer_slice = physics_data
            .physics_buffer_cpu
            .slice(..physics_data.buffer_length * 4);
        physics_buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        render_device.poll(wgpu::Maintain::Wait);

        let data = physics_buffer_slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        physics_data.physics_buffer_cpu.unmap();

        if result[0] == 0 {
            warn!("No physics data returned from the gpu!");
            return;
        }

        // process points and boxes
        for (mut transform, mut voxel_physics, entity) in voxel_physics_query.iter_mut() {
            if let Some(index) = physics_data.entities.get(&entity) {
                let data_index = result[index + 1] as usize & 0xFFFFFF;
                transform.translation = Vec3::new(
                    bytemuck::cast(result[data_index + 0]),
                    bytemuck::cast(result[data_index + 1]),
                    bytemuck::cast(result[data_index + 2]),
                );
                voxel_physics.velocity = Vec3::new(
                    bytemuck::cast(result[data_index + 3]),
                    bytemuck::cast(result[data_index + 4]),
                    bytemuck::cast(result[data_index + 5]),
                );
                voxel_physics.hit_normal = Vec3::new(
                    bytemuck::cast(result[data_index + 12]),
                    bytemuck::cast(result[data_index + 13]),
                    bytemuck::cast(result[data_index + 14]),
                );
                voxel_physics.portal_rotation = Mat3::from_cols(
                    Vec3::new(
                        bytemuck::cast(result[data_index + 15]),
                        bytemuck::cast(result[data_index + 16]),
                        bytemuck::cast(result[data_index + 17]),
                    ),
                    Vec3::new(
                        bytemuck::cast(result[data_index + 18]),
                        bytemuck::cast(result[data_index + 19]),
                        bytemuck::cast(result[data_index + 20]),
                    ),
                    Vec3::new(
                        bytemuck::cast(result[data_index + 21]),
                        bytemuck::cast(result[data_index + 22]),
                        bytemuck::cast(result[data_index + 23]),
                    ),
                );
            }
        }
    }
}

#[allow(unused)]
pub fn world_to_voxel(world_pos: Vec3, voxel_world_size: u32) -> IVec3 {
    let world_pos = world_pos * VOXELS_PER_METER;
    world_pos.as_ivec3() + IVec3::splat(voxel_world_size as i32 / 2)
}

#[allow(unused)]
pub fn world_to_render(world_pos: Vec3, voxel_world_size: u32) -> Vec3 {
    2.0 * world_pos * VOXELS_PER_METER / voxel_world_size as f32
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

    fn push_mat3(&mut self, value: Mat3) {
        self.data.push(bytemuck::cast(value.x_axis.x));
        self.data.push(bytemuck::cast(value.x_axis.y));
        self.data.push(bytemuck::cast(value.x_axis.z));
        self.data.push(bytemuck::cast(value.y_axis.x));
        self.data.push(bytemuck::cast(value.y_axis.y));
        self.data.push(bytemuck::cast(value.y_axis.z));
        self.data.push(bytemuck::cast(value.z_axis.x));
        self.data.push(bytemuck::cast(value.z_axis.y));
        self.data.push(bytemuck::cast(value.z_axis.z));
    }
}

pub fn extract_animation_data(
    mut animation_data: ResMut<AnimationData>,
    particle_query: Query<(&Transform, &Particle)>,
    mut portal_query: Query<(&Transform, &Portal, &mut VoxelizationMaterial)>,
    edges_query: Query<(&Transform, &Edges)>,
    boxes_query: Query<(&Transform, &Box)>,
    mut voxel_uniforms: ResMut<VoxelUniforms>,
    render_queue: Res<RenderQueue>,
) {
    let mut type_buffer = TypeBuffer::new();

    let voxel_world_size = voxel_uniforms.texture_size;

    // add particles
    for (transform, particle) in particle_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(0, |type_buffer| {
            type_buffer.push_ivec3(pos);
            type_buffer.push_u32(particle.material as u32);
            type_buffer.push_u32(particle.flags as u32);
        });
    }

    // add edges
    for (transform, edges) in edges_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(1, |type_buffer| {
            type_buffer.push_ivec3(pos);
            type_buffer.push_u32(edges.material as u32);
            type_buffer.push_u32(edges.flags as u32);
            type_buffer.push_ivec3(edges.half_size);
        });
    }

    // add boxes
    for (transform, boxes) in boxes_query.iter() {
        let pos = world_to_voxel(transform.translation, voxel_world_size);
        type_buffer.push_object(2, |type_buffer| {
            type_buffer.push_ivec3(pos);
            type_buffer.push_u32(boxes.material as u32);
            type_buffer.push_u32(boxes.flags as u32);
            type_buffer.push_ivec3(boxes.half_size);
        });
    }

    // grab all the poratls in pairs
    voxel_uniforms.portals = [ExtractedPortal::default(); 32];
    let mut portals: Vec<(&Transform, &Portal, Mut<VoxelizationMaterial>)> =
        portal_query.iter_mut().collect();
    for i in 0..portals.len() {
        portals[i].2.material = VoxelizationMaterialType::Material(i as u8);
        if i % 2 == 1 {
            let first = &portals[i - 1];
            let second = &portals[i];

            let first_matrix = first.0.compute_matrix();
            let second_matrix = second.0.compute_matrix();

            let first_normal = -first.0.local_z();
            let first_pos = first.0.translation;

            let second_normal = -second.0.local_z();
            let second_pos = second.0.translation;

            voxel_uniforms.portals[i - 1] = ExtractedPortal {
                transformation: second_matrix * first_matrix.inverse(),
                position: first_pos,
                normal: first_normal,
            };
            voxel_uniforms.portals[i] = ExtractedPortal {
                transformation: first_matrix * second_matrix.inverse(),
                position: second_pos,
                normal: second_normal,
            };
        }
    }

    animation_data.dispatch_size = type_buffer.header.len() as u32;

    // copy animation data to the buffer
    render_queue.write_buffer(
        &animation_data.animation_buffer,
        0,
        bytemuck::cast_slice(&type_buffer.finish()),
    );
}
