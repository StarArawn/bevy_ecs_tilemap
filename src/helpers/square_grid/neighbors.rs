use crate::helpers::square_grid::staggered::StaggeredPos;
use crate::helpers::square_grid::SquarePos;
use crate::map::TilemapSize;
use crate::prelude::{TilePos, TileStorage};
use bevy::prelude::Entity;
use std::ops::{Add, Sub};

/// The eight directions a neighbor a neighbor may lie, in a rectangular grid.
///
/// Note that isometric grids are also (currently) rectangular grids. In particular, there is no
/// difference between the grid system for square and diamond-isometric grids.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RectangularDirection {
    East,
    NorthEast,
    North,
    NorthWest,
    West,
    SouthWest,
    South,
    SouthEast,
}

/// Array of [`SquareDirection`] variants.
pub const SQUARE_DIRECTIONS: [RectangularDirection; 8] = [
    RectangularDirection::East,
    RectangularDirection::NorthEast,
    RectangularDirection::North,
    RectangularDirection::NorthWest,
    RectangularDirection::West,
    RectangularDirection::SouthWest,
    RectangularDirection::South,
    RectangularDirection::SouthEast,
];

/// Array of cardinal [`SquareDirection]s (N, W, S, E).
pub const CARDINAL_SQUARE_DIRECTIONS: [RectangularDirection; 4] = [
    RectangularDirection::North,
    RectangularDirection::West,
    RectangularDirection::South,
    RectangularDirection::East,
];

/// Offsets of tiles that lie in each [`SquareDirection`].
pub const SQUARE_OFFSETS: [SquarePos; 8] = [
    SquarePos { x: 1, y: 0 },
    SquarePos { x: 1, y: 1 },
    SquarePos { x: 0, y: 1 },
    SquarePos { x: -1, y: 1 },
    SquarePos { x: -1, y: 0 },
    SquarePos { x: -1, y: -1 },
    SquarePos { x: 0, y: -1 },
    SquarePos { x: 1, y: -1 },
];

impl From<RectangularDirection> for SquarePos {
    fn from(direction: RectangularDirection) -> Self {
        SQUARE_OFFSETS[direction as usize]
    }
}

impl From<&RectangularDirection> for SquarePos {
    fn from(direction: &RectangularDirection) -> Self {
        SquarePos::from(*direction)
    }
}

impl From<usize> for RectangularDirection {
    fn from(choice: usize) -> Self {
        let ix = choice % 8;
        SQUARE_DIRECTIONS[ix]
    }
}

impl From<isize> for RectangularDirection {
    fn from(choice: isize) -> Self {
        // The Euclidean remainder is always positive, so it is safe to convert to usize;
        let ix = choice.rem_euclid(8) as usize;
        SQUARE_DIRECTIONS[ix]
    }
}

impl From<u32> for RectangularDirection {
    fn from(choice: u32) -> Self {
        (choice as usize).into()
    }
}

impl From<i32> for RectangularDirection {
    fn from(choice: i32) -> Self {
        (choice as isize).into()
    }
}

impl Add<usize> for RectangularDirection {
    type Output = RectangularDirection;

    fn add(self, rhs: usize) -> Self::Output {
        ((self as usize) + rhs).into()
    }
}

impl Add<u32> for RectangularDirection {
    type Output = RectangularDirection;

    fn add(self, rhs: u32) -> Self::Output {
        ((self as usize) + rhs as usize).into()
    }
}

impl Add<isize> for RectangularDirection {
    type Output = RectangularDirection;

    fn add(self, rhs: isize) -> Self::Output {
        ((self as isize) + rhs).into()
    }
}

impl Add<i32> for RectangularDirection {
    type Output = RectangularDirection;

    fn add(self, rhs: i32) -> Self::Output {
        ((self as i32) + rhs).into()
    }
}

impl Sub<usize> for RectangularDirection {
    type Output = RectangularDirection;

    fn sub(self, rhs: usize) -> Self::Output {
        ((self as usize) - rhs).into()
    }
}

