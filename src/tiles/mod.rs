mod storage;

use bevy::{math::UVec2, prelude::Component};
pub use storage::*;

#[derive(Component, Clone, Copy, Debug)]
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

#[derive(Component, Clone, Copy, Debug)]
pub struct TileTexture(pub u32);
