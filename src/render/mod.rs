use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::renderer::{RenderResource, RenderResources},
};
use bytemuck::{Pod, Zeroable};

use crate::Chunk;

pub(crate) mod pipeline;

// Used to transfer info to the GPU for tile building.
#[derive(Debug, Default, Copy, Clone, TypeUuid, Reflect, RenderResources, RenderResource, Pod, Zeroable)]
#[render_resources(from_self)]
#[uuid = "7233c597-ccfa-411f-bd59-9af349432ada"]
#[repr(C)]
pub(crate) struct TilemapData {
    pub(crate) texture_size: Vec2,
    pub(crate) tile_size: Vec2,
    pub(crate) grid_size: Vec2,
    pub(crate) spacing: Vec2,
    pub(crate) chunk_pos: Vec2,
    pub(crate) map_size: Vec2,
    pub(crate) time: f32,
}

impl From<&Chunk> for TilemapData {
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
