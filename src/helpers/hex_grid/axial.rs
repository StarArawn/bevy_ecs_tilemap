//! Code for the axial coordinate system.

use crate::helpers::hex_grid::consts::{DOUBLE_INV_SQRT_3, HALF_SQRT_3, INV_SQRT_3};
use crate::helpers::hex_grid::cube::{CubePos, FractionalCubePos};
use crate::helpers::hex_grid::neighbors::{
    HexColDirection, HexDirection, HexRowDirection, HEX_OFFSETS,
};
use crate::helpers::hex_grid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::map::HexCoordSystem;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::{Mat2, Vec2};
use std::ops::{Add, Mul, Sub};

/// A position in a hex grid labelled according to [`HexCoordSystem::Row`] or
/// [`HexCoordSystem::Column`]. It is composed of a pair of `i32` digits named `q` and `r`. When
/// converting from a [`TilePos`], `TilePos.x` is mapped to `q`, while `TilePos.y` is mapped to `r`.
///
/// It is vector-like. In other words: two `AxialPos` can be added/subtracted, and it can be multiplied
/// by an `i32` scalar.
///
/// Since this position type covers both [`HexCoordSystem::Row`] and [`HexCoordSystem::Column`],
/// it has `*_col` and `*_row` variants for important methods.
///
/// It can be converted from/into [`RowOddPos`], [`RowEvenPos`], [`ColOddPos`] and [`ColEvenPos`].
/// It can also be converted from/into [`CubePos`].
///
/// For more information, including interactive diagrams, see
/// [Red Blob Games](https://www.redblobgames.com/grids/hexagons/#coordinates-axial) (RBG). Note
/// however, that while positive `r` goes "downward" in RBG's article, we consider it as going
/// "upward".
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct AxialPos {
    pub q: i32,
    pub r: i32,
}

impl From<&TilePos> for AxialPos {
    fn from(tile_pos: &TilePos) -> Self {
        AxialPos {
            q: tile_pos.x as i32,
            r: tile_pos.y as i32,
        }
    }
}

impl From<CubePos> for AxialPos {
    fn from(cube_pos: CubePos) -> Self {
        let CubePos { q, r, .. } = cube_pos;
        AxialPos { q, r }
    }
}

impl Add<AxialPos> for AxialPos {
    type Output = AxialPos;

    fn add(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl Sub<AxialPos> for AxialPos {
    type Output = AxialPos;

    fn sub(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}

impl Mul<AxialPos> for i32 {
    type Output = AxialPos;

    fn mul(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            q: self * rhs.q,
            r: self * rhs.r,
        }
    }
}

impl Mul<AxialPos> for u32 {
    type Output = AxialPos;

    fn mul(self, rhs: AxialPos) -> Self::Output {
        AxialPos {
            q: (self as i32) * rhs.q,
            r: (self as i32) * rhs.r,
        }
    }
}

#[inline]
fn ceiled_division_by_2(x: i32) -> i32 {
    if x < 0 {
        (x - 1) / 2
    } else {
        (x + 1) / 2
    }
}

impl From<AxialPos> for RowOddPos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q, r } = axial_pos;
        let delta = r / 2;
        RowOddPos { q: q + delta, r }
    }
}

impl From<RowOddPos> for AxialPos {
    #[inline]
    fn from(offset_pos: RowOddPos) -> Self {
        let RowOddPos { q, r } = offset_pos;
        let delta = r / 2;
        AxialPos { q: q - delta, r }
    }
}

impl From<AxialPos> for RowEvenPos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q, r } = axial_pos;
        // (n + 1) / 2 is a ceil'ed rather than floored division
        let delta = ceiled_division_by_2(r);
        RowEvenPos { q: q + delta, r }
    }
}

impl From<RowEvenPos> for AxialPos {
    #[inline]
    fn from(offset_pos: RowEvenPos) -> Self {
        let RowEvenPos { q, r } = offset_pos;
        let delta = ceiled_division_by_2(r);
        AxialPos { q: q - delta, r }
    }
}

impl From<AxialPos> for ColOddPos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q, r } = axial_pos;
        let delta = q / 2;
        ColOddPos { q, r: r + delta }
    }
}

impl From<ColOddPos> for AxialPos {
    #[inline]
    fn from(offset_pos: ColOddPos) -> Self {
        let ColOddPos { q, r } = offset_pos;
        let delta = q / 2;
        AxialPos { q, r: r - delta }
    }
}

impl From<AxialPos> for ColEvenPos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        let AxialPos { q, r } = axial_pos;
        let delta = ceiled_division_by_2(q);
        ColEvenPos { q, r: r + delta }
    }
}

