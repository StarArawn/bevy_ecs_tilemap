use crate::helpers::hex_grid::axial::AxialPos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize};
use bevy::math::Vec2;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RowOddPos {
    pub alpha: i32,
    pub beta: i32,
}

impl RowOddPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_row(grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowOddPos::from(axial_pos)
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `alpha` or `beta` is negative, or lies out of the bounds of
    /// `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.alpha, self.beta, map_size)
    }
}

impl From<&TilePos> for RowOddPos {
    fn from(tile_pos: &TilePos) -> Self {
        RowOddPos {
            alpha: tile_pos.x as i32,
            beta: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RowEvenPos {
    pub alpha: i32,
    pub beta: i32,
}

impl RowEvenPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_row(grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowEvenPos::from(axial_pos)
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `alpha` or `beta` is negative, or lies out of the bounds of
    /// `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.alpha, self.beta, map_size)
    }
}

impl From<&TilePos> for RowEvenPos {
    fn from(tile_pos: &TilePos) -> Self {
        RowEvenPos {
            alpha: tile_pos.x as i32,
            beta: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ColOddPos {
    pub alpha: i32,
    pub beta: i32,
}

impl ColOddPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_col(grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_col(world_pos, grid_size);
        ColOddPos::from(axial_pos)
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `alpha` or `beta` is negative, or lies out of the bounds of
    /// `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.alpha, self.beta, map_size)
    }
}

impl From<&TilePos> for ColOddPos {
    fn from(tile_pos: &TilePos) -> Self {
        ColOddPos {
            alpha: tile_pos.x as i32,
            beta: tile_pos.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ColEvenPos {
    pub alpha: i32,
    pub beta: i32,
}

impl ColEvenPos {
    /// Returns the position of this tile's center, in world space.
    pub fn center_in_world(&self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(*self);
        axial_pos.center_in_world_col(grid_size)
    }

    /// Returns the tile containing the given world position.
    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_col(world_pos, grid_size);
        ColEvenPos::from(axial_pos)
    }

    /// Try converting into a [`TilePos`].
    ///
    /// Returns `None` if either one of `alpha` or `beta` is negative, or lies out of the bounds of
    /// `map_size`.
    pub fn as_tile_pos(&self, map_size: &TilemapSize) -> Option<TilePos> {
        TilePos::from_i32_pair(self.alpha, self.beta, map_size)
    }
}

impl From<&TilePos> for ColEvenPos {
    fn from(tile_pos: &TilePos) -> Self {
        ColEvenPos {
            alpha: tile_pos.x as i32,
            beta: tile_pos.y as i32,
        }
    }
}
