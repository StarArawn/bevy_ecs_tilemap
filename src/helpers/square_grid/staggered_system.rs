use crate::helpers::square_grid::diamond_system::DiamondPos;
use crate::helpers::square_grid::neighbors::{SquareDirection, SQUARE_OFFSETS};
use crate::helpers::square_grid::SquarePos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::Vec2;
use std::ops::{Add, Mul, Sub};

/// Position for tiles arranged in [`Staggered`](crate::map::IsoCoordSystem::Diamond) isometric
/// coordinate system.
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

impl From<DiamondPos> for StaggeredPos {
    fn from(diamond_pos: DiamondPos) -> Self {
        let DiamondPos { x, y } = diamond_pos;
        StaggeredPos { x, y: y - x }
    }
}

impl From<&DiamondPos> for StaggeredPos {
    fn from(diamond_pos: &DiamondPos) -> Self {
        StaggeredPos::from(*diamond_pos)
    }
}

impl From<SquarePos> for StaggeredPos {
    fn from(square_pos: SquarePos) -> Self {
        let SquarePos { x, y } = square_pos;
        StaggeredPos { x, y: y - x }
    }
}

impl From<&SquarePos> for StaggeredPos {
    fn from(square_pos: &SquarePos) -> Self {
        StaggeredPos::from(*square_pos)
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

impl StaggeredPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        DiamondPos::from(self).center_in_world(grid_size)
    }

    /// Returns the offset to the corner of a tile in the specified `corner_direction`,
    /// in world space
    pub fn corner_offset_in_world(
        corner_direction: SquareDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        DiamondPos::corner_offset_in_world(corner_direction, grid_size)
    }

    /// Returns the coordinate of the corner of a tile in the specified `corner_direction`,
    /// in world space
    pub fn corner_in_world(
        &self,
        corner_direction: SquareDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let diamond_pos = DiamondPos::from(self);

        let center = Vec2::new(diamond_pos.x as f32, diamond_pos.y as f32);

        let corner_offset = DiamondPos::from(SquarePos::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.x as f32, corner_offset.y as f32);

        DiamondPos::project(center + corner_pos, grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> StaggeredPos {
        DiamondPos::from_world_pos(world_pos, grid_size).into()
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `self.x` or `self.y` is negative, or lies outside of the
    /// bounds of `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.x, self.y, map_size)
    }

    /// Calculate offset in the given direction.
    pub fn offset(&self, direction: &SquareDirection) -> StaggeredPos {
        StaggeredPos::from(SquarePos::from(self) + SQUARE_OFFSETS[*direction as usize])
    }
}
