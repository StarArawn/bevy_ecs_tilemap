use crate::helpers::hex_grid::axial::AxialPos;
use crate::map::HexCoordSystem;
use crate::TilePos;
use std::ops::{Add, Sub};

/// Neighbors of a hexagonal tile. `Zero` corresponds with `East` for row-oriented tiles, and
/// `North` for column-oriented tiles. It might also correspond with a custom direction for
/// hex maps which are oriented by a non-standard rotation.
///
/// Mathematically: `Zero` corresponds with "zero multiples of `pi/3`", `One` with "one multiple of
/// `pi/3`", and so on.
///
/// [`HexDirection`]s can be added, and subtracted (under the hood, it is addition/subtraction
/// modulo 6).
///
/// [`HexDirection`]s can be converted from/into `usize`, `u32`, `isize`, `i32`.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum HexDirection {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
}

/// Array of [`HexDirection`] variants.
pub const HEX_DIRECTIONS: [HexDirection; 6] = [
    HexDirection::Zero,
    HexDirection::One,
    HexDirection::Two,
    HexDirection::Three,
    HexDirection::Four,
    HexDirection::Five,
];

/// Offsets of tiles that lie in each [`HexDirection`].
pub const HEX_OFFSETS: [AxialPos; 6] = [
    AxialPos { q: 1, r: 0 },
    AxialPos { q: 0, r: 1 },
    AxialPos { q: -1, r: 1 },
    AxialPos { q: -1, r: 0 },
    AxialPos { q: 0, r: -1 },
    AxialPos { q: 1, r: -1 },
];

impl From<HexDirection> for AxialPos {
    fn from(direction: HexDirection) -> Self {
        HEX_OFFSETS[direction as usize]
    }
}

impl From<&HexDirection> for AxialPos {
    fn from(direction: &HexDirection) -> Self {
        AxialPos::from(*direction)
    }
}

impl From<usize> for HexDirection {
    fn from(choice: usize) -> Self {
        let ix = choice % 6;
        HEX_DIRECTIONS[ix]
    }
}

impl From<isize> for HexDirection {
    fn from(choice: isize) -> Self {
        // The Euclidean remainder is always positive, so it is safe to convert to usize;
        let ix = choice.rem_euclid(6) as usize;
        HEX_DIRECTIONS[ix]
    }
}

impl From<u32> for HexDirection {
    fn from(choice: u32) -> Self {
        (choice as usize).into()
    }
}

impl From<i32> for HexDirection {
    fn from(choice: i32) -> Self {
        (choice as isize).into()
    }
}

impl Add<usize> for HexDirection {
    type Output = HexDirection;

    fn add(self, rhs: usize) -> Self::Output {
        ((self as usize) + rhs).into()
    }
}

impl Add<u32> for HexDirection {
    type Output = HexDirection;

    fn add(self, rhs: u32) -> Self::Output {
        ((self as usize) + rhs as usize).into()
    }
}

impl Add<isize> for HexDirection {
    type Output = HexDirection;

    fn add(self, rhs: isize) -> Self::Output {
        ((self as isize) + rhs).into()
    }
}

impl Add<i32> for HexDirection {
    type Output = HexDirection;

    fn add(self, rhs: i32) -> Self::Output {
        ((self as i32) + rhs).into()
    }
}

impl Sub<usize> for HexDirection {
    type Output = HexDirection;

    fn sub(self, rhs: usize) -> Self::Output {
        ((self as usize) - rhs).into()
    }
}

impl Sub<u32> for HexDirection {
    type Output = HexDirection;

    fn sub(self, rhs: u32) -> Self::Output {
        ((self as usize) - rhs as usize).into()
    }
}

impl Sub<isize> for HexDirection {
    type Output = HexDirection;

    fn sub(self, rhs: isize) -> Self::Output {
        ((self as isize) - rhs).into()
    }
}

impl Sub<i32> for HexDirection {
    type Output = HexDirection;

    fn sub(self, rhs: i32) -> Self::Output {
        ((self as i32) - rhs).into()
    }
}

impl HexDirection {
    pub fn offset(&self, tile_pos: &TilePos, coord_sys: HexCoordSystem) -> TilePos {
        AxialPos::from_tile_pos_given_coord_system(tile_pos, coord_sys)
            .offset(*self)
            .as_tile_pos_given_coord_system(coord_sys)
    }
}

/// Compass directions of a tile in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row), [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
#[derive(Clone, Copy, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum HexRowDirection {
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

/// Compass directions of a tile in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
#[derive(Clone, Copy, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum HexColDirection {
    North,
    NorthWest,
    SouthWest,
    South,
    SouthEast,
    NorthEast,
}

impl From<HexDirection> for HexRowDirection {
    fn from(direction: HexDirection) -> Self {
        use HexDirection::*;
        use HexRowDirection::*;
        match direction {
            Zero => East,
            One => NorthEast,
            Two => NorthWest,
            Three => West,
            Four => SouthWest,
            Five => SouthEast,
        }
    }
}

impl From<HexRowDirection> for HexDirection {
    fn from(direction: HexRowDirection) -> Self {
        (direction as usize).into()
    }
}

impl From<HexDirection> for HexColDirection {
    fn from(direction: HexDirection) -> Self {
        use HexColDirection::*;
        use HexDirection::*;
        match direction {
            Zero => North,
            One => NorthWest,
            Two => SouthWest,
            Three => South,
            Four => SouthEast,
            Five => NorthEast,
        }
    }
}

impl From<HexColDirection> for HexDirection {
    fn from(direction: HexColDirection) -> Self {
        (direction as usize).into()
    }
}

impl HexRowDirection {
    pub fn offset(&self, tile_pos: &TilePos, coord_sys: HexCoordSystem) -> TilePos {
        AxialPos::from_tile_pos_given_coord_system(tile_pos, coord_sys)
            .offset_compass_row(*self)
            .as_tile_pos_given_coord_system(coord_sys)
    }
}

impl HexColDirection {
    pub fn offset(&self, tile_pos: &TilePos, coord_sys: HexCoordSystem) -> TilePos {
        AxialPos::from_tile_pos_given_coord_system(tile_pos, coord_sys)
            .offset_compass_col(*self)
            .as_tile_pos_given_coord_system(coord_sys)
    }
}
