use crate::helpers::hexgrid::consts::{HALF_SQRT_3, INV_SQRT_3, SQRT_3};
use crate::helpers::hexgrid::cube::{CubePos, FractionalCubePos};
use crate::helpers::hexgrid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::tiles::TilePos;
use crate::TilemapGridSize;
use bevy::math::Vec2;
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

/// The basis vector associated with the `alpha` component of [`AxialPos`](AxialPos), when hexagons
/// are arranged in row format ("pointy top" per Red Blob Games). It corresponds with moving
/// one tile to the east. See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const ROW_BASIS_ALPHA: Vec2 = Vec2::new(SQRT_3, 0.0);

/// The basis vector associated with the `beta` component of [`AxialPos`](AxialPos), when hexagons
/// are arranged in row format ("pointy top" per Red Blob Games). It corresponds with moving one
/// tile to the north east. See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const ROW_BASIS_BETA: Vec2 = Vec2::new(HALF_SQRT_3, 1.5);

/// The basis vector associated with the `x` component of a world position, when hexagons
/// are arranged in row format ("pointy top" per Red Blob Games). See
/// [Pixel to Hex](https://www.redblobgames.com/grids/hexagons/#pixel-to-hex)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const ROW_BASIS_X: Vec2 = Vec2::new(INV_SQRT_3, 0.0);

/// The basis vector associated with the `y` component of a world position, when hexagons
/// are arranged in row format ("pointy top" per Red Blob Games). See
/// [Pixel to Hex](https://www.redblobgames.com/grids/hexagons/#pixel-to-hex)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const ROW_BASIS_Y: Vec2 = Vec2::new(-1.0 / 3.0, 2.0 / 3.0);

/// The basis vector associated with the `alpha` component of [`AxialPos`](AxialPos), when hexagons
/// are arranged in column format ("flat top" per Red Blob Games). It corresponds with moving one
/// tile to the north east. See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const COL_BASIS_ALPHA: Vec2 = Vec2::new(1.5, HALF_SQRT_3);

/// The basis vector associated with the `beta` component of [`AxialPos`](AxialPos), when hexagons
/// are arranged in column format ("flat top" per Red Blob Games). It corresponds with moving
/// one tile to the north. See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const COL_BASIS_BETA: Vec2 = Vec2::new(0.0, SQRT_3);

/// The basis vector associated with the `x` component of a world position, when hexagons
/// are arranged in column format ("flat top" per Red Blob Games). See
/// [Pixel to Hex](https://www.redblobgames.com/grids/hexagons/#pixel-to-hex)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const COL_BASIS_X: Vec2 = Vec2::new(2.0 / 3.0, -1.0 / 3.0);

/// The basis vector associated with the `y` component of a world position, when hexagons
/// are arranged in column format ("flat top" per Red Blob Games). See
/// [Pixel to Hex](https://www.redblobgames.com/grids/hexagons/#pixel-to-hex)
/// at Red Blob Games for an interactive visual explanation, but note that we consider increasing-y
/// to be the same as "going up", while RBG considers increasing-y to be "going down".
pub const COL_BASIS_Y: Vec2 = Vec2::new(0.0, INV_SQRT_3);

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
        grid_size.x * (self.alpha as f32) * ROW_BASIS_ALPHA
            + grid_size.y * (self.beta as f32) * ROW_BASIS_BETA
    }

    /// Returns the center of the hex in world space, assuming that:
    ///     1) tiles are column-oriented ("flat top"),
    ///     2) the center of the hex with index `(0, 0)` is located at `[0.0, 0.0]`.
    pub fn to_world_pos_col(&self, grid_size: &TilemapGridSize) -> Vec2 {
        grid_size.x * (self.alpha as f32) * COL_BASIS_ALPHA
            + grid_size.y * (self.beta as f32) * COL_BASIS_BETA
    }

    /// Returns the axial position of the hex containing the given world position, assuming that:
    ///     1) tiles are row-oriented ("pointy top") and that
    ///     2) the world position corresponding to `[0.0, 0.0]` lies in the hex indexed `(0, 0)`.
    pub fn from_world_pos_row(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let frac_pos = FractionalAxialPos::from(
            normalized_world_pos.x * ROW_BASIS_X + normalized_world_pos.y * ROW_BASIS_Y,
        );
        frac_pos.round()
    }

    /// Returns the axial position of the hex containing the given world position, assuming that:
    ///     1) tiles are column-oriented ("flat top") and that
    ///     2) the world position corresponding to `[0.0, 0.0]` lies in the hex indexed `(0, 0)`.
    pub fn from_world_pos_col(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(world_pos.x / grid_size.x, world_pos.y / grid_size.y);
        let frac_pos = FractionalAxialPos::from(
            normalized_world_pos.x * COL_BASIS_X + normalized_world_pos.y * COL_BASIS_Y,
        );
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
