use crate::helpers::hex_grid::neighbors::{HexColDirection, HexRowDirection};
use crate::helpers::square_grid::neighbors::SquareDirection;
use crate::map::{HexCoordSystem, IsoCoordSystem, TilemapType};
use crate::prelude::axial_system::AxialPos;
use crate::prelude::diamond_system::DiamondPos;
use crate::prelude::staggered_system::StaggeredPos;
use crate::prelude::SquarePos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, Transform};
use bevy::math::Vec2;

/// Gets the [`Vec2`] world space position of the center of the tilemap.
///
/// If the tilemap has odd dimensions, then this is just the center of the central tile in the map.
///
/// Otherwise, we must add an offset of "half-a-tile", or put differently, we must center the tile
/// on one of the corners/edges of the tile.
pub fn get_tilemap_center(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Vec2 {
    let center_pos = TilePos::new(size.x / 2.0 as u32, size.y / 2.0 as u32);

    // Note: when tile size is even, the center tile as calculated by `center_pos` above lies
    // *right* of the central point. Therefore, we have to move south/west to the center's location,
    // not north/east.
    match (size.x % 2, size.y % 2) {
        (1, 1) => center_pos.center_in_world(grid_size, map_type),
        (0, 1) => match map_type {
            TilemapType::Square => {
                let center = SquarePos::from(&center_pos);
                center.corner_in_world(SquareDirection::West, grid_size)
            }
            TilemapType::Hexagon(hex_coord_system) => {
                let center =
                    AxialPos::from_tile_pos_given_coord_system(&center_pos, *hex_coord_system);
                match hex_coord_system {
                    HexCoordSystem::RowEven | HexCoordSystem::RowOdd | HexCoordSystem::Row => {
                        (center.corner_in_world_row(HexRowDirection::SouthWest, grid_size)
                            + center.corner_in_world_row(HexRowDirection::NorthWest, grid_size))
                            / 2.0
                    }
                    _ => center.corner_in_world_col(HexColDirection::West, grid_size),
                }
            }
            TilemapType::Isometric(iso_coord_system) => match iso_coord_system {
                IsoCoordSystem::Diamond => {
                    let center = DiamondPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::West, grid_size)
                }
                IsoCoordSystem::Staggered => {
                    let center = StaggeredPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::West, grid_size)
                }
            },
        },
        (1, 0) => match map_type {
            TilemapType::Square => {
                let center = SquarePos::from(&center_pos);
                center.corner_in_world(SquareDirection::South, grid_size)
            }
            TilemapType::Hexagon(hex_coord_system) => {
                let center =
                    AxialPos::from_tile_pos_given_coord_system(&center_pos, *hex_coord_system);
                match hex_coord_system {
                    HexCoordSystem::RowEven | HexCoordSystem::RowOdd | HexCoordSystem::Row => {
                        center.corner_in_world_row(HexRowDirection::South, grid_size)
                    }
                    _ => {
                        (center.corner_in_world_col(HexColDirection::SouthWest, grid_size)
                            + center.corner_in_world_col(HexColDirection::SouthEast, grid_size))
                            / 2.0
                    }
                }
            }
            TilemapType::Isometric(iso_coord_system) => match iso_coord_system {
                IsoCoordSystem::Diamond => {
                    let center = DiamondPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::South, grid_size)
                }
                IsoCoordSystem::Staggered => {
                    let center = StaggeredPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::South, grid_size)
                }
            },
        },
        (0, 0) => match map_type {
            TilemapType::Square => {
                let center = SquarePos::from(&center_pos);
                center.corner_in_world(SquareDirection::SouthWest, grid_size)
            }
            TilemapType::Hexagon(hex_coord_system) => {
                let center =
                    AxialPos::from_tile_pos_given_coord_system(&center_pos, *hex_coord_system);
                match hex_coord_system {
                    HexCoordSystem::RowEven | HexCoordSystem::RowOdd | HexCoordSystem::Row => {
                        center.corner_in_world_row(HexRowDirection::SouthWest, grid_size)
                    }
                    _ => center.corner_in_world_col(HexColDirection::SouthWest, grid_size),
                }
            }
            TilemapType::Isometric(iso_coord_system) => match iso_coord_system {
                IsoCoordSystem::Diamond => {
                    let center = DiamondPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::SouthWest, grid_size)
                }
                IsoCoordSystem::Staggered => {
                    let center = StaggeredPos::from(&center_pos);
                    center.corner_in_world(SquareDirection::SouthWest, grid_size)
                }
            },
        },
        (_, _) => {
            // a number modulo 2 can only be 0 or 1
            unreachable!()
        }
    }
}

/// Calculates a [`Transform`] for a tilemap that places it so that its center is at
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
