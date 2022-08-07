mod storage;

use bevy::{
    math::UVec2,
    prelude::{Bundle, Color, Component},
};
pub use storage::*;

use crate::map::TilemapId;

/// A tile position in the tilemap grid.
#[derive(Component, Default, Clone, Copy, Debug, Hash, PartialEq, PartialOrd)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

impl TilePos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Into<UVec2> for TilePos {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl Into<UVec2> for &TilePos {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl From<UVec2> for TilePos {
    fn from(v: UVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// A texture index into the atlas or texture array for a single tile. Indices in an atlas are horizontal based.
#[derive(Component, Default, Clone, Copy, Debug, Hash)]
pub struct TileTexture(pub u32);

/// A custom color for the tile.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TileColor(pub Color);

/// Hides or shows a tile based on the boolean. Default: True
#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}

/// Flips the tiles texture along the X, Y or diagonal axises
#[derive(Component, Default, Clone, Copy, Debug, Hash)]
pub struct TileFlip {
    /// Flip tile along the x axis.
    pub x: bool,
    /// Flip tile along the Y axis.
    pub y: bool,
    pub d: bool, // anti
}

/// This an optional tile bundle with default components.
#[derive(Bundle, Default, Clone, Copy, Debug)]
pub struct TileBundle {
    pub position: TilePos,
    pub texture: TileTexture,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub flip: TileFlip,
    pub color: TileColor,
}

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
