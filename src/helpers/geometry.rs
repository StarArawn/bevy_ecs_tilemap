use crate::{TilemapSize, TilemapTileSize, Transform};
use bevy::math::Vec2;

/// Calculates a [`Vec2`] position for a tilemap so that when set to this position, it shows up
/// centered on the screen.
pub fn get_tilemap_center(size: &TilemapSize, tile_size: &TilemapTileSize) -> Vec2 {
    Vec2::new(
        -(size.x as f32 * tile_size.x as f32) / 2.0,
        -(size.y as f32 * tile_size.y as f32) / 2.0,
    )
}

/// Calculates a [`Transform`] for a tilemap so that when set to this position, it shows up  
/// centered on the screen.
pub fn get_tilemap_center_transform(
    size: &TilemapSize,
    tile_size: &TilemapTileSize,
    z: f32,
) -> Transform {
    let center = get_tilemap_center(size, tile_size);
    Transform::from_xyz(center.x, center.y, z)
}
