use bevy::prelude::*;

pub fn load_vox() -> Result<Vec<u8>, String> {
    let vox = dot_vox::load("assets/vox/monu1.vox")?;
    let size = vox.models[0].size;
    if size.x != size.y || size.x != size.z || size.y != size.z {
        return Err("Voxel model is not a cube!".to_string());
    }

    let size = size.x as usize;
    let length = size * size * size;

    let mut data = Vec::new();
    for _ in 0..(length / 8) {
        data.push(0);
    }

    for voxel in &vox.models[0].voxels {
        let pos = (size - voxel.x as usize - 1, voxel.z as usize, voxel.y as usize);
        let index = pos.0 * size * size + pos.1 * size + pos.2;

        let byte = index / 8;
        let bit = index % 8;

        data[byte] |= 1 << bit;
    }

    Ok(data)
}