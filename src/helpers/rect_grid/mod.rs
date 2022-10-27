pub mod diamond;
pub mod neighbors;
pub mod staggered;

use crate::helpers::rect_grid::diamond::DiamondPos;
use crate::helpers::rect_grid::neighbors::{RectangularDirection, SQUARE_OFFSETS};
use crate::helpers::rect_grid::staggered::StaggeredPos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::Vec2;
use std::ops::{Add, Mul, Sub};

/// Position for tiles arranged in [`Square`](crate::map::IsoCoordSystem::Square) coordinate system.
///
/// It is a vector-like. In other words: it makes sense to add and subtract
/// two `SquarePos`, and it makes sense to multiply a `SquarePos` by
/// an [`i32`](i32) scalar.
///
/// A `SquarePos` can be mapped to world space, and a world space position can be mapped to
/// the tile with `SquarePos` containing said world space position.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SquarePos {
    pub x: i32,
    pub y: i32,
}

impl Add<SquarePos> for SquarePos {
    type Output = SquarePos;

    fn add(self, rhs: SquarePos) -> Self::Output {
        SquarePos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<SquarePos> for SquarePos {
    type Output = SquarePos;

    fn sub(self, rhs: SquarePos) -> Self::Output {
        SquarePos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<SquarePos> for i32 {
    type Output = SquarePos;

    fn mul(self, rhs: SquarePos) -> Self::Output {
        SquarePos {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl From<&TilePos> for SquarePos {
    fn from(tile_pos: &TilePos) -> Self {
        Self {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }
}

impl From<&DiamondPos> for SquarePos {
    fn from(diamond_pos: &DiamondPos) -> Self {
        let DiamondPos { x, y } = *diamond_pos;
        SquarePos { x, y }
    }
}

impl From<DiamondPos> for SquarePos {
    fn from(diamond_pos: DiamondPos) -> Self {
        let DiamondPos { x, y } = diamond_pos;
        SquarePos { x, y }
    }
}

impl From<&StaggeredPos> for SquarePos {
    fn from(staggered_pos: &StaggeredPos) -> Self {
        let StaggeredPos { x, y } = *staggered_pos;
        SquarePos { x, y: y + x }
    }
}

impl From<StaggeredPos> for SquarePos {
    fn from(staggered_pos: StaggeredPos) -> Self {
        let StaggeredPos { x, y } = staggered_pos;
        SquarePos { x, y: y + x }
    }
}

impl SquarePos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let unscaled_pos = Vec2::new(self.x as f32, self.y as f32);
        Vec2::new(grid_size.x * unscaled_pos.x, grid_size.y * unscaled_pos.y)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> SquarePos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let Vec2 { x, y } = normalized_world_pos;
        SquarePos {
            x: (x + 0.5).floor() as i32,
            y: (y + 0.5).floor() as i32,
        }
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `self.x` or `self.y` is negative, or lies outside of the
    /// bounds of `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.x, self.y, map_size)
    }

    /// Calculate offset in the given direction.
    pub fn offset(&self, direction: &RectangularDirection) -> SquarePos {
        *self + SQUARE_OFFSETS[*direction as usize]
    }
}
