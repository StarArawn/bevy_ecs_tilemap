use crate::map::{HexCoordSystem, IsoCoordSystem};
use crate::tiles::TilePos;
use crate::{TileStorage, TilemapSize, TilemapType};
use bevy::prelude::Entity;

#[derive(Clone, Copy, Debug)]
pub enum NeighborDirection {
    North,
    NorthWest,
    West,
    SouthWest,
    South,
    SouthEast,
    East,
    NorthEast,
}

impl NeighborDirection {
    fn next_direction(&self) -> NeighborDirection {
        use NeighborDirection::*;
        match self {
            North => NorthWest,
            NorthWest => West,
            West => SouthWest,
            SouthWest => South,
            South => SouthEast,
            SouthEast => East,
            East => NorthEast,
            NorthEast => North,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Neighbors<T: Copy> {
    pub north: Option<T>,
    pub north_west: Option<T>,
    pub west: Option<T>,
    pub south_west: Option<T>,
    pub south: Option<T>,
    pub south_east: Option<T>,
    pub east: Option<T>,
    pub north_east: Option<T>,
}

pub struct NeighborsIntoIterator<T: Copy> {
    neighbors: Neighbors<T>,
    /// The next direction the iterator will output.
    cursor: Option<NeighborDirection>,
}

impl<T: Copy> NeighborsIntoIterator<T> {
    fn get_at_cursor(&self) -> Option<T> {
        self.cursor.and_then(|direction| match direction {
            NeighborDirection::North => self.neighbors.north,
            NeighborDirection::NorthWest => self.neighbors.north_west,
            NeighborDirection::West => self.neighbors.west,
            NeighborDirection::SouthWest => self.neighbors.south_west,
            NeighborDirection::South => self.neighbors.south,
            NeighborDirection::SouthEast => self.neighbors.south_east,
            NeighborDirection::East => self.neighbors.east,
            NeighborDirection::NorthEast => self.neighbors.north_east,
        })
    }
}

impl<T: Copy> Iterator for NeighborsIntoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.and_then(|direction| {
            let neighbor = self.get_at_cursor();
            match direction {
                NeighborDirection::NorthEast => {
                    self.cursor = None;
                    neighbor
                }
                direction => {
                    self.cursor = Some(direction.next_direction());
                    neighbor.or_else(|| self.next())
                }
            }
        })
    }
}

impl<T: Copy> IntoIterator for Neighbors<T> {
    type Item = T;
    type IntoIter = NeighborsIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        NeighborsIntoIterator {
            neighbors: self,
            cursor: Some(NeighborDirection::North),
        }
    }
}

impl<T: Copy> Neighbors<T> {
    pub fn count(&self) -> usize {
        self.into_iter().map(|_| 1).sum()
    }
}

impl Neighbors<Entity> {
    fn from_neighboring_pos(
        neighbors: &Neighbors<TilePos>,
        tile_storage: &TileStorage,
    ) -> Neighbors<Entity> {
        Neighbors {
            north: neighbors.north.and_then(|pos| tile_storage.get(&pos)),
            south: neighbors.south.and_then(|pos| tile_storage.get(&pos)),
            east: neighbors.east.and_then(|pos| tile_storage.get(&pos)),
            west: neighbors.west.and_then(|pos| tile_storage.get(&pos)),
            north_east: neighbors.north_east.and_then(|pos| tile_storage.get(&pos)),
            north_west: neighbors.north_west.and_then(|pos| tile_storage.get(&pos)),
            south_east: neighbors.south_east.and_then(|pos| tile_storage.get(&pos)),
            south_west: neighbors.south_west.and_then(|pos| tile_storage.get(&pos)),
        }
    }
}

/// Retrieves a list of neighbors for the given tile position.
///
/// If a particular neighboring position does not exist in the provided storage,
/// then it will not be returned.
pub fn get_tile_neighbors(
    tile_pos: &TilePos,
    tile_storage: &TileStorage,
    tilemap_type: &TilemapType,
) -> Neighbors<Entity> {
    Neighbors::from_neighboring_pos(
        &get_neighboring_pos(tile_pos, &tile_storage.size, tilemap_type),
        tile_storage,
    )
}

/// Retrieves the positions of neighbors of the tile with the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn get_neighboring_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
    map_type: &TilemapType,
) -> Neighbors<TilePos> {
    match map_type {
        TilemapType::Square {
            neighbors_include_diagonals: true,
        } => square_neighbor_pos_with_diagonals(tile_pos, tilemap_size),
        TilemapType::Square {
            neighbors_include_diagonals: false,
        } => square_neighbor_pos(tile_pos, tilemap_size),
        TilemapType::Isometric {
            neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Diamond,
        } => {
            if *neighbors_include_diagonals {
                diamond_neighbor_pos_with_diagonals(tile_pos, tilemap_size)
            } else {
                diamond_neighbor_pos(tile_pos, tilemap_size)
            }
        }
        TilemapType::Isometric {
            neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Staggered,
        } => {
            if *neighbors_include_diagonals {
                staggered_neighbor_pos_with_diagonals(tile_pos, tilemap_size)
            } else {
                staggered_neighbor_pos(tile_pos, tilemap_size)
            }
        }
        TilemapType::Hexagon(HexCoordSystem::Row) => hex_row_neighbor_pos(tile_pos, tilemap_size),
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            hex_row_even_neighbor_pos(tile_pos, tilemap_size)
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            hex_row_odd_neighbor_pos(tile_pos, tilemap_size)
        }
        TilemapType::Hexagon(HexCoordSystem::Column) => {
            hex_col_neighbor_pos(tile_pos, tilemap_size)
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            hex_col_even_neighbor_pos(tile_pos, tilemap_size)
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            hex_col_odd_neighbor_pos(tile_pos, tilemap_size)
        }
    }
}

