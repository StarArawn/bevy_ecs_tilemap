use crate::helpers::iso_grid::diamond::DiamondPos;
use crate::prelude::NeighborDirection;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::Vec2;
use std::ops::{Add, Mul, Sub};

/// Position for tiles arranged in [`Staggered`](crate::map::IsoCoordSystem::Diamond) isometric
/// coordinate system.
///
/// It is a vector-like. In other words: it makes sense to add and subtract
/// two `StaggeredPos`, and it makes sense to multiply a `StaggeredPos` by
/// an [`i32`](i32) scalar.
///
/// Constants [`UNIT_X`](UNIT_X) and [`UNIT_Y`](`UNIT_Y`) correspond (respectively) with
/// [`North`](NeighborDirection::North) and [`West`](NeighborDirection::West). Since
/// `StaggeredPos` is a vector-like, other directions can be obtained by adding/subtracting
/// combinations of [`UNIT_X`](UNIT_X) and [`UNIT_Y`](`UNIT_Y`).
///
/// A `StaggeredPos` can be mapped to world space, and a world space position can be mapped to
/// the tile with `StaggeredPos` containing said world space position.
///
/// Under the hood, in order to reduce code duplication, a `StaggeredPos` is mapped to
/// [`DiamondPos`] for world space to grid space related calculations.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct StaggeredPos {
    pub x: i32,
    pub y: i32,
}

impl From<&TilePos> for StaggeredPos {
    fn from(tile_pos: &TilePos) -> Self {
        Self {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }
}

impl From<&DiamondPos> for StaggeredPos {
    fn from(diamond_pos: &DiamondPos) -> Self {
        let DiamondPos { x, y } = *diamond_pos;
        StaggeredPos { x, y: y - x }
    }
}

impl From<DiamondPos> for StaggeredPos {
    fn from(diamond_pos: DiamondPos) -> Self {
        let DiamondPos { x, y } = diamond_pos;
        StaggeredPos { x, y: y - x }
    }
}

impl Add<StaggeredPos> for StaggeredPos {
    type Output = StaggeredPos;

    fn add(self, rhs: StaggeredPos) -> Self::Output {
        StaggeredPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<StaggeredPos> for StaggeredPos {
    type Output = StaggeredPos;

    fn sub(self, rhs: StaggeredPos) -> Self::Output {
        StaggeredPos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<StaggeredPos> for i32 {
    type Output = StaggeredPos;

    fn mul(self, rhs: StaggeredPos) -> Self::Output {
        StaggeredPos {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

/// The `(+1, 0)` constant.
pub const UNIT_X: StaggeredPos = StaggeredPos { x: 1, y: 0 };

/// The `(0, +1)` constant.
pub const UNIT_Y: StaggeredPos = StaggeredPos { x: 0, y: -1 };

impl StaggeredPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        DiamondPos::from(self).center_in_world(grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> StaggeredPos {
        DiamondPos::from_world_pos(world_pos, grid_size).into()
    }

    /// Get the position of the neighbor in the specified direction.
    pub fn neighbor(&self, direction: NeighborDirection) -> StaggeredPos {
        match direction {
            NeighborDirection::North => *self + UNIT_X,
            NeighborDirection::NorthWest => *self - UNIT_X + UNIT_Y,
            NeighborDirection::West => *self - UNIT_X,
            NeighborDirection::SouthWest => *self - UNIT_X - UNIT_Y,
            NeighborDirection::South => *self - UNIT_Y,
            NeighborDirection::SouthEast => *self - UNIT_Y + UNIT_X,
            NeighborDirection::East => *self + UNIT_X,
            NeighborDirection::NorthEast => *self + UNIT_X + UNIT_Y,
        }
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `self.x` or `self.y` is negative, or lies outside of the
    /// bounds of `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.x, self.y, map_size)
    }
}
