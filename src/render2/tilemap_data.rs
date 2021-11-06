use bevy::math::Vec2;
use crevice::std140::AsStd140;

use crate::Chunk;

// Used to transfer info to the GPU for tile building.
#[derive(Debug, Default, Copy, Clone, AsStd140)]
pub struct TilemapUniformData {
    pub texture_size: Vec2,
    pub tile_size: Vec2,
    pub grid_size: Vec2,
    pub spacing: Vec2,
    pub chunk_pos: Vec2,
    pub map_size: Vec2,
    pub time: f32,
}

impl From<&Chunk> for TilemapUniformData {
    fn from(chunk: &Chunk) -> Self {
        let chunk_pos: Vec2 = chunk.position.into();
        let chunk_size: Vec2 = chunk.settings.chunk_size.into();
        let map_size: Vec2 = chunk.settings.map_size.into();
        Self {
            texture_size: chunk.settings.texture_size.into(),
            tile_size: chunk.settings.tile_size.into(),
            grid_size: chunk.settings.grid_size,
            spacing: chunk.settings.tile_spacing,
            chunk_pos: chunk_pos * chunk_size,
            map_size: map_size * chunk_size * chunk.settings.grid_size,
            time: 0.0,
        }
    }
}
