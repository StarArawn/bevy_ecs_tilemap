//! Code for the isometric diamond coordinate system.

use crate::helpers::square_grid::neighbors::{SquareDirection, SQUARE_OFFSETS};
use crate::helpers::square_grid::staggered::StaggeredPos;
use crate::helpers::square_grid::SquarePos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::{Mat2, Vec2};
use std::ops::{Add, Mul, Sub};

/// Position for tiles arranged in [`Diamond`](crate::map::IsoCoordSystem::Diamond) isometric
/// coordinate system.
///
/// It is vector-like. In other words: it makes sense to add and subtract
/// two `DiamondPos`, and it makes sense to multiply a `DiamondPos` by
/// an [`i32`] scalar.
///
/// A `DiamondPos` can be mapped to world space, and a world space position can be mapped to
/// the tile with `DiamondPos` containing said world space position.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct DiamondPos {
    pub x: i32,
    pub y: i32,
}

impl Add<DiamondPos> for DiamondPos {
    type Output = DiamondPos;

    fn add(self, rhs: DiamondPos) -> Self::Output {
        DiamondPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<DiamondPos> for DiamondPos {
    type Output = DiamondPos;

    fn sub(self, rhs: DiamondPos) -> Self::Output {
        DiamondPos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<DiamondPos> for i32 {
    type Output = DiamondPos;

    fn mul(self, rhs: DiamondPos) -> Self::Output {
        DiamondPos {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

/// The matrix mapping from tile positions in the diamond isometric grid system to world space.
///
/// It can be derived in multiple ways; one method is by considering it as a product three matrices:
/// `S * A * B`, where `A` and `B` are
/// [shear matrices](https://en.wikipedia.org/wiki/Shear_matrix), while `S` is a scaling matrix:
/// * `A = Mat2::from_cols(Vec2::new(1.0, -0.5), Vec2::new(0.0, 1.0))`,
/// * `B = Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0))`,
/// * `S = Mat2::from_cols(Vec2::new(0.5, 0.0), Vec2::new(0.0, 1.0))`.
///
/// One can derive `A` and `B` by solving for shear factors which produce a sheared
/// unit square such that the diagonals of the sheared square are perpendicular to each other. The
/// scaling matrix `S` ensures that the the diagonals of the sheared square both have length `1`.
pub const DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(0.5, -0.5), Vec2::new(0.5, 0.5));

/// The inverse of [`DIAMOND_BASIS`].
pub const INV_DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0));

impl From<TilePos> for DiamondPos {
    #[inline]
    fn from(tile_pos: TilePos) -> Self {
        let TilePos { x, y } = tile_pos;
        DiamondPos {
            x: x as i32,
            y: y as i32,
        }
    }
}

impl From<&TilePos> for DiamondPos {
    #[inline]
    fn from(tile_pos: &TilePos) -> Self {
        DiamondPos::from(*tile_pos)
    }
}

impl From<StaggeredPos> for DiamondPos {
    #[inline]
    fn from(staggered_pos: StaggeredPos) -> Self {
        let StaggeredPos { x, y } = staggered_pos;
        DiamondPos { x, y: y + x }
    }
}

impl From<&StaggeredPos> for DiamondPos {
    #[inline]
    fn from(staggered_pos: &StaggeredPos) -> Self {
        DiamondPos::from(*staggered_pos)
    }
}

impl From<SquarePos> for DiamondPos {
    #[inline]
    fn from(square_pos: SquarePos) -> Self {
        let SquarePos { x, y } = square_pos;
        DiamondPos { x, y }
    }
}

impl From<&SquarePos> for DiamondPos {
    #[inline]
    fn from(square_pos: &SquarePos) -> Self {
        DiamondPos::from(*square_pos)
    }
}

impl DiamondPos {
    /// Project a vector representing a fractional tile position (i.e. the components can be `f32`)
    /// into world space.
    ///
    /// This is a helper function for
    /// [`center_in_world`](Self::center_in_world),
    /// [`corner_offset_in_world`](Self::corner_offset_in_world) and
    /// [`corner_in_world`](Self::corner_in_world).
    #[inline]
    pub fn project(pos: Vec2, grid_size: &TilemapGridSize) -> Vec2 {
        let unscaled_pos = DIAMOND_BASIS * pos;
        Vec2::new(grid_size.x * unscaled_pos.x, grid_size.y * unscaled_pos.y)
    }

    /// Returns the position of this tile's center, in world space.
    #[inline]
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        Self::project(Vec2::new(self.x as f32, self.y as f32), grid_size)
    }

    /// Returns the offset to the corner of a tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_offset_in_world(
        corner_direction: SquareDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let corner_offset = DiamondPos::from(SquarePos::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.x as f32, corner_offset.y as f32);
        Self::project(corner_pos, grid_size)
    }

    /// Returns the coordinate of the corner of a tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_in_world(
        &self,
        corner_direction: SquareDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let center = Vec2::new(self.x as f32, self.y as f32);

        let corner_offset = DiamondPos::from(SquarePos::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.x as f32, corner_offset.y as f32);

        Self::project(center + corner_pos, grid_size)
    }

    /// Returns the tile containing the given world position.
    #[inline]
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> DiamondPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let Vec2 { x, y } = INV_DIAMOND_BASIS * normalized_world_pos;
        DiamondPos {
            x: (x + 0.5).floor() as i32,
            y: (y + 0.5).floor() as i32,
        }
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `self.x` or `self.y` is negative, or lies outside of the
    /// bounds of `map_size`.
    #[inline]
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.x, self.y, map_size)
    }

    /// Calculate offset in the given direction.
    #[inline]
    pub fn offset(&self, direction: &SquareDirection) -> DiamondPos {
        DiamondPos::from(SquarePos::from(self) + SQUARE_OFFSETS[*direction as usize])
    }
}

impl TilePos {
    /// Get the neighbor lying in the specified direction from this position, if it  fits on the map
    /// and assuming that this is a map using the isometric diamond coordinate system.
    #[inline]
    pub fn diamond_offset(
        &self,
        direction: &SquareDirection,
        map_size: &TilemapSize,
    ) -> Option<TilePos> {
        DiamondPos::from(self)
            .offset(direction)
            .as_tile_pos(map_size)
    }
}