impl Sub<u32> for RectangularDirection {
    type Output = RectangularDirection;

    fn sub(self, rhs: u32) -> Self::Output {
        ((self as usize) - rhs as usize).into()
    }
}

impl Sub<isize> for RectangularDirection {
    type Output = RectangularDirection;

    fn sub(self, rhs: isize) -> Self::Output {
        ((self as isize) - rhs).into()
    }
}

impl Sub<i32> for RectangularDirection {
    type Output = RectangularDirection;

    fn sub(self, rhs: i32) -> Self::Output {
        ((self as i32) - rhs).into()
    }
}

/// Stores some data `T` associated with each neighboring grid cell, if present.
#[derive(Clone, Copy, Debug, Default)]
pub struct Neighbors<T> {
    east: Option<T>,
    north_east: Option<T>,
    north: Option<T>,
    north_west: Option<T>,
    west: Option<T>,
    south_west: Option<T>,
    south: Option<T>,
    south_east: Option<T>,
}

impl<T> Neighbors<T> {
    /// Get an item that lies in a particular square direction.
    ///
    /// Will be `None` if no such items exists.
    pub fn get(&self, direction: RectangularDirection) -> Option<&T> {
        use RectangularDirection::*;
        match direction {
            East => self.east.as_ref(),
            NorthEast => self.north_east.as_ref(),
            North => self.north.as_ref(),
            NorthWest => self.north_west.as_ref(),
            West => self.west.as_ref(),
            SouthWest => self.south_west.as_ref(),
            South => self.south.as_ref(),
            SouthEast => self.south_east.as_ref(),
        }
    }

    /// Get a mutable reference to an item that lies in a particular square direction.
    ///
    /// Will be `None` if no such items exists.
    pub fn get_inner_mut(&mut self, direction: RectangularDirection) -> Option<&mut T> {
        use RectangularDirection::*;
        match direction {
            East => self.east.as_mut(),
            NorthEast => self.north_east.as_mut(),
            North => self.north.as_mut(),
            NorthWest => self.north_west.as_mut(),
            West => self.west.as_mut(),
            SouthWest => self.south_west.as_mut(),
            South => self.south.as_mut(),
            SouthEast => self.south_east.as_mut(),
        }
    }

    /// Get a mutable reference to the optional item that lies in a particular square direction.
    ///
    /// Will be `None` if no such items exists.
    pub fn get_mut(&mut self, direction: RectangularDirection) -> &mut Option<T> {
        use RectangularDirection::*;
        match direction {
            East => &mut self.east,
            NorthEast => &mut self.north_east,
            North => &mut self.north,
            NorthWest => &mut self.north_west,
            West => &mut self.west,
            SouthWest => &mut self.south_west,
            South => &mut self.south,
            SouthEast => &mut self.south_east,
        }
    }

    /// Set item that lies in a particular square direction.
    ///
    /// This does an [`Option::replace`](Option::replace) under the hood.
    pub fn set(&mut self, direction: RectangularDirection, data: T) {
        self.get_mut(direction).replace(data);
    }

