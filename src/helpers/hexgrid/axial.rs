use crate::helpers::hexgrid::consts::{HALF_SQRT_3, INV_SQRT_3, SQRT_3};
use crate::helpers::hexgrid::cube::{CubePos, FractionalCubePos};
use crate::helpers::hexgrid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::tiles::TilePos;
use crate::TilemapGridSize;
use bevy::math::{Mat2, Vec2};
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct AxialPos {
    pub alpha: i32,
    pub beta: i32,
}

impl From<AxialPos> for CubePos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { alpha, beta } = axial_pos;
        CubePos {
            alpha,
            beta,
            gamma: -(alpha + beta),
        }
    }
}

impl From<CubePos> for AxialPos {
    fn from(cube_pos: CubePos) -> Self {
        let CubePos { alpha, beta, .. } = cube_pos;
        AxialPos { alpha, beta }
    }
}

impl TryFrom<&TilePos> for AxialPos {
    type Error = String;

    fn try_from(tile_pos: &TilePos) -> Result<Self, Self::Error> {
        Ok(AxialPos {
            alpha: tile_pos.x.try_into().map_err(|_| {
                format!(
                    "Could not safely convert unsigned integer {} into `i32`",
                    tile_pos.x
                )
            })?,
            beta: tile_pos.y.try_into().map_err(|_| {
                format!(
                    "Could not safely convert unsigned integer {} into `i32`",
                    tile_pos.x
                )
            })?,
        })
    }
}

impl Add<AxialPos> for AxialPos {
    type Output = AxialPos;

    fn add(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            alpha: self.alpha + rhs.alpha,
            beta: self.beta + rhs.beta,
        }
    }
}

impl Sub<AxialPos> for AxialPos {
    type Output = AxialPos;

    fn sub(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            alpha: self.alpha - rhs.alpha,
            beta: self.beta - rhs.beta,
        }
    }
}

impl From<AxialPos> for RowOddPos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { alpha, beta } = axial_pos;
        let delta = beta / 2;
        RowOddPos {
            alpha: alpha + delta,
            beta,
        }
    }
}

impl From<RowOddPos> for AxialPos {
    fn from(offset_pos: RowOddPos) -> Self {
        let RowOddPos { alpha, beta } = offset_pos;
        let delta = beta / 2;
        AxialPos {
            alpha: alpha - delta,
            beta,
        }
    }
}

impl From<AxialPos> for RowEvenPos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { alpha, beta } = axial_pos;
        // (n + 1) / 2 is a ceil'ed rather than floored division
        let delta = (beta + 1) / 2;
        RowEvenPos {
            alpha: alpha + delta,
            beta,
        }
    }
}

impl From<RowEvenPos> for AxialPos {
    fn from(offset_pos: RowEvenPos) -> Self {
        let RowEvenPos { alpha, beta } = offset_pos;
        let delta = (beta + 1) / 2;
        AxialPos {
            alpha: alpha - delta,
            beta,
        }
    }
}

impl From<AxialPos> for ColOddPos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { alpha, beta } = axial_pos;
        let delta = alpha / 2;
        ColOddPos {
            alpha,
            beta: beta + delta,
        }
    }
}

impl From<ColOddPos> for AxialPos {
    fn from(offset_pos: ColOddPos) -> Self {
        let ColOddPos { alpha, beta } = offset_pos;
        let delta = alpha / 2;
        AxialPos {
            alpha,
            beta: beta - delta,
        }
    }
}

impl From<AxialPos> for ColEvenPos {
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { alpha, beta } = axial_pos;
        // (n + 1) / 2 is a ceil'ed rather than floored division
        let delta = (alpha + 1) / 2;
        ColEvenPos {
            alpha,
            beta: beta + delta,
        }
    }
}

impl From<ColEvenPos> for AxialPos {
    fn from(offset_pos: ColEvenPos) -> Self {
        let ColEvenPos { alpha, beta } = offset_pos;
        let delta = (alpha + 1) / 2;
        AxialPos {
            alpha,
            beta: beta - delta,
        }
    }
}

