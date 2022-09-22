use crate::helpers::hex_grid::axial::AxialPos;
use std::ops::{Add, Sub};

/// Compass neighbors of a tile in hexagonal row-oriented coordinate systems
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

pub const HEX_ROW_DIRECTIONS: [HexRowDirection; 6] = [
    HexRowDirection::East,
    HexRowDirection::NorthEast,
    HexRowDirection::NorthWest,
    HexRowDirection::West,
    HexRowDirection::SouthWest,
    HexRowDirection::SouthEast,
];

impl From<usize> for HexRowDirection {
    fn from(choice: usize) -> Self {
        let ix = choice % 6;
        HEX_ROW_DIRECTIONS[ix]
    }
}

impl From<isize> for HexRowDirection {
    fn from(choice: isize) -> Self {
        // The Euclidean remainder is always positive, so it is safe to convert to usize;
        let ix = choice.rem_euclid(6) as usize;
        HEX_ROW_DIRECTIONS[ix]
    }
}

impl From<u32> for HexRowDirection {
    fn from(choice: u32) -> Self {
        (choice as usize).into()
    }
}

impl From<i32> for HexRowDirection {
    fn from(choice: i32) -> Self {
        (choice as isize).into()
    }
}

impl Add<usize> for HexRowDirection {
    type Output = HexRowDirection;

    fn add(self, rhs: usize) -> Self::Output {
        ((self as usize) + rhs).into()
    }
}

impl Add<u32> for HexRowDirection {
    type Output = HexRowDirection;

    fn add(self, rhs: u32) -> Self::Output {
        ((self as usize) + rhs as usize).into()
    }
}

impl Add<isize> for HexRowDirection {
    type Output = HexRowDirection;

    fn add(self, rhs: isize) -> Self::Output {
        ((self as isize) + rhs).into()
    }
}

impl Add<i32> for HexRowDirection {
    type Output = HexRowDirection;

    fn add(self, rhs: i32) -> Self::Output {
        ((self as i32) + rhs).into()
    }
}

impl Sub<usize> for HexRowDirection {
    type Output = HexRowDirection;

    fn sub(self, rhs: usize) -> Self::Output {
        ((self as usize) - rhs).into()
    }
}

impl Sub<u32> for HexRowDirection {
    type Output = HexRowDirection;

    fn sub(self, rhs: u32) -> Self::Output {
        ((self as usize) - rhs as usize).into()
    }
}

impl Sub<isize> for HexRowDirection {
    type Output = HexRowDirection;

    fn sub(self, rhs: isize) -> Self::Output {
        ((self as isize) - rhs).into()
    }
}

impl Sub<i32> for HexRowDirection {
    type Output = HexRowDirection;

    fn sub(self, rhs: i32) -> Self::Output {
        ((self as i32) - rhs).into()
    }
}

/// Vectors associated with compass directions of neighboring tiles in hexagonal row-oriented
/// coordinate systems ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_DIRECTIONS: [AxialPos; 6] = [
    AXIAL_ROW_EAST,
    AXIAL_ROW_NORTH_EAST,
    AXIAL_ROW_NORTH_WEST,
    AXIAL_ROW_WEST,
    AXIAL_ROW_SOUTH_WEST,
    AXIAL_ROW_SOUTH_EAST,
];

/// Vector associated with East in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_EAST: AxialPos = AxialPos { q: 1, r: 0 };

/// Vector associated with East in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_NORTH_EAST: AxialPos = AxialPos { q: 0, r: 1 };

/// Vector associated with North-West in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_NORTH_WEST: AxialPos = AxialPos { q: -1, r: 1 };

/// Vector associated with West in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_WEST: AxialPos = -1 * AXIAL_ROW_EAST;

/// Vector associated with South-West in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_SOUTH_WEST: AxialPos = -1 * AXIAL_ROW_NORTH_EAST;

