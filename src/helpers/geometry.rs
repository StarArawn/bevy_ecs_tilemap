use crate::{TilemapGridSize, TilemapSize, Transform};
use bevy::math::Vec2;

/// Calculates a [`Vec2`] position for a tilemap so that when set to this position, it shows up
/// centered on the screen.
pub fn get_tilemap_center(size: &TilemapSize, grid_size: &TilemapGridSize) -> Vec2 {
    Vec2::new(
        -(size.x as f32 * grid_size.x) / 2.0,
        -(size.y as f32 * grid_size.y) / 2.0,
    )
}

/// Calculates a [`Transform`] for a tilemap so that when set to this position, it shows up  
/// centered on the screen.
pub fn get_tilemap_center_transform(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    z: f32,
) -> Transform {
    let center = get_tilemap_center(size, grid_size);
    Transform::from_xyz(center.x, center.y, z)
}
