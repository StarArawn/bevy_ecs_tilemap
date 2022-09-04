use crate::{TilemapSize, TilemapTileSize};
use bevy::prelude::Transform;

/// Calculates a tilemap's centered position.
pub fn calculate_tilemap_center(
    size: &TilemapSize,
    tile_size: &TilemapTileSize,
    z_index: f32,
) -> Transform {
    Transform::from_xyz(
        -(size.x as f32 * tile_size.x as f32) / 2.0,
        -(size.y as f32 * tile_size.y as f32) / 2.0,
        z_index,
    )
}


