use crate::{prelude::chunk_aabb, TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy::prelude::*;
use std::borrow::Cow;

/// How a tilemap is positioned relative to its [`Transform`]. It defaults to
/// `TilemapAnchor::None` which is the center of the bottom-left tile.
#[derive(Debug, Clone, Copy, Component, Default, Reflect, PartialEq)]
#[reflect(Component, Default, Debug, PartialEq)]
pub enum TilemapAnchor {
    /// The center of the bottom-left tile.
    #[default]
    None,
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    /// Custom anchor point. Top left is `(-0.5, 0.5)`, center is `(0.0, 0.0)`. The value will
    /// be scaled with the tilemap size.
    Custom(Vec2),
    // TileCenter(TilePos),
}

impl TilemapAnchor {
    /// Provides an offset from the center of the bottom-left tile with no
    /// anchor to the center of the bottom-left tile with the given anchor.
    pub(crate) fn as_offset(
        &self,
        map_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        tile_size: Option<&TilemapTileSize>,
        map_type: &TilemapType,
    ) -> Vec2 {
        let tile_size = tile_size
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(TilemapTileSize::new(grid_size.x, grid_size.y)));
        let aabb = chunk_aabb(
            UVec2::new(map_size.x - 1, map_size.y - 1),
            grid_size,
            &tile_size,
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
