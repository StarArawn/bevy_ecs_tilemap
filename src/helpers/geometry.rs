use crate::map::TilemapType;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, Transform};
use bevy::math::Vec2;

/// Gets the [`Vec2`] world space position of the center tile of the tilemap.
///
/// If the tilemap has odd dimensions, then this is just the center of the central tile in the map.
///
/// Otherwise, we must add an offset of half-a-tile.
pub fn get_tilemap_center(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Vec2 {
    let center_tile_pos = TilePos::new(size.x / 2.0 as u32, size.y / 2.0 as u32);

    let mut offset = Vec2::new(0.0, 0.0);
    // If the tilemap is even in the x-direction
    if size.x % 2 == 0 {
        offset.x -= grid_size.x / 2.0;
    }
    // If the tilemap is even in the y-direction
    if size.y % 2 == 0 {
        offset.y -= grid_size.y / 2.0;
    }

    center_tile_pos.center_in_world(grid_size, map_type) + offset
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
