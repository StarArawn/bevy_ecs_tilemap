use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapTileSize, TilemapType};
use bevy::math::{UVec2, Vec2};
use bevy::render::primitives::Aabb;

/// Calculates the world-space position of the bottom-left of the specified chunk.
pub fn chunk_index_to_world_space(
    chunk_index: UVec2,
    chunk_size: UVec2,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Vec2 {
    // Get the position of the bottom left tile of the chunk: the "anchor tile".
    let anchor_tile_pos = TilePos {
        x: chunk_index.x * chunk_size.x,
        y: chunk_index.y * chunk_size.y,
    };
    anchor_tile_pos.center_in_world(grid_size, map_type)
}

/// Calculates the [`Aabb`] of a generic chunk. The AABB depends upon the grid size, tile size, and
/// map type of the the chunk the tile belongs to.
///
/// The minimum is set at `(0.0, 0.0, 0.0)`. The maximum is set at
/// `(chunk_x_size_in_world_space, chunk_y_size_in_world_space, 1.0)`.
///
/// Note that the AABB must be transformed by a chunk's actual position in order for it to be
/// useful.
pub fn chunk_aabb(
    chunk_size: UVec2,
    grid_size: &TilemapGridSize,
    tile_size: &TilemapTileSize,
    map_type: &TilemapType,
) -> Aabb {
    let delta = Vec2::new(grid_size.x.max(tile_size.x), grid_size.y.max(tile_size.y));
    let minimum = Vec2::new(0.0, 0.0) - delta;
    let maximum =
        chunk_index_to_world_space(UVec2::new(1, 1), chunk_size, grid_size, map_type) + delta;
    Aabb::from_min_max(minimum.extend(0.0), maximum.extend(1.0))
}
