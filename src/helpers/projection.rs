use crate::helpers::hex_grid::axial::AxialPos;
use crate::helpers::hex_grid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::helpers::square_grid::diamond::DiamondPos;
use crate::helpers::square_grid::staggered::StaggeredPos;
use crate::map::{HexCoordSystem, IsoCoordSystem};
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, TilemapType};
use bevy::math::Vec2;

impl TilePos {
    /// Get the center of this tile in world space.
    ///
    /// The center is well defined for all [`TilemapType`]s.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize, map_type: &TilemapType) -> Vec2 {
        match map_type {
            TilemapType::Square { .. } => {
                Vec2::new(grid_size.x * (self.x as f32), grid_size.y * (self.y as f32))
            }
            TilemapType::Hexagon(hex_coord_sys) => match hex_coord_sys {
                HexCoordSystem::RowEven => RowEvenPos::from(self).center_in_world(grid_size),
                HexCoordSystem::RowOdd => RowOddPos::from(self).center_in_world(grid_size),
                HexCoordSystem::ColumnEven => ColEvenPos::from(self).center_in_world(grid_size),
                HexCoordSystem::ColumnOdd => ColOddPos::from(self).center_in_world(grid_size),
                HexCoordSystem::Row => AxialPos::from(self).center_in_world_row(grid_size),
                HexCoordSystem::Column => AxialPos::from(self).center_in_world_col(grid_size),
            },
            TilemapType::Isometric(coord_system) => match coord_system {
                IsoCoordSystem::Diamond => DiamondPos::from(self).center_in_world(grid_size),
                IsoCoordSystem::Staggered => StaggeredPos::from(self).center_in_world(grid_size),
            },
        }
    }

    /// Try converting a pair of `i32` numbers into a `TilePos`.
    ///
    /// Returns `None` if either one of `x` or `y` is negative, or lies out of the bounds of
    /// `map_size`.
    pub fn from_i32_pair(x: i32, y: i32, map_size: &TilemapSize) -> Option<TilePos> {
        if x < 0 || y < 0 {
            None
        } else {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };

            if tile_pos.within_map_bounds(map_size) {
                Some(tile_pos)
            } else {
                None
            }
        }
    }

    pub fn from_world_pos(
        world_pos: &Vec2,
        map_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        map_type: &TilemapType,
    ) -> Option<TilePos> {
        match map_type {
            TilemapType::Square { .. } => {
                let x = ((world_pos.x / grid_size.x) + 0.5).floor() as i32;
                let y = ((world_pos.y / grid_size.y) + 0.5).floor() as i32;

                TilePos::from_i32_pair(x, y, map_size)
            }
            TilemapType::Hexagon(hex_coord_sys) => match hex_coord_sys {
                HexCoordSystem::RowEven => RowEvenPos::from_world_pos(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
                HexCoordSystem::RowOdd => RowOddPos::from_world_pos(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
                HexCoordSystem::ColumnEven => ColEvenPos::from_world_pos(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
                HexCoordSystem::ColumnOdd => ColOddPos::from_world_pos(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
                HexCoordSystem::Row => AxialPos::from_world_pos_row(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
                HexCoordSystem::Column => AxialPos::from_world_pos_col(world_pos, grid_size)
                    .as_tile_pos_given_map_size(map_size),
            },
            TilemapType::Isometric(coord_system) => match coord_system {
                IsoCoordSystem::Diamond => {
                    DiamondPos::from_world_pos(world_pos, grid_size).as_tile_pos(map_size)
                }
                IsoCoordSystem::Staggered => {
                    StaggeredPos::from_world_pos(world_pos, grid_size).as_tile_pos(map_size)
                }
            },
        }
    }
}
