mod storage;

use bevy::{
    math::UVec2,
    prelude::{Bundle, Color, Component},
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

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TileColor(pub Color);

#[derive(Component, Clone, Copy, Debug)]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TileFlip {
    /// Flip tile along the x axis.
    pub x: bool,
    /// Flip tile along the Y axis.
    pub y: bool,
    pub d: bool, // anti
}

#[derive(Bundle, Default, Clone, Copy, Debug)]
pub struct TileBundle {
    pub position: TilePos2d,
    pub texture: TileTexture,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub flip: TileFlip,
    pub color: TileColor,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct RemoveTile;

/// A component that is attached to a Tile entity that
/// tells the GPU how to animate the tile.
/// Currently all frames must be aligned in your tilemap.
#[derive(Component, Clone, Copy, Debug)]
pub struct AnimatedTile {
    /// The start frame index in the tilemap atlas/array (inclusive).
    pub start: u32,
    /// The end frame index in the tilemap atlas/array (exclusive).
    pub end: u32,
    /// The speed the animation plays back at.
    pub speed: f32,
}
