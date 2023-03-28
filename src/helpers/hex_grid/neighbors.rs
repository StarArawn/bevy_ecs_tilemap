use crate::helpers::hex_grid::axial::AxialPos;
use crate::helpers::hex_grid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::map::{HexCoordSystem, TilemapSize};
use crate::prelude::TileStorage;
use crate::TilePos;
use bevy::prelude::Entity;
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
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
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
    North,
    NorthWest,
    SouthWest,
    South,
    SouthEast,
    NorthEast,
}

/// Compass directions of a tile in hexagonal column-oriented coordinate systems
/// ([Column](crate::map::HexCoordSystem::Column),
/// [ColumnEven](crate::map::HexCoordSystem::ColumnEven), and
/// [ColumnOdd](crate::map::HexCoordSystem::ColumnOdd)).
#[derive(Clone, Copy, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum HexColDirection {
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

impl From<HexDirection> for HexRowDirection {
    #[inline]
    fn from(direction: HexDirection) -> Self {
        use HexDirection::*;
        use HexRowDirection::*;
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

impl From<HexDirection> for HexColDirection {
    #[inline]
    fn from(direction: HexDirection) -> Self {
        use HexColDirection::*;
        use HexDirection::*;
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
    #[inline]
    fn from(direction: HexRowDirection) -> Self {
        (direction as usize).into()
    }
}

impl From<HexColDirection> for HexDirection {
    #[inline]
    fn from(direction: HexColDirection) -> Self {
        (direction as usize).into()
    }
}

impl HexRowDirection {
    #[inline]
    pub fn offset(&self, tile_pos: &TilePos, coord_sys: HexCoordSystem) -> TilePos {
        AxialPos::from_tile_pos_given_coord_system(tile_pos, coord_sys)
            .offset_compass_row(*self)
            .as_tile_pos_given_coord_system(coord_sys)
    }
}

impl HexColDirection {
    #[inline]
    pub fn offset(&self, tile_pos: &TilePos, coord_sys: HexCoordSystem) -> TilePos {
        AxialPos::from_tile_pos_given_coord_system(tile_pos, coord_sys)
            .offset_compass_col(*self)
            .as_tile_pos_given_coord_system(coord_sys)
    }
}

/// Stores some data `T` associated with each neighboring hex cell, if present.
#[derive(Debug, Default)]
pub struct HexNeighbors<T> {
    pub zero: Option<T>,
    pub one: Option<T>,
    pub two: Option<T>,
    pub three: Option<T>,
    pub four: Option<T>,
    pub five: Option<T>,
}

impl<T> HexNeighbors<T> {
    /// Get an item that lies in a particular hex direction, specified by a [`HexDirection`].
    ///
    /// Will be `None` if no such items exists.
    #[inline]
    pub fn get(&self, direction: HexDirection) -> Option<&T> {
        use HexDirection::*;
        match direction {
            Zero => self.zero.as_ref(),
            One => self.one.as_ref(),
            Two => self.two.as_ref(),
            Three => self.three.as_ref(),
            Four => self.four.as_ref(),
            Five => self.five.as_ref(),
        }
    }

    /// Get a mutable reference to an item that lies in a particular hex direction.
    ///
    /// Will be `None` if no such items exists.
    #[inline]
    pub fn get_inner_mut(&mut self, direction: HexDirection) -> Option<&mut T> {
        use HexDirection::*;
        match direction {
            Zero => self.zero.as_mut(),
            One => self.one.as_mut(),
            Two => self.two.as_mut(),
            Three => self.three.as_mut(),
            Four => self.four.as_mut(),
            Five => self.five.as_mut(),
        }
    }

    /// Get a mutable reference to the optional item that lies in a particular hex direction.
    ///
    /// Will be `None` if no such items exists.
    #[inline]
    pub fn get_mut(&mut self, direction: HexDirection) -> &mut Option<T> {
        use HexDirection::*;
        match direction {
            Zero => &mut self.zero,
            One => &mut self.one,
            Two => &mut self.two,
            Three => &mut self.three,
            Four => &mut self.four,
            Five => &mut self.five,
        }
    }

    /// Set item that lies in a particular hex direction.
    ///
    /// This does an [`Option::replace`](Option::replace) under the hood.
    #[inline]
    pub fn set(&mut self, direction: HexDirection, data: T) {
        self.get_mut(direction).replace(data);
    }

    /// Iterate over neighbors, in the order specified by [`HEX_DIRECTIONS`].
    ///
    /// If a neighbor is `None`, this iterator will skip it.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        HEX_DIRECTIONS
            .into_iter()
            .filter_map(|direction| self.get(direction))
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by value.
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> HexNeighbors<U>
    where
        F: Fn(T) -> Option<U>,
    {
        HexNeighbors {
            zero: self.zero.and_then(&f),
            one: self.one.and_then(&f),
            two: self.two.and_then(&f),
            three: self.three.and_then(&f),
            four: self.four.and_then(&f),
            five: self.five.and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with an [`and_then`](std::option::Option::and_then) to each
    /// neighbor element, where `f` takes `T` by reference.
    #[inline]
    pub fn and_then_ref<'a, U, F>(&'a self, f: F) -> HexNeighbors<U>
    where
        F: Fn(&'a T) -> Option<U>,
    {
        HexNeighbors {
            zero: self.zero.as_ref().and_then(&f),
            one: self.one.as_ref().and_then(&f),
            two: self.two.as_ref().and_then(&f),
            three: self.three.as_ref().and_then(&f),
            four: self.four.as_ref().and_then(&f),
            five: self.five.as_ref().and_then(&f),
        }
    }

    /// Applies the supplied closure `f` with a [`map`](std::option::Option::map) to each
    /// neighbor element, where `f` takes `T` by reference.
    #[inline]
    pub fn map_ref<'a, U, F>(&'a self, f: F) -> HexNeighbors<U>
    where
        F: Fn(&'a T) -> U,
    {
        HexNeighbors {
            zero: self.zero.as_ref().map(&f),
            one: self.one.as_ref().map(&f),
            two: self.two.as_ref().map(&f),
            three: self.three.as_ref().map(&f),
            four: self.four.as_ref().map(&f),
            five: self.five.as_ref().map(&f),
        }
    }

    /// Generates `HexNeighbors<T>` from a closure that takes a hex direction and outputs
    /// `Option<T>`.
    #[inline]
    pub fn from_directional_closure<F>(f: F) -> HexNeighbors<T>
    where
        F: Fn(HexDirection) -> Option<T>,
    {
        use HexDirection::*;
        HexNeighbors {
            zero: f(Zero),
            one: f(One),
            two: f(Two),
            three: f(Three),
            four: f(Four),
            five: f(Five),
        }
    }
}

impl HexNeighbors<TilePos> {
    /// Returns neighboring tile positions, given a coordinate system.
    ///
    /// In general, if you know which coordinate system you are using, it will be more efficient to
    /// use one of:
    ///     * [`HexNeighbors::get_neighboring_positions_standard`]
    ///     * [`HexNeighbors::get_neighboring_positions_row_even`]
    ///     * [`HexNeighbors::get_neighboring_positions_row_odd`]
    ///     * [`HexNeighbors::get_neighboring_positions_col_even`]
    ///     * [`HexNeighbors::get_neighboring_positions_col_odd`]
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
        hex_coord_sys: &HexCoordSystem,
    ) -> HexNeighbors<TilePos> {
        match hex_coord_sys {
            HexCoordSystem::RowEven => {
                HexNeighbors::get_neighboring_positions_row_even(tile_pos, map_size)
            }
            HexCoordSystem::RowOdd => {
                HexNeighbors::get_neighboring_positions_row_odd(tile_pos, map_size)
            }
            HexCoordSystem::ColumnEven => {
                HexNeighbors::get_neighboring_positions_col_even(tile_pos, map_size)
            }
            HexCoordSystem::ColumnOdd => {
                HexNeighbors::get_neighboring_positions_col_odd(tile_pos, map_size)
            }
            HexCoordSystem::Row | HexCoordSystem::Column => {
                HexNeighbors::get_neighboring_positions_standard(tile_pos, map_size)
            }
        }
    }

    /// Returns neighboring tile positions. This works for maps using [`HexCoordSystem::Row`] and
    /// [`HexCoordSystem::Column`].
    ///
    /// For maps using [`HexCoordSystem::RowEven`], [`HexCoordSystem::ColumnEven`],
    /// [`HexCoordSystem::RowOdd`], [`HexCoordSystem::RowOdd`], use one of:
    ///     * [`HexNeighbors::get_neighboring_positions_row_even`]
    ///     * [`HexNeighbors::get_neighboring_positions_row_odd`]
    ///     * [`HexNeighbors::get_neighboring_positions_col_even`]
    ///     * [`HexNeighbors::get_neighboring_positions_col_odd`]
    /// (Or, just don't use a map that with a odd/even coordinate system, and prefer to use one
    /// with  [`HexCoordSystem::Row`] or [`HexCoordSystem::Column`]).
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions_standard(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(tile_pos);
        let f = |direction| {
            axial_pos
                .offset(direction)
                .as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Returns neighboring tile positions on a map using [`HexCoordSystem::RowEven`].
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions_row_even(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(RowEvenPos::from(tile_pos));
        let f = |direction| {
            RowEvenPos::from(axial_pos.offset(direction)).as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Returns neighboring tile positions on a map using [`HexCoordSystem::RowOdd`].
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions_row_odd(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(RowOddPos::from(tile_pos));
        let f = |direction| {
            RowOddPos::from(axial_pos.offset(direction)).as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Returns neighboring tile positions on a map using [`HexCoordSystem::ColumnEven`].
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions_col_even(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(ColEvenPos::from(tile_pos));
        let f = |direction| {
            ColEvenPos::from(axial_pos.offset(direction)).as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Returns neighboring tile positions on a map using [`HexCoordSystem::ColumnOdd`].
    ///
    /// A tile position will be `None` for a particular direction, if that neighbor would not lie
    /// on the map.
    #[inline]
    pub fn get_neighboring_positions_col_odd(
        tile_pos: &TilePos,
        map_size: &TilemapSize,
    ) -> HexNeighbors<TilePos> {
        let axial_pos = AxialPos::from(ColOddPos::from(tile_pos));
        let f = |direction| {
            ColOddPos::from(axial_pos.offset(direction)).as_tile_pos_given_map_size(map_size)
        };
        HexNeighbors::from_directional_closure(f)
    }

    /// Returns the entities associated with each tile position.
    #[inline]
    pub fn entities(&self, tile_storage: &TileStorage) -> HexNeighbors<Entity> {
        let f = |tile_pos| tile_storage.get(tile_pos);
        self.and_then_ref(f)
    }
}
