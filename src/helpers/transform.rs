use crate::helpers::projection::{project_iso_diamond, project_iso_staggered};
use crate::map::{HexCoordSystem, IsoCoordSystem};
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapType};
use bevy::math::{UVec2, Vec2};
use bevy::prelude::Transform;

/// Calculates the world-space position of the bottom-left of the specified chunk.
pub fn chunk_index_to_world_units(
    chunk_index: UVec2,
    chunk_size: UVec2,
    z_index: u32,
    grid_size: Vec2,
    map_type: &TilemapType,
) -> Transform {
    // Get the position of the bottom left tile of the chunk: the "anchor tile".
    let anchor_tile_pos = TilePos {
        x: chunk_index.x * chunk_size.x,
        y: chunk_index.y * chunk_size.y,
    };
    let grid_size: TilemapGridSize = grid_size.into();
    // Now get the position of the anchor tile.
    let r = tile_pos_to_world_units(&anchor_tile_pos, &grid_size, map_type);
    Transform::from_xyz(r.x, r.y, z_index as f32)
}

/// Returns the bottom-left coordinate of the tile at specified `tile_pos`, in world units.
pub fn tile_pos_to_world_units(
    tile_pos: &TilePos,
    grid_size: &TilemapGridSize,
    tilemap_type: &TilemapType,
) -> Vec2 {
    let tile_pos_f32: Vec2 = tile_pos.into();
    let grid_size: Vec2 = grid_size.into();
    let mut pos = Vec2::new(grid_size.x * tile_pos_f32.x, grid_size.y * tile_pos_f32.y);

    match tilemap_type {
        TilemapType::Hexagon(HexCoordSystem::Row) => {
            let x_offset = tile_pos_f32.y * (0.5 * grid_size.x).floor();
            let y_offset = -1.0 * tile_pos_f32.y * (0.25 * grid_size.y).ceil();
            pos.x += x_offset;
            pos.y += y_offset;
        }
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            let offset = (0.25 * grid_size.x).floor();
            if tile_pos.y % 2 == 0 {
                pos.x -= offset;
            } else {
                pos.x += offset;
            }
            pos.y -= tile_pos_f32.y * (0.25 * grid_size.y as f32).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            let offset = (0.25 * grid_size.x).floor();
            if tile_pos.y % 2 == 0 {
                pos.x += offset;
            } else {
                pos.x -= offset;
            }
            pos.y -= tile_pos_f32.y * (0.25 * grid_size.y).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::Column) => {
            let x_offset = -1.0 * tile_pos_f32.x * (0.25 * grid_size.x).floor();
            let y_offset = tile_pos_f32.x * (0.5 * grid_size.y).ceil();
            pos.x += x_offset;
            pos.y += y_offset;
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            let offset = (0.25 * grid_size.y).floor();
            if tile_pos.x % 2 == 0 {
                pos.y -= offset;
            } else {
                pos.y += offset;
            }
            pos.x -= tile_pos_f32.x * (0.25 * grid_size.x as f32).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            let offset = (0.25 * grid_size.y).floor();
            if tile_pos.x % 2 == 0 {
                pos.y += offset;
            } else {
                pos.y -= offset;
            }
            pos.x -= tile_pos_f32.x * (0.25 * grid_size.x).ceil();
        }
        TilemapType::Isometric {
            coord_system: IsoCoordSystem::Diamond,
            ..
        } => {
            pos = project_iso_diamond(tile_pos_f32.x, tile_pos_f32.y, grid_size.x, grid_size.y);
        }
        TilemapType::Isometric {
            coord_system: IsoCoordSystem::Staggered,
            ..
        } => {
            pos = project_iso_staggered(tile_pos_f32.x, tile_pos_f32.y, grid_size.x, grid_size.y);
        }
        TilemapType::Square { .. } => {}
    };
    pos
}
