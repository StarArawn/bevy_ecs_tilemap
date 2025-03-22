use crate::{prelude::chunk_aabb, TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy::prelude::*;

/// How a tilemap is positioned relative to its [`Transform`]. It defaults to
/// `TilemapAnchor::None` which is the center of the bottom-left tile. Note that
/// `BottomLeft` refers to the bottom-left of the tilemap--not that tile's center.
#[derive(Debug, Clone, Copy, Component, Default, Reflect, PartialEq)]
#[reflect(Component, Default, Debug, PartialEq)]
pub enum TilemapAnchor {
    /// The center of the bottom-left tile
    #[default]
    None,
    Center,
    /// The bottom-left of the tilemap
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    /// Custom anchor point
    ///
    /// Top left is `(-0.5, 0.5)`, center is `(0.0, 0.0)`. The value will be
    /// scaled with the tilemap size.
    Custom(Vec2),
}

impl TilemapAnchor {
    /// Anchor's offset
    ///
    /// Background: The tilemap's original anchor is the center of the
    /// bottom-left tile.
    ///
    /// This offset is used to translate the tilemap to the given anchor.
    ///
    /// For instance a `TilemapAnchor::None` has an offset of `Vec2::ZERO` since
    /// it applies no translation, while a `TilemapAnchor::BottomLeft` has an
    /// offset of `Vec2::new(-grid_size.x, -grid_size.y) / 2.0` to move
    /// the anchor from the center of the bottom-left tile to the bottom-left of
    /// the tile and the tilemap.
    pub fn as_offset(
        &self,
        map_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        tile_size: &TilemapTileSize,
        map_type: &TilemapType,
    ) -> Vec2 {
        let aabb = chunk_aabb(
            UVec2::new(map_size.x - 1, map_size.y - 1),
            grid_size,
            tile_size,
            map_type,
        );
        let min = aabb.min();
        let max = aabb.max();
        match self {
            TilemapAnchor::None => Vec2::ZERO,
            TilemapAnchor::TopLeft => Vec2::new(-min.x, -max.y),
            TilemapAnchor::TopRight => Vec2::new(-max.x, -max.y),
            TilemapAnchor::TopCenter => Vec2::new(-(max.x + min.x) / 2.0, -max.y),
            TilemapAnchor::CenterRight => Vec2::new(-max.x, -(max.y + min.y) / 2.0),
            TilemapAnchor::CenterLeft => Vec2::new(-min.x, -(max.y + min.y) / 2.0),
            TilemapAnchor::BottomLeft => Vec2::new(-min.x, -min.y),
            TilemapAnchor::BottomRight => Vec2::new(-max.x, -min.y),
            TilemapAnchor::BottomCenter => Vec2::new(-(max.x + min.x) / 2.0, -min.y),
            TilemapAnchor::Center => Vec2::new(-(max.x + min.x) / 2.0, -(max.y + min.y) / 2.0),
            TilemapAnchor::Custom(v) => Vec2::new(
                (-0.5 - v.x) * (max.x - min.x) - min.x,
                (-0.5 - v.y) * (max.y - min.y) - min.y,
            ),
        }
    }
}
