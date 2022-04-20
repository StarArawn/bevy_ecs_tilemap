mod storage;

use bevy::{
    math::UVec2,
    prelude::{Bundle, Component},
};
pub use storage::*;

use crate::map::TilemapId;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TilePos2d {
    pub x: u32,
    pub y: u32,
}

impl TilePos2d {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Into<UVec2> for TilePos2d {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl Into<UVec2> for &TilePos2d {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl From<UVec2> for TilePos2d {
    fn from(v: UVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TileTexture(pub u32);

#[derive(Component, Clone, Copy, Debug)]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Bundle, Default, Clone, Copy, Debug)]
pub struct TileBundle {
    pub position: TilePos2d,
    pub texture: TileTexture,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
}
