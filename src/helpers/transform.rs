use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapTileSize, TilemapType};
use bevy::math::{UVec2, Vec2, Vec3};
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
    // The AABB minimum and maximum have to be modified by -border and +border respectively.
    let border = Vec2::from(grid_size).max(tile_size.into());

    // For most map types, it would be sufficient to calculate c0 and c3. However, for some map
    // types (right now, isometric diamond), this would not work, and for robustness (especially
    // with respect map types added in the future), we calculate all four corner points, and
    // then minimize/maximize them.
    //
    // Alternatively, a map-type specific calculations could be executed here.
    let c0 = chunk_index_to_world_space(UVec2::new(0, 0), chunk_size, grid_size, map_type);
    let c1 = chunk_index_to_world_space(UVec2::new(1, 0), chunk_size, grid_size, map_type);
    let c2 = chunk_index_to_world_space(UVec2::new(0, 1), chunk_size, grid_size, map_type);
    let c3 = chunk_index_to_world_space(UVec2::new(1, 1), chunk_size, grid_size, map_type);

    let minimum = Vec3::from((c0.min(c1).min(c2).min(c3) - border, 0.0));
    let maximum = Vec3::from((c0.max(c1).max(c2).max(c3) + border, 1.0));
    Aabb::from_min_max(minimum, maximum)
}
