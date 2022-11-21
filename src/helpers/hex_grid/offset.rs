//! Code for the offset coordinate system.

use crate::helpers::hex_grid::axial::AxialPos;
use crate::helpers::hex_grid::neighbors::{HexColDirection, HexDirection, HexRowDirection};
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::Vec2;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RowOddPos {
    pub q: i32,
    pub r: i32,
}

impl RowOddPos {
    /// Returns the position of this tile's center, in world space.
    #[inline]
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_row(grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_offset_in_world(
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        AxialPos::corner_offset_in_world_row(corner_direction, grid_size)
    }

    /// Returns the position of the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_in_world(
        &self,
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.corner_in_world_row(corner_direction, grid_size)
    }

    /// Returns the tile containing the given world position.
    #[inline]
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowOddPos::from(axial_pos)
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

    /// Get the tile offset from `self` in the given [`HexDirection`].
    #[inline]
    pub fn offset(&self, direction: HexDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction))
    }

    /// Get the tile offset from `self` in the given [`HexRowDirection`].
    #[inline]
    pub fn offset_compass(&self, direction: HexColDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction.into()))
    }
}

impl From<&TilePos> for RowOddPos {
    #[inline]
    fn from(tile_pos: &TilePos) -> Self {
        RowOddPos {
            q: tile_pos.x as i32,
            r: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RowEvenPos {
    pub q: i32,
    pub r: i32,
}

impl RowEvenPos {
    /// Returns the position of this tile's center, in world space.
    #[inline]
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_row(grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_offset_in_world(
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        AxialPos::corner_offset_in_world_row(corner_direction, grid_size)
    }

    /// Returns the position of the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_in_world(
        &self,
        corner_direction: HexRowDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.corner_in_world_row(corner_direction, grid_size)
    }

    /// Returns the tile containing the given world position.
    #[inline]
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowEvenPos::from(axial_pos)
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

    /// Get the tile offset from `self` in the given [`HexDirection`].
    #[inline]
    pub fn offset(&self, direction: HexDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction))
    }

    /// Get the tile offset from `self` in the given [`HexRowDirection`].
    #[inline]
    pub fn offset_compass(&self, direction: HexColDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction.into()))
    }
}

impl From<&TilePos> for RowEvenPos {
    #[inline]
    fn from(tile_pos: &TilePos) -> Self {
        RowEvenPos {
            q: tile_pos.x as i32,
            r: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ColOddPos {
    pub q: i32,
    pub r: i32,
}

impl ColOddPos {
    /// Returns the position of this tile's center, in world space.
    #[inline]
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_col(grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_offset_in_world(
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        AxialPos::corner_offset_in_world_col(corner_direction, grid_size)
    }

    /// Returns the position of the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_in_world(
        &self,
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.corner_in_world_col(corner_direction, grid_size)
    }

    /// Returns the tile containing the given world position.
    #[inline]
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_col(world_pos, grid_size);
        ColOddPos::from(axial_pos)
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

    /// Get the tile offset from `self` in the given [`HexDirection`].
    #[inline]
    pub fn offset(&self, direction: HexDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction))
    }

    /// Get the tile offset from `self` in the given [`HexColDirection`].
    #[inline]
    pub fn offset_compass(&self, direction: HexRowDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction.into()))
    }
}

impl From<&TilePos> for ColOddPos {
    #[inline]
    fn from(tile_pos: &TilePos) -> Self {
        ColOddPos {
            q: tile_pos.x as i32,
            r: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ColEvenPos {
    pub q: i32,
    pub r: i32,
}

impl ColEvenPos {
    /// Returns the position of this tile's center, in world space.
    #[inline]
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_col(grid_size)
    }

    /// Returns the offset to the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_offset_in_world(
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        AxialPos::corner_offset_in_world_col(corner_direction, grid_size)
    }

    /// Returns the position of the corner of a hex tile in the specified `corner_direction`,
    /// in world space
    #[inline]
    pub fn corner_in_world(
        &self,
        corner_direction: HexColDirection,
        grid_size: &TilemapGridSize,
    ) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.corner_in_world_col(corner_direction, grid_size)
    }

    /// Returns the tile containing the given world position.
    #[inline]
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_col(world_pos, grid_size);
        ColEvenPos::from(axial_pos)
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

    /// Get the tile offset from `self` in the given [`HexDirection`].
    #[inline]
    pub fn offset(&self, direction: HexDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction))
    }

    /// Get the tile offset from `self` in the given [`HexColDirection`].
    #[inline]
    pub fn offset_compass(&self, direction: HexRowDirection) -> Self {
        Self::from(AxialPos::from(*self).offset(direction.into()))
    }
}

impl From<&TilePos> for ColEvenPos {
    #[inline]
    fn from(tile_pos: &TilePos) -> Self {
        ColEvenPos {
            q: tile_pos.x as i32,
            r: tile_pos.y as i32,
        }
    }
}
