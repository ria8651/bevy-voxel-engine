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
    fn set_bit(&mut self, pos: Vec3, value: u8) {
        let offsets = self.get_offsets();
        for i in 0..8 {
            let size = if self.levels[i] != 0 {
                self.levels[i] as i32
            } else {
                self.texture_size as i32
            };

            let new_pos = pos * 0.5 + 0.5;
            let new_pos = new_pos * size as f32 - 0.5;
            let int_pos = new_pos.as_ivec3();

            let index = int_pos.x * size * size + int_pos.y * size + int_pos.z;

            if self.levels[i] != 0 {
                let byte = (index as u32 + offsets[i]) / 8;
                let bit = index as usize % 8;

                self.data[byte as usize] |= 1 << bit;
            } else {
                self.texture_data[index as usize] = value;
                return;
            }
        }
    }
}

pub fn load_vox() -> Result<GH, String> {
    let vox = dot_vox::load("assets/vox/phantom_mansion.vox")?;
    let size = vox.models[0].size;
    if size.x != size.y || size.x != size.z || size.y != size.z {
        return Err("Voxel model is not a cube!".to_string());
    }

    let size = size.x as usize;

    let mut gh = GH::new([4, 8, 16, 32, 64, 128, 0, 0], 256);
    for i in 0..256 {
        gh.pallete[i] = PalleteEntry {
            colour: vox.palette[i],
        }
    }

    for _ in 0..(gh.get_final_length() / 8) {
        gh.data.push(0);
    }

    for _ in 0..(gh.texture_size * gh.texture_size * gh.texture_size) {
        gh.texture_data.push(0);
    }

    for voxel in &vox.models[0].voxels {
        let mut pos = Vec3::new(voxel.x as f32, voxel.z as f32, voxel.y as f32);
        pos /= size as f32;
        pos = pos * 2.0 - 1.0;
        pos = pos * Vec3::new(-1.0, 1.0, 1.0);

        gh.set_bit(pos, voxel.i);
    }

    Ok(gh)
}
