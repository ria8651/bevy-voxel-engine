use bevy::prelude::*;

pub struct GH {
    pub levels: [u32; 8],
    pub data: Vec<u8>,
}

impl GH {
    pub fn new(levels: [u32; 8]) -> Self {
        Self {
            levels: levels,
            data: Vec::new(),
        }
    }

    pub fn get_offsets(&self) -> [usize; 8] {
        let mut offsets = [0; 8];
        let mut last = 0;
        for i in 0..8 {
            offsets[i] = last;
            last =
                last + self.levels[i] as usize * self.levels[i] as usize * self.levels[i] as usize;
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

    fn set_bit(&mut self, pos: Vec3) {
        let offsets = self.get_offsets();
        for i in 0..8 {
            if self.levels[i] != 0 {
                let size = self.levels[i] as i32;

                let new_pos = pos * 0.5 + 0.5;
                let new_pos = new_pos * self.levels[i] as f32;
                let int_pos = new_pos.as_ivec3();

                let index = int_pos.x * size * size + int_pos.y * size + int_pos.z;

                let byte = (index as usize + offsets[i]) / 8;
                let bit = index as usize % 8;

                self.data[byte] |= 1 << bit;
            }
        }
    }
}

pub fn load_vox() -> Result<GH, String> {
    let vox = dot_vox::load("assets/vox/monu1.vox")?;
    let size = vox.models[0].size;
    if size.x != size.y || size.x != size.z || size.y != size.z {
        return Err("Voxel model is not a cube!".to_string());
    }

    let size = size.x as usize;

    let mut gh = GH::new([4, 8, 32, 128, 0, 0, 0, 0]);
    println!("{:?}", gh.get_offsets());

    for _ in 0..(gh.get_final_length() / 8) {
        gh.data.push(0);
    }

    for voxel in &vox.models[0].voxels {
        let mut pos = Vec3::new(voxel.x as f32, voxel.z as f32, voxel.y as f32);
        pos /= size as f32;
        pos = pos * 2.0 - 1.0;
        pos = pos * Vec3::new(-1.0, 1.0, 1.0);

        gh.set_bit(pos);
    }

    Ok(gh)
}
