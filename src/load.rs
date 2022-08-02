use super::trace::PalleteEntry;
use bevy::prelude::*;

pub struct GH {
    pub levels: [u32; 8],
    pub data: Vec<u8>,
    pub texture_size: u32,
    pub texture_data: Vec<u8>,
    pub pallete: [PalleteEntry; 256],
}

impl GH {
    pub fn new(levels: [u32; 8], texture_size: u32) -> Self {
        Self {
            levels,
            data: Vec::new(),
            texture_size,
            texture_data: Vec::new(),
            pallete: [PalleteEntry::default(); 256],
        }
    }

    pub fn get_offsets(&self) -> [u32; 8] {
        let mut offsets = [0; 8];
        let mut last = 0;
        for i in 0..8 {
            offsets[i] = last;
            last = last + self.levels[i] as u32 * self.levels[i] as u32 * self.levels[i] as u32;
        }
        offsets
    }

    pub fn get_final_length(&self) -> u32 {
        let mut length = 0;
        for i in 0..8 {
            length += self.levels[i] * self.levels[i] * self.levels[i];
        }
        length
    }

    // Assumes data and texture_data are filled to the correct length
    fn set_bit(&mut self, pos: IVec3, value: u8) {
        let offsets = self.get_offsets();
        for i in 0..8 {
            let size = if self.levels[i] != 0 {
                self.levels[i]
            } else {
                self.texture_size
            };

            let new_pos = pos * size as i32 / self.texture_size as i32;
            let index = new_pos.x as u32 * size * size + new_pos.y as u32 * size + new_pos.z as u32;

            if self.levels[i] != 0 {
                let byte = (index as u32 + offsets[i]) / 8;
                let bit = (index as u32 + offsets[i]) % 8;

                self.data[byte as usize] |= 1 << bit;
            } else {
                self.texture_data[index as usize] = value;
                return;
            }
        }
    }
}

pub fn load_vox() -> Result<GH, String> {
    let vox = dot_vox::load("assets/vox/monu9.vox")?;
    let size = vox.models[0].size;
    if size.x != size.y || size.x != size.z || size.y != size.z {
        return Err("Voxel model is not a cube!".to_string());
    }

    let size = size.x as usize;

    let mut gh = GH::new([8, 16, 32, 64, 0, 0, 0, 0], size as u32);
    for i in 0..256 {
        let value = vox.palette[i].to_le_bytes();
        let mut material = Vec4::new(
            value[0] as f32 / 255.0,
            value[1] as f32 / 255.0,
            value[2] as f32 / 255.0,
            0.0,
        );

        let vox_material = vox.materials[i].properties.clone();
        // println!("{:?}", vox_material);
        if vox_material["_type"] == "_emit" {
            material *= 1.0 + vox_material["_emit"].parse::<f32>().unwrap();
            material.w = 1.0;
        }

        gh.pallete[i] = PalleteEntry {
            colour: material.to_array(),
        }
    }

    for _ in 0..(gh.get_final_length() / 8) {
        gh.data.push(0);
    }

    for _ in 0..(gh.texture_size * gh.texture_size * gh.texture_size) {
        gh.texture_data.push(0);
    }

    for voxel in &vox.models[0].voxels {
        let pos = IVec3::new(
            size as i32 - 1 - voxel.x as i32,
            voxel.z as i32,
            voxel.y as i32,
        );

        gh.set_bit(pos, voxel.i);
    }

    Ok(gh)
}
