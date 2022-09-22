use crate::helpers::iso_grid::staggered::StaggeredPos;
use crate::prelude::NeighborDirection;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::{Mat2, Vec2};
use std::ops::{Add, Mul, Sub};

/// Position for tiles arranged in [`Diamond`](crate::map::IsoCoordSystem::Diamond) isometric
/// coordinate system.
///
/// It is a vector-like. In other words: it makes sense to add and subtract
/// two `DiamondPos`, and it makes sense to multiply a `DiamondPos` by
/// an [`i32`](i32) scalar.
///
/// Constants [`UNIT_X`](UNIT_X) and [`UNIT_Y`](`UNIT_Y`) correspond (respectively) with
/// [`North`](NeighborDirection::North) and [`West`](NeighborDirection::West). Since
/// `DiamondPos` is a vector-like, other directions can be obtained by adding/subtracting
/// combinations of [`UNIT_X`](UNIT_X) and [`UNIT_Y`](`UNIT_Y`).
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

/// The `(+1, 0)` constant.
pub const UNIT_X: DiamondPos = DiamondPos { x: 1, y: 0 };

/// The `(0, +1)` constant.
pub const UNIT_Y: DiamondPos = DiamondPos { x: 0, y: -1 };

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
// pub const INV_DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(0.5, 0.5), Vec2::new(-1.0, 0.5));
pub const INV_DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0));

impl From<&TilePos> for DiamondPos {
    fn from(tile_pos: &TilePos) -> Self {
        Self {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }
}

impl From<&StaggeredPos> for DiamondPos {
    fn from(staggered_pos: &StaggeredPos) -> Self {
        let StaggeredPos { x, y } = *staggered_pos;
        DiamondPos { x, y: y + x }
    }
}

impl DiamondPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let unscaled_pos = DIAMOND_BASIS * Vec2::new(self.x as f32, self.y as f32);
        Vec2::new(grid_size.x * unscaled_pos.x, grid_size.y * unscaled_pos.y)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> DiamondPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let Vec2 { x, y } = INV_DIAMOND_BASIS * normalized_world_pos;
        DiamondPos {
            x: (x + 0.5).floor() as i32,
            y: (y + 0.5).floor() as i32,
        }
    }

    /// Get the position of the neighbor in the specified direction.
    pub fn neighbor(&self, direction: NeighborDirection) -> DiamondPos {
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