impl TilePos {
    #[inline]
    fn plus_x(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y < tilemap_size.y - 1 {
            Some(TilePos {
                x: self.x,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_xy(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 && self.y < tilemap_size.y - 1 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_x(&self) -> Option<TilePos> {
        if self.x != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_y(&self) -> Option<TilePos> {
        if self.y != 0 {
            Some(TilePos {
                x: self.x,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_xy(&self) -> Option<TilePos> {
        if self.x != 0 && self.y != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_x_minus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 && self.y != 0 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_x_minus_2y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 && self.y > 1 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y - 2,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_x_plus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y < tilemap_size.y - 1 && self.x != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_x_plus_2y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y < tilemap_size.y - 2 && self.x != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y + 2,
            })
        } else {
            None
        }
    }

    fn square_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_y(tilemap_size)
    }

    fn square_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn square_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn square_south_west(&self) -> Option<TilePos> {
        self.minus_xy()
    }

    fn square_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn square_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn square_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn square_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_xy(tilemap_size)
    }

    fn hex_row_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn hex_row_odd_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_y(tilemap_size)
        } else {
            self.minus_x_plus_y(tilemap_size)
        }
    }

    fn hex_row_even_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_y(tilemap_size)
        } else {
            self.minus_x_plus_y(tilemap_size)
        }
    }

    fn hex_row_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_odd_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_even_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_south_west(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_row_odd_south_west(&self) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.minus_y()
        } else {
            self.minus_xy()
        }
    }

    fn hex_row_even_south_west(&self) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.minus_y()
        } else {
            self.minus_xy()
        }
    }

    fn hex_row_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn hex_row_odd_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_x_minus_y(tilemap_size)
        } else {
            self.minus_y()
        }
    }

    fn hex_row_even_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_x_minus_y(tilemap_size)
        } else {
            self.minus_y()
        }
    }

    fn hex_row_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_odd_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_even_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_y(tilemap_size)
    }

    fn hex_row_odd_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_row_even_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_col_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_odd_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_even_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn hex_col_odd_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.minus_x_plus_y(tilemap_size)
        } else {
            self.minus_x()
        }
    }

    fn hex_col_even_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.minus_x_plus_y(tilemap_size)
        } else {
            self.minus_x()
        }
    }

    fn hex_col_south_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_col_odd_south_west(&self) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.minus_x()
        } else {
            self.minus_xy()
        }
    }

    fn hex_col_even_south_west(&self) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.minus_x()
        } else {
            self.minus_xy()
        }
    }

    fn hex_col_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_odd_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_even_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn hex_col_odd_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.plus_x(tilemap_size)
        } else {
            self.plus_x_minus_y(tilemap_size)
        }
    }

    fn hex_col_even_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.plus_x(tilemap_size)
        } else {
            self.plus_x_minus_y(tilemap_size)
        }
    }

    fn hex_col_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_odd_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_col_even_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn iso_staggered_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_y(tilemap_size)
    }

    fn iso_staggered_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_2y(tilemap_size)
    }

    fn iso_staggered_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn iso_staggered_south_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn iso_staggered_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn iso_staggered_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_2y(tilemap_size)
    }

    fn iso_staggered_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn iso_staggered_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// that 1) the tile exists on [`Square`](crate::map::TilemapType::Square) tilemap
/// and 2) neighbors **do** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn square_neighbor_pos_with_diagonals(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.square_north(tilemap_size),
        north_west: tile_pos.square_north_west(tilemap_size),
        west: tile_pos.square_west(),
        south_west: tile_pos.square_south_west(),
        south: tile_pos.square_south(),
        south_east: tile_pos.square_south_east(tilemap_size),
        east: tile_pos.square_east(tilemap_size),
        north_east: tile_pos.square_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the 1) tile exists on a [`Isometric`](crate::map::Isometric) tilemap with [`Diamond`](crate::map::IsoCoordSystem::Staggered) coordinate system,
/// and 2) neighbors **do** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn staggered_neighbor_pos_with_diagonals(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.iso_staggered_north(tilemap_size),
        north_west: tile_pos.iso_staggered_north_west(tilemap_size),
        west: tile_pos.iso_staggered_west(tilemap_size),
        south_west: tile_pos.iso_staggered_south_west(),
        south: tile_pos.iso_staggered_south(),
        south_east: tile_pos.iso_staggered_south_east(tilemap_size),
        east: tile_pos.iso_staggered_east(tilemap_size),
        north_east: tile_pos.iso_staggered_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the 1) tile exists on a [`Isometric`](crate::map::Isometric) tilemap with [`Diamond`](crate::map::IsoCoordSystem::Diamond) coordinate system,