/// Vector associated with South-East in hexagonal row-oriented coordinate systems
/// ([Row](crate::map::HexCoordSystem::Row),
/// [RowEven](crate::map::HexCoordSystem::RowEven), and
/// [RowOdd](crate::map::HexCoordSystem::RowOdd)).
pub const AXIAL_ROW_SOUTH_EAST: AxialPos = -1 * AXIAL_ROW_NORTH_WEST;

/// Compass neighbors of a tile in hexagonal column-oriented coordinate systems
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

pub const HEX_COL_DIRECTIONS: [HexColDirection; 6] = [
    HexColDirection::North,
    HexColDirection::NorthWest,
    HexColDirection::SouthWest,
    HexColDirection::South,
    HexColDirection::SouthEast,
    HexColDirection::NorthEast,
];

impl From<usize> for HexColDirection {
    fn from(choice: usize) -> Self {
        let ix = choice % 6;
        HEX_COL_DIRECTIONS[ix]
    }
}

impl From<isize> for HexColDirection {
    fn from(choice: isize) -> Self {
        // The Euclidean remainder is always positive, so it is safe to convert to usize;
        let ix = choice.rem_euclid(6) as usize;
        HEX_COL_DIRECTIONS[ix]
    }
}

impl From<u32> for HexColDirection {
    fn from(choice: u32) -> Self {
        (choice as usize).into()
    }
}

impl From<i32> for HexColDirection {
    fn from(choice: i32) -> Self {
        (choice as isize).into()
    }
}

impl Add<usize> for HexColDirection {
    type Output = HexColDirection;

    fn add(self, rhs: usize) -> Self::Output {
        ((self as usize) + rhs).into()
    }
}

impl Add<u32> for HexColDirection {
    type Output = HexColDirection;

    fn add(self, rhs: u32) -> Self::Output {
        ((self as usize) + rhs as usize).into()
    }
}

impl Add<isize> for HexColDirection {
    type Output = HexColDirection;

    fn add(self, rhs: isize) -> Self::Output {
        ((self as isize) + rhs).into()
    }
}

impl Add<i32> for HexColDirection {
    type Output = HexColDirection;

    fn add(self, rhs: i32) -> Self::Output {
        ((self as i32) + rhs).into()
    }
}

impl Sub<usize> for HexColDirection {
    type Output = HexColDirection;

    fn sub(self, rhs: usize) -> Self::Output {
        ((self as usize) - rhs).into()
    }
}

impl Sub<u32> for HexColDirection {
    type Output = HexColDirection;

    fn sub(self, rhs: u32) -> Self::Output {
        ((self as usize) - rhs as usize).into()
    }
}

impl Sub<isize> for HexColDirection {
    type Output = HexColDirection;

    fn sub(self, rhs: isize) -> Self::Output {
        ((self as isize) - rhs).into()
    }
}

impl Sub<i32> for HexColDirection {
    type Output = HexColDirection;

    fn sub(self, rhs: i32) -> Self::Output {
        ((self as i32) - rhs).into()
    }
}

/// Vectors associated with compass directions of neighboring tiles in hexagonal column-oriented
/// coordinate systems ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_DIRECTIONS: [AxialPos; 6] = [
    AXIAL_COL_NORTH,
    AXIAL_COL_NORTH_WEST,
    AXIAL_COL_SOUTH_WEST,
    AXIAL_COL_SOUTH,
    AXIAL_COL_SOUTH_EAST,
    AXIAL_COL_NORTH_EAST,
];

/// Vector associated with North-East in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_NORTH_EAST: AxialPos = AxialPos { q: 1, r: 0 };

/// Vector associated with North in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_NORTH: AxialPos = AxialPos { q: 0, r: 1 };

/// Vector associated with West in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_NORTH_WEST: AxialPos = AxialPos { q: -1, r: 1 };

/// Vector associated with South-West in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_SOUTH_WEST: AxialPos = -1 * AXIAL_COL_NORTH_EAST;

/// Vector associated with South in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_SOUTH: AxialPos = -1 * AXIAL_COL_NORTH;

/// Vector associated with South-East in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
pub const AXIAL_COL_SOUTH_EAST: AxialPos = -1 * AXIAL_COL_NORTH_WEST;
