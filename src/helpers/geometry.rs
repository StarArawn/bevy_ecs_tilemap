use crate::map::TilemapType;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, Transform};
use bevy::math::Vec2;

/// Gets the [`Vec2`] world space position of the center tile in the tilemap.
///
/// The center tile is defined to be the tile at
/// `TilePos { x: size.x / 2.0 as u32, y: size.y / 2.0 as u32 }`, where `size` is a `TilemapSize`.
pub fn get_tilemap_center(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Vec2 {
    let center_pos = TilePos::new(size.x / 2.0 as u32, size.y / 2.0 as u32);
    center_pos.center_in_world(grid_size, map_type)
}

/// Calculates a [`Transform`] for a tilemap that places it so that the center tile is at
/// `(0.0, 0.0, 0.0)` in world space.
pub fn get_tilemap_center_transform(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    z: f32,
) -> Transform {
    let center = get_tilemap_center(size, grid_size, map_type);
    Transform::from_xyz(-center.x, -center.y, z)
}
