use bevy::{
    core::Byteable,
    prelude::*,
    reflect::TypeUuid,
    render::renderer::{RenderResource, RenderResources},
};

use crate::prelude::ChunkSettings;

pub(crate) mod pipeline;

// Used to transfer info to the GPU for tile building.
#[derive(Debug, Default, Clone, TypeUuid, Reflect, RenderResources, RenderResource)]
#[render_resources(from_self)]
#[uuid = "7233c597-ccfa-411f-bd59-9af349432ada"]
#[repr(C)]
pub(crate) struct TilemapData {
    pub(crate) texture_size: Vec2,
    pub(crate) tile_size: Vec2,
    pub(crate) spacing: Vec2,
    pub(crate) time: f32,
}

unsafe impl Byteable for TilemapData {}

impl From<&ChunkSettings> for TilemapData {
    fn from(settings: &ChunkSettings) -> Self {
        Self {
            texture_size: settings.texture_size,
            tile_size: settings.tile_size,
            spacing: settings.spacing,
            time: 0.0,
        }
    }
}