    /// Iterate over neighbors, in the order specified by [`SQUARE_DIRECTIONS`].
    ///
    /// If a neighbor is `None`, this iterator will skip it.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        SQUARE_DIRECTIONS
            .into_iter()
            .filter_map(|direction| self.get(direction))
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by value.
    pub fn and_then<U, F>(self, f: F) -> Neighbors<U>
    where
        F: Fn(T) -> Option<U>,
    {
        Neighbors {
            east: self.east.and_then(&f),
            north_east: self.north_east.and_then(&f),
            north: self.north.and_then(&f),
            north_west: self.north_west.and_then(&f),
            west: self.west.and_then(&f),
            south_west: self.south_west.and_then(&f),
            south: self.south.and_then(&f),
            south_east: self.south_east.and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn and_then_ref<'a, U, F>(&'a self, f: F) -> Neighbors<U>
    where
        F: Fn(&'a T) -> Option<U>,
    {
        Neighbors {
            east: self.east.as_ref().and_then(&f),
            north_east: self.north_east.as_ref().and_then(&f),
            north: self.north.as_ref().and_then(&f),
            north_west: self.north_west.as_ref().and_then(&f),
            west: self.west.as_ref().and_then(&f),
            south_west: self.south_west.as_ref().and_then(&f),
            south: self.south.as_ref().and_then(&f),
            south_east: self.south_east.as_ref().and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with a [`map`](std::option::Option::map) to each
    /// neighbor element, where `f` takes `T` by reference.
    pub fn map_ref<'a, U, F>(&'a self, f: F) -> Neighbors<U>
    where
        F: Fn(&'a T) -> U,
    {
        Neighbors {
            east: self.east.as_ref().map(&f),
            north_east: self.north_east.as_ref().map(&f),
            north: self.north.as_ref().map(&f),
            north_west: self.north_west.as_ref().map(&f),
            west: self.west.as_ref().map(&f),
            south_west: self.south_west.as_ref().map(&f),
            south: self.south.as_ref().map(&f),
            south_east: self.south_east.as_ref().map(&f),
        }
    }

    /// Generates `SquareNeighbors<T>` from a closure that takes a hex direction and outputs
    /// `Option<T>`.
    pub fn from_directional_closure<F>(f: F) -> Neighbors<T>
    where
        F: Fn(RectangularDirection) -> Option<T>,
    {
        use RectangularDirection::*;
        Neighbors {
            east: f(East),
            north_east: f(NorthEast),
            north: f(North),
            north_west: f(NorthWest),
            west: f(West),
            south_west: f(SouthWest),
            south: f(South),
            south_east: f(SouthEast),
        }
    }
}

impl RectangularDirection {
    /// Is this direction a cardinal direction (i.e. North, South, East, West)?
    pub fn is_cardinal(&self) -> bool {
        use RectangularDirection::*;
        matches!(self, East | North | West | South)
    }

    /// Is this direction a diagonal direction (i.e. NorthEast, NorthWest, SouthWest, SouthEast)?
    pub fn is_diagonal(&self) -> bool {
        !self.is_cardinal()
    }
}

impl Neighbors<TilePos> {
    /// Returns neighboring tile positions for a tile position in a square grid (which includes
    /// isometric diamond and isometric staggered).
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    pub fn get_square_neighboring_positions(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
        include_diagonals: bool,
    ) -> Neighbors<TilePos> {
        let square_pos = SquarePos::from(tile_pos);
        if include_diagonals {
            let f = |direction: RectangularDirection| {
                square_pos.offset(&direction).as_tile_pos(map_size)
            };

            Neighbors::from_directional_closure(f)
        } else {
            let f = |direction: RectangularDirection| {
                if direction.is_cardinal() {
                    square_pos.offset(&direction).as_tile_pos(map_size)
                } else {
                    None
                }
            };

            Neighbors::from_directional_closure(f)
        }
    }

    /// Returns neighboring tile positions for a tile position in a staggered square grid, which is
    /// the case for the isometric staggered grid.
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    pub fn get_staggered_neighboring_positions(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
        include_diagonals: bool,
    ) -> Neighbors<TilePos> {
        let staggered_pos = StaggeredPos::from(tile_pos);
        if include_diagonals {
            let f = |direction: RectangularDirection| {
                staggered_pos.offset(&direction).as_tile_pos(map_size)
            };

            Neighbors::from_directional_closure(f)
        } else {
            let f = |direction: RectangularDirection| {
                if direction.is_cardinal() {
                    staggered_pos.offset(&direction).as_tile_pos(map_size)
                } else {
                    None
                }
            };

            Neighbors::from_directional_closure(f)
        }
    }

    /// Returns the entities associated with each tile position.
    pub fn entities(&self, tile_storage: &TileStorage) -> Neighbors<Entity> {
        let f = |tile_pos| tile_storage.get(tile_pos);
        self.and_then_ref(f)
    }
}