/// and 2) neighbors **do** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
///
/// Note that is equivalent to calling [`square_neighbor_pos_with_diagonals`](crate::helpers::square_neighbor_pos_with_diagonals) as the connectivity of the graph underlying
/// [`Square`](crate::map::TilemapType::Square) is the same as the connectivity of the graph underlying
/// [`Isometric`](crate::map::TilemapType::Isometric) with coordinate system [`Diamond`](crate::map::IsoCoordSystem::Diamond).
pub fn diamond_neighbor_pos_with_diagonals(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    square_neighbor_pos_with_diagonals(tile_pos, tilemap_size)
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// that 1) the tile exists on [`Square`](crate::map::TilemapType::Square) tilemap
/// and 2) neighbors **do not** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn square_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.square_north(tilemap_size),
        north_west: None,
        west: tile_pos.square_west(),
        south_west: None,
        south: tile_pos.square_south(),
        south_east: None,
        east: tile_pos.square_east(tilemap_size),
        north_east: None,
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the 1) tile exists on a [`Isometric`](crate::map::Isometric) tilemap with [`Diamond`](crate::map::IsoCoordSystem::Diamond) coordinate system,
/// and 2) neighbors **do not** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
///
/// Note that is equivalent to calling [`square_neighbor_pos`](crate::helpers::square_neighbor_pos) as the connectivity of the graph underlying
/// [`TilemapType::Square`](crate::map::TilemapType::Square) is the same as the connectivity of the graph underlying
/// [`TilemapType::Isometric`](crate::map::TilemapType::Isometric) with coordinate system [`Diamond`](crate::map::IsoCoordSystem::Diamond).
pub fn diamond_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    square_neighbor_pos(tile_pos, tilemap_size)
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the 1) tile exists on a [`Isometric`](crate::map::Isometric) tilemap with [`Diamond`](crate::map::IsoCoordSystem::Staggered) coordinate system,
/// and 2) neighbors **do not** include tiles located diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn staggered_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.iso_staggered_north(tilemap_size),
        north_west: None,
        west: tile_pos.iso_staggered_west(tilemap_size),
        south_west: None,
        south: tile_pos.iso_staggered_south(),
        south_east: None,
        east: tile_pos.iso_staggered_east(tilemap_size),
        north_east: None,
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap with
/// coordinate system [`HexCoordSystem::Row`](crate::map::HexCoordSystem::Row).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_north_west(tilemap_size),
        west: tile_pos.hex_row_west(),
        south_west: tile_pos.hex_row_south_west(),
        south: None,
        south_east: tile_pos.hex_row_south_east(tilemap_size),
        east: tile_pos.hex_row_east(tilemap_size),
        north_east: tile_pos.hex_row_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap
/// with coordinate system [`HexCoordSystem::RowOdd`](crate::map::HexCoordSystem::RowOdd).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_odd_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_odd_north_west(tilemap_size),
        west: tile_pos.hex_row_odd_west(),
        south_west: tile_pos.hex_row_odd_south_west(),
        south: None,
        south_east: tile_pos.hex_row_odd_south_east(tilemap_size),
        east: tile_pos.hex_row_odd_east(tilemap_size),
        north_east: tile_pos.hex_row_odd_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap with
/// coordinate system [`HexCoordSystem::Row`](crate::map::HexCoordSystem::RowEven).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_even_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_north_west(tilemap_size),
        west: tile_pos.hex_row_west(),
        south_west: tile_pos.hex_row_south_west(),
        south: None,
        south_east: tile_pos.hex_row_south_east(tilemap_size),
        east: tile_pos.hex_row_east(tilemap_size),
        north_east: tile_pos.hex_row_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap with
/// coordinate system [`HexCoordSystem::Col`](crate::map::HexCoordSystem::Col).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_north(tilemap_size),
        north_west: tile_pos.hex_col_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_south_west(),
        south: tile_pos.hex_col_south(),
        south_east: tile_pos.hex_col_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap with
/// coordinate system [`HexCoordSystem::ColOdd`](crate::map::HexCoordSystem::ColOdd).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_odd_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_odd_north(tilemap_size),
        north_west: tile_pos.hex_col_odd_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_odd_south_west(),
        south: tile_pos.hex_col_odd_south(),
        south_east: tile_pos.hex_col_odd_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_odd_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a [`Hexagon`](crate::map::TilemapType::Hexagon) tilemap with
/// coordinate system [`HexCoordSystem::ColEven`](crate::map::HexCoordSystem::ColEven).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_even_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_even_north(tilemap_size),
        north_west: tile_pos.hex_col_even_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_even_south_west(),
        south: tile_pos.hex_col_even_south(),
        south_east: tile_pos.hex_col_even_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_even_north_east(tilemap_size),
    }
}