impl From<ColEvenPos> for AxialPos {
    #[inline]
    fn from(offset_pos: ColEvenPos) -> Self {
        let ColEvenPos { q, r } = offset_pos;
        let delta = ceiled_division_by_2(q);
        AxialPos { q, r: r - delta }
    }
}

/// The matrix for mapping from [`AxialPos`], to world position when hexes are arranged
/// in row format ("pointy top" per Red Blob Games). See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that:
///     1) we consider increasing-y to be the same as "going up", while RBG considers increasing-y to be "going down",
///     2) our vectors have magnitude 1 (in order to allow for easy scaling based on grid-size)
pub const ROW_BASIS: Mat2 = Mat2::from_cols(Vec2::new(1.0, 0.0), Vec2::new(0.5, HALF_SQRT_3));

/// The inverse of [`ROW_BASIS`].
pub const INV_ROW_BASIS: Mat2 = Mat2::from_cols(
    Vec2::new(1.0, 0.0),
    Vec2::new(-1.0 * INV_SQRT_3, DOUBLE_INV_SQRT_3),
);

/// The matrix for mapping from [`AxialPos`], to world position when hexes are arranged
/// in column format ("flat top" per Red Blob Games). See
/// [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation, but note that:
///     1) we consider increasing-y to be the same as "going up", while RBG considers increasing-y to be "going down",
///     2) our vectors have magnitude 1 (in order to allow for easy scaling based on grid-size)
pub const COL_BASIS: Mat2 = Mat2::from_cols(Vec2::new(HALF_SQRT_3, 0.5), Vec2::new(0.0, 1.0));

/// The inverse of [`COL_BASIS`].
pub const INV_COL_BASIS: Mat2 = Mat2::from_cols(
    Vec2::new(DOUBLE_INV_SQRT_3, -1.0 * INV_SQRT_3),
    Vec2::new(0.0, 1.0),
);

pub const UNIT_Q: AxialPos = AxialPos { q: 1, r: 0 };

pub const UNIT_R: AxialPos = AxialPos { q: 0, r: -1 };

pub const UNIT_S: AxialPos = AxialPos { q: 1, r: -1 };

impl AxialPos {
    /// The magnitude of an axial position is its distance away from `(0, 0)` in the hex grid.
    ///
    /// See the Red Blob Games article for a [helpful interactive diagram](https://www.redblobgames.com/grids/hexagons/#distances-cube).
    #[inline]
    pub fn magnitude(&self) -> i32 {
        let cube_pos = CubePos::from(*self);
        cube_pos.magnitude()
    }

    /// Returns the distance between `self` and `other` on the hex grid.
    #[inline]
    pub fn distance_from(&self, other: &AxialPos) -> i32 {
        (*self - *other).magnitude()
    }

    /// Project a vector representing a fractional axial position (i.e. the components can be `f32`)
    /// into world space.
    #[inline]
    pub fn project_row(axial_pos: Vec2, grid_size: &TilemapGridSize) -> Vec2 {
        let unscaled_pos = ROW_BASIS * axial_pos;
        Vec2::new(
            grid_size.x * unscaled_pos.x,
            ROW_BASIS.y_axis.y * grid_size.y * unscaled_pos.y,
        )
    }

