use super::trace::PalleteEntry;
use bevy::prelude::*;

pub struct GH {
    pub levels: [u32; 8],
    pub texture_size: u32,
    pub texture_data: Vec<u8>,
    pub pallete: [PalleteEntry; 256],
}

impl GH {
    pub fn new(levels: [u32; 8], texture_size: u32) -> Self {
        Self {
            levels,
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
}

pub fn load_vox() -> Result<GH, String> {
    let vox = dot_vox::load("assets/vox/map1.vox")?;
    let size = vox.models[0].size;
    if size.x != size.y || size.x != size.z || size.y != size.z {
        return Err("Voxel model is not a cube!".to_string());
    }

    let size = size.x as usize;

    let mut gh = GH::new([8, 16, 32, 64, 128, 0, 0, 0], size as u32);
    for i in 0..256 {
        let value = vox.palette[i].to_le_bytes();
        let mut material = Vec4::new(
            value[0] as f32 / 255.0,
            value[1] as f32 / 255.0,
            value[2] as f32 / 255.0,
            0.0,
        );

        let vox_material = vox.materials[i].properties.clone();
        if vox_material["_type"] == "_emit" {
            material *= 1.0 + vox_material["_emit"].parse::<f32>().unwrap();
            if vox_material.contains_key("_flux") {
                material = material.powf(vox_material["_flux"].parse::<f32>().unwrap());
            }
            material.w = 1.0;
        }

        gh.pallete[i] = PalleteEntry {
            colour: material.to_array(),
        }
    }

    let texture_size = gh.texture_size * gh.texture_size * gh.texture_size * 2;
    gh.texture_data = vec![0; texture_size as usize];

    for voxel in &vox.models[0].voxels {
        let pos = IVec3::new(
            size as i32 - 1 - voxel.x as i32,
            voxel.z as i32,
            voxel.y as i32,
        );

        let index = pos.x as usize * size * size + pos.y as usize * size + pos.z as usize;
        gh.texture_data[index as usize * 2] = voxel.i;
    }

    Ok(gh)
}
