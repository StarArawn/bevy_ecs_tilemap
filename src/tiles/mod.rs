mod storage;

use bevy::{
    math::{UVec2, Vec2},
    prelude::{Bundle, Color, Component, Reflect, ReflectComponent},
    render::sync_world::SyncToRenderWorld,
};
pub use storage::*;

use crate::TilemapSize;
use crate::map::TilemapId;

/// A tile position in the tilemap grid.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

impl TilePos {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Converts a tile position (2D) into an index in a flattened vector (1D), assuming the
    /// tile position lies in a tilemap of the specified size.
    ///
    /// An out-of-bounds x will produce a different tile position. Use
    /// [`TilePos::to_checked_index`] to make sure the returned index is correct.
    pub fn to_index(&self, tilemap_size: &TilemapSize) -> usize {
        (self.y as usize * tilemap_size.x as usize) + self.x as usize
    }

    /// Converts a tile position (2D) into an index in a flattened vector (1D), or returns `None`
    /// if the tile position doesn't lie within a tilemap of the specified size.
    pub fn to_checked_index(&self, tilemap_size: &TilemapSize) -> Option<usize> {
        self.within_map_bounds(tilemap_size)
            .then(|| self.to_index(tilemap_size))
    }

    /// Checks to see if `self` lies within a tilemap of the specified size.
    pub fn within_map_bounds(&self, map_size: &TilemapSize) -> bool {
        self.x < map_size.x && self.y < map_size.y
    }
}

impl From<TilePos> for UVec2 {
    fn from(pos: TilePos) -> Self {
        UVec2::new(pos.x, pos.y)
    }
}

impl From<&TilePos> for UVec2 {
    fn from(pos: &TilePos) -> Self {
        UVec2::new(pos.x, pos.y)
    }
}

impl From<UVec2> for TilePos {
    fn from(v: UVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<TilePos> for Vec2 {
    fn from(pos: TilePos) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}

impl From<&TilePos> for Vec2 {
    fn from(pos: &TilePos) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}

/// A texture index into the atlas or texture array for a single tile. Indices in an atlas are horizontal based.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component)]
pub struct TileTextureIndex(pub u32);

/// A custom color for the tile.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TileColor(pub Color);

impl From<Color> for TileColor {
    fn from(color: Color) -> Self {
        TileColor(color)
    }
}

/// Hides or shows a tile based on the boolean. Default: True
#[derive(Component, Reflect, Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}

/// Flips the tiles texture along the X, Y or diagonal axes
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TileFlip {
    /// Flip tile along the x axis.
    pub x: bool,
    /// Flip tile along the Y axis.
    pub y: bool,
    pub d: bool, // anti
}

/// This an optional tile bundle with default components.
#[derive(Bundle, Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TileBundle {
    pub position: TilePos,
    pub texture_index: TileTextureIndex,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub flip: TileFlip,
    pub color: TileColor,
    pub old_position: TilePosOld,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub sync: SyncToRenderWorld,
}

#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TilePosOld(pub TilePos);

/// A component that is attached to a Tile entity that
/// tells the GPU how to animate the tile.
/// Currently all frames must be aligned in your tilemap.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    /// The start frame index in the tilemap atlas/array (inclusive).
    pub start: u32,
    /// The end frame index in the tilemap atlas/array (exclusive).
    pub end: u32,
    /// The speed the animation plays back at.
    pub speed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index_flattens_row_major() {
        let size = TilemapSize { x: 4, y: 4 };
        assert_eq!(TilePos::new(0, 0).to_index(&size), 0);
        assert_eq!(TilePos::new(1, 1).to_index(&size), 5);
        assert_eq!(TilePos::new(3, 3).to_index(&size), 15);
    }

    #[test]
    fn checked_to_index_rejects_out_of_bounds_positions() {
        let size = TilemapSize { x: 4, y: 4 };
        assert_eq!(TilePos::new(1, 1).to_checked_index(&size), Some(5));
        // `(5, 0)` flattens to `5` as well, aliasing `(1, 1)`, so it must be rejected.
        assert_eq!(TilePos::new(5, 0).to_checked_index(&size), None);
        assert_eq!(TilePos::new(0, u32::MAX).to_checked_index(&size), None);
    }

    // `usize` is 32 bits wide on targets such as wasm32, where `u32::MAX * 4` genuinely cannot be
    // represented, so this only holds on 64-bit targets.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn to_index_does_not_overflow_out_of_bounds() {
        let size = TilemapSize { x: 4, y: 4 };
        assert_eq!(
            TilePos::new(0, u32::MAX).to_index(&size),
            u32::MAX as usize * 4
        );
    }
}