    /// Returns the center of a hex tile world space, assuming that:
    ///
    /// * Tiles are row-oriented ("pointy top"),
    /// * The center of the hex grid with index `(0, 0)` is located at `[0.0, 0.0]`.
    #[inline]
    pub fn center_in_world_row(&self, grid_size: &TilemapGridSize) -> Vec2 {
        Self::project_row(Vec2::new(self.q as f32, self.r as f32), grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space, assuming that tiles are row-oriented ("pointy top")
    #[inline]
    pub fn corner_offset_in_world_row(
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let corner_offset = AxialPos::from(HexDirection::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.q as f32, corner_offset.r as f32);
        Self::project_row(corner_pos, grid_size)
    }

    /// Returns the coordinate of the corner of a hex tile in the specified `corner_direction`,
    /// in world space, assuming that:
    ///
    /// * Tiles are row-oriented ("pointy top"),
    /// * The center of the hex grid with index `(0, 0)` is located at `[0.0, 0.0]`.
    #[inline]
    pub fn corner_in_world_row(
        &self,
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let center = Vec2::new(self.q as f32, self.r as f32);

        let corner_offset = AxialPos::from(HexDirection::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.q as f32, corner_offset.r as f32);

        Self::project_row(center + corner_pos, grid_size)
    }

    /// Project a vector, representing a fractional axial position (i.e. the components can be `f32`)
    /// on a column-oriented grid ("flat top"), into world space.
    ///
    /// This is a helper function for [`center_in_world_col`](`Self::center_in_world_col`),
    /// [`corner_offset_in_world_col`](`Self::corner_offset_in_world_col`) and
    /// [`corner_in_world_col`](`Self::corner_in_world_col`).
    #[inline]
    pub fn project_col(axial_pos: Vec2, grid_size: &TilemapGridSize) -> Vec2 {
        let unscaled_pos = COL_BASIS * axial_pos;
        Vec2::new(
            COL_BASIS.x_axis.x * grid_size.x * unscaled_pos.x,
            grid_size.y * unscaled_pos.y,
        )
    }

    /// Returns the center of a hex tile world space, assuming that:
    ///
    /// * Tiles are column-oriented ("flat top"),
    /// * The center of the hex grid with index `(0, 0)` is located at `[0.0, 0.0]`.
    #[inline]
    pub fn center_in_world_col(&self, grid_size: &TilemapGridSize) -> Vec2 {
        Self::project_col(Vec2::new(self.q as f32, self.r as f32), grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space, assuming that tiles are col-oriented ("flat top")
    #[inline]
    pub fn corner_offset_in_world_col(
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let corner_offset = AxialPos::from(HexDirection::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.q as f32, corner_offset.r as f32);
        Self::project_col(corner_pos, grid_size)
    }

    /// Returns the coordinate of the corner of a hex tile in the specified `corner_direction`,
    /// in world space, assuming that:
    ///
    /// * Tiles are column-oriented ("flat top"),
    /// * The center of the hex grid with index `(0, 0)` is located at `[0.0, 0.0]`.
    #[inline]
    pub fn corner_in_world_col(
        &self,
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let center = Vec2::new(self.q as f32, self.r as f32);

        let corner_offset = AxialPos::from(HexDirection::from(corner_direction));
        let corner_pos = 0.5 * Vec2::new(corner_offset.q as f32, corner_offset.r as f32);

        Self::project_col(center + corner_pos, grid_size)
    }

    /// Returns the axial position of the hex grid containing the given world position, assuming that:
    ///
    /// * Tiles are row-oriented ("pointy top") and that
    /// * The world position corresponding to `[0.0, 0.0]` lies on the hex grid at index `(0, 0)`.
    #[inline]
    pub fn from_world_pos_row(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(
            world_pos.x / grid_size.x,
            world_pos.y / (ROW_BASIS.y_axis.y * grid_size.y),
        );
        let frac_pos = FractionalAxialPos::from(INV_ROW_BASIS * normalized_world_pos);
        frac_pos.round()
    }

    /// Returns the axial position of the hex grid containing the given world position, assuming that:
    ///
    /// * Tiles are column-oriented ("flat top") and that
    /// * The world position corresponding to `[0.0, 0.0]` lies on the hex grid at index `(0, 0)`.
    #[inline]
    pub fn from_world_pos_col(world_pos: &Vec2, grid_size: &TilemapGridSize) -> AxialPos {
        let normalized_world_pos = Vec2::new(
            world_pos.x / (COL_BASIS.x_axis.x * grid_size.x),
            world_pos.y / grid_size.y,
        );
        let frac_pos = FractionalAxialPos::from(INV_COL_BASIS * normalized_world_pos);
        frac_pos.round()
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `q` or `r` is negative, or lies out of the bounds of
    /// `map_size`.
    #[inline]
    pub fn as_tile_pos_given_map_size(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.q, self.r, map_size)
    }

    /// Convert naively into a [`TilePos`].
    ///
    /// `q` becomes `x` and `r` becomes `y`.
    #[inline]
    pub fn as_tile_pos_unchecked(&self) -> TilePos {
        TilePos {
            x: self.q as u32,
            y: self.r as u32,
        }
    }

    /// Converts an axial position into a tile position in the given hex coordinate system.
    ///
    /// If `hex_coord_sys` is [`RowEven`](HexCoordSystem::RowEven),
    /// [`RowOdd`](HexCoordSystem::RowOdd), or [`ColumnEven`](HexCoordSystem::ColumnEven),
    /// [`ColumnOdd`](HexCoordSystem::ColumnOdd), `self` will be converted into the appropriate
    /// coordinate system before being returned as a `TilePos`.
    #[inline]
    pub fn as_tile_pos_given_coord_system(&self, hex_coord_sys: HexCoordSystem) -> TilePos {
        match hex_coord_sys {
            HexCoordSystem::RowEven => RowEvenPos::from(*self).as_tile_pos_unchecked(),
            HexCoordSystem::RowOdd => RowOddPos::from(*self).as_tile_pos_unchecked(),
            HexCoordSystem::ColumnEven => ColEvenPos::from(*self).as_tile_pos_unchecked(),
            HexCoordSystem::ColumnOdd => ColOddPos::from(*self).as_tile_pos_unchecked(),
            HexCoordSystem::Row | HexCoordSystem::Column => self.as_tile_pos_unchecked(),
        }
    }

    /// Converts an axial position into a tile position in the given hex coordinate system, if it
    /// fits within the extents of the map.
    ///
    /// If `hex_coord_sys` is [`RowEven`](HexCoordSystem::RowEven),
    /// [`RowOdd`](HexCoordSystem::RowOdd), or [`ColumnEven`](HexCoordSystem::ColumnEven),
    /// [`ColumnOdd`](HexCoordSystem::ColumnOdd), `self` will be converted into the appropriate
    /// coordinate system before being returned as a `TilePos`.
    #[inline]
    pub fn as_tile_pos_given_coord_system_and_map_size(
        &self,
        hex_coord_sys: HexCoordSystem,
        map_size: &TilemapSize,
    ) -> Option<TilePos> {
        match hex_coord_sys {
            HexCoordSystem::RowEven => RowEvenPos::from(*self).as_tile_pos_given_map_size(map_size),
            HexCoordSystem::RowOdd => RowOddPos::from(*self).as_tile_pos_given_map_size(map_size),
            HexCoordSystem::ColumnEven => {
                ColEvenPos::from(*self).as_tile_pos_given_map_size(map_size)
            }
            HexCoordSystem::ColumnOdd => {
                ColOddPos::from(*self).as_tile_pos_given_map_size(map_size)
            }
            HexCoordSystem::Row | HexCoordSystem::Column => {
                self.as_tile_pos_given_map_size(map_size)
            }
        }
    }

    /// Converts an axial position into a tile position in the given hex coordinate system.
    ///
    /// If `hex_coord_sys` is [`RowEven`](HexCoordSystem::RowEven),
    /// [`RowOdd`](HexCoordSystem::RowOdd), or [`ColumnEven`](HexCoordSystem::ColumnEven),
    /// [`ColumnOdd`](HexCoordSystem::ColumnOdd), `self` will be converted into the appropriate
    /// coordinate system before being returned as a `TilePos`.
    #[inline]
    pub fn from_tile_pos_given_coord_system(
        tile_pos: &TilePos,
        hex_coord_sys: HexCoordSystem,
    ) -> AxialPos {
        match hex_coord_sys {
            HexCoordSystem::RowEven => RowEvenPos::from(tile_pos).into(),
            HexCoordSystem::RowOdd => RowOddPos::from(tile_pos).into(),
            HexCoordSystem::ColumnEven => ColEvenPos::from(tile_pos).into(),
            HexCoordSystem::ColumnOdd => ColOddPos::from(tile_pos).into(),
            HexCoordSystem::Row | HexCoordSystem::Column => AxialPos::from(tile_pos),
        }
    }

    #[inline]
    pub fn offset(&self, direction: HexDirection) -> AxialPos {
        *self + HEX_OFFSETS[direction as usize]
    }

    #[inline]
    pub fn offset_compass_row(&self, direction: HexRowDirection) -> AxialPos {
        *self + HEX_OFFSETS[direction as usize]
    }

    #[inline]
    pub fn offset_compass_col(&self, direction: HexColDirection) -> AxialPos {
        *self + HEX_OFFSETS[direction as usize]
    }
}

/// A fractional axial position can represent a point that lies inside a hexagon. It is typically
/// the result of mapping a world position into hexagonal space.
///
/// It can be rounded into an [`AxialPos`].
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct FractionalAxialPos {
    pub q: f32,
    pub r: f32,
}

impl FractionalAxialPos {
    #[inline]
    fn round(&self) -> AxialPos {
        let frac_cube_pos = FractionalCubePos::from(*self);
        let cube_pos = frac_cube_pos.round();
        cube_pos.into()
    }
}

impl From<Vec2> for FractionalAxialPos {
    #[inline]
    fn from(v: Vec2) -> Self {
        FractionalAxialPos { q: v.x, r: v.y }
    }
}

impl From<AxialPos> for FractionalAxialPos {
    #[inline]
    fn from(axial_pos: AxialPos) -> Self {
        FractionalAxialPos {
            q: axial_pos.q as f32,
            r: axial_pos.r as f32,
        }
    }
}
