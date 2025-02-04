use crate::{prelude::chunk_aabb, TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Component, Default)]
pub enum TilemapAnchor {
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
    Custom(Vec2),
    // TileCenter(TilePos),
}

impl TilemapAnchor {
    pub fn from_map(
        &self,
        map_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        tile_size: &TilemapTileSize,
        map_type: &TilemapType,
    ) -> Transform {
        let aabb = chunk_aabb(
            UVec2::new(map_size.x - 1, map_size.y - 1),
            grid_size,
            tile_size,
            map_type,
        );
        let min = aabb.min();
        let max = aabb.max();
        match self {
            TilemapAnchor::None => Transform::IDENTITY,
            TilemapAnchor::TopLeft => Transform::from_xyz(-min.x, -max.y, 0.0),
            TilemapAnchor::TopRight => Transform::from_xyz(-max.x, -max.y, 0.0),
            TilemapAnchor::TopCenter => Transform::from_xyz(-(max.x + min.x) / 2.0, -max.y, 0.0),
            TilemapAnchor::CenterRight => Transform::from_xyz(-max.x, -(max.y + min.y) / 2.0, 0.0),
            TilemapAnchor::CenterLeft => Transform::from_xyz(-min.x, -(max.y + min.y) / 2.0, 0.0),
            TilemapAnchor::BottomLeft => Transform::from_xyz(-min.x, -min.y, 0.0),
            TilemapAnchor::BottomRight => Transform::from_xyz(-max.x, -min.y, 0.0),
            TilemapAnchor::BottomCenter => Transform::from_xyz(-(max.x + min.x) / 2.0, -min.y, 0.0),
            TilemapAnchor::Center => {
                Transform::from_xyz(-(max.x + min.x) / 2.0, -(max.y + min.y) / 2.0, 0.0)
            }
            TilemapAnchor::Custom(v) => Transform::from_xyz(
                (-0.5 - v.x) * (max.x - min.x) - min.x,
                (-0.5 - v.y) * (max.y - min.y) - min.y,
                0.0,
            ),
        }
    }

}