/// The matrix for mapping from [`AxialPos`](AxialPos), to world position when hexes are arranged
/// in row format ("pointy top" per Red Blob Games). See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that:
///     1) we consider increasing-y to be the same as "going up", while RBG considers increasing-y to be "going down",
///     2) our vectors have magnitude 1 (in order to allow for easy scaling based on grid-size)
pub const ROW_BASIS: Mat2 = Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(0.5, HALF_SQRT_3));

/// The inverse of [`ROW_BASIS`](ROW_BASIS).
pub const INV_ROW_BASIS: Mat2 =
    Mat2::from_cols(Vec2::new(INV_SQRT_3, 0.0), Vec2::new(-1.0 / 3.0, 2.0 / 3.0));

/// The matrix for mapping from [`AxialPos`](AxialPos), to world position when hexes are arranged
/// in column format ("flat top" per Red Blob Games). See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that:
///     1) we consider increasing-y to be the same as "going up", while RBG considers increasing-y to be "going down",
///     2) our vectors have magnitude 1 (in order to allow for easy scaling based on grid-size)
pub const COL_BASIS: Mat2 = Mat2::from_cols(Vec2::new(HALF_SQRT_3, 0.5), Vec2::new(0.0, 1.0));

/// The inverse of [`COL_BASIS`](COL_BASIS).
pub const INV_COL_BASIS: Mat2 =
    Mat2::from_cols(Vec2::new(2.0 / 3.0, -1.0 / 3.0), Vec2::new(0.0, INV_SQRT_3));

impl AxialPos {
    /// The magnitude of a cube position is its distance away from the `(0, 0)` hex.
    ///
    /// See the Red Blob Games article for a [helpful interactive diagram](https://www.redblobgames.com/grids/hexagons/#distances-cube).
    pub fn magnitude(&self) -> i32 {
        let cube_pos = CubePos::from(*self);
        cube_pos.magnitude()
    }

    /// Returns the hex distance between `self` and `other`.
    pub fn distance_from(&self, other: &AxialPos) -> i32 {
        (*self - *other).magnitude()
    }

    /// Returns the center of the hex in world space, assuming that:
    ///     1) tiles are row-oriented ("pointy top"),
    ///     2) the center of the hex with index `(0, 0)` is located at `[0.0, 0.0]`.
    pub fn to_world_pos_row(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let pos_vec = Vec2::new(self.alpha as f32, self.beta as f32);
        let transformed_vec = ROW_BASIS * pos_vec;
        Vec2::new(
            transformed_vec.x * grid_size.x,
            transformed_vec.y * grid_size.y,
        )
    }

    /// Returns the center of the hex in world space, assuming that:
    ///     1) tiles are column-oriented ("flat top"),
    ///     2) the center of the hex with index `(0, 0)` is located at `[0.0, 0.0]`.
    pub fn to_world_pos_col(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let pos_vec = Vec2::new(self.alpha as f32, self.beta as f32);
        let transformed_vec = COL_BASIS * pos_vec;
        Vec2::new(
            transformed_vec.x * grid_size.x,
            transformed_vec.y * grid_size.y,
        )
    }

    /// Returns the axial position of the hex containing the given world position, assuming that:
    ///     1) tiles are row-oriented ("pointy top") and that
    ///     2) the world position corresponding to `[0.0, 0.0]` lies in the hex indexed `(0, 0)`.
    pub fn from_world_pos_row(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let frac_pos = FractionalAxialPos::from(INV_ROW_BASIS * normalized_world_pos);
        frac_pos.round()
    }

    /// Returns the axial position of the hex containing the given world position, assuming that:
    ///     1) tiles are column-oriented ("flat top") and that
    ///     2) the world position corresponding to `[0.0, 0.0]` lies in the hex indexed `(0, 0)`.
    pub fn from_world_pos_col(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let frac_pos = FractionalAxialPos::from(INV_ROW_BASIS * normalized_world_pos);
        frac_pos.round()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FractionalAxialPos {
    pub alpha: f32,
    pub beta: f32,
}

impl FractionalAxialPos {
    fn round(&self) -> AxialPos {
        let frac_cube_pos = FractionalCubePos::from(*self);
        let cube_pos = frac_cube_pos.round();
        cube_pos.into()
    }
}

impl From<Vec2> for FractionalAxialPos {
    fn from(v: Vec2) -> Self {
        FractionalAxialPos {
            alpha: v.x,
            beta: v.y,
        }
    }
}
