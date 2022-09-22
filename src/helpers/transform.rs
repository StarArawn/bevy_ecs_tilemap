use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapType};
use bevy::math::{UVec2, Vec2};

/// Calculates the world-space position of the bottom-left of the specified chunk.
pub fn chunk_index_to_world_space(
    chunk_index: UVec2,
    chunk_size: UVec2,
    grid_size: Vec2,
    map_type: &TilemapType,
) -> Vec2 {
    // Get the position of the bottom left tile of the chunk: the "anchor tile".
    let anchor_tile_pos = TilePos {
        x: chunk_index.x * chunk_size.x,
        y: chunk_index.y * chunk_size.y,
    };
    let grid_size: TilemapGridSize = grid_size.into();
    anchor_tile_pos.center_in_world(&grid_size, map_type)
}
