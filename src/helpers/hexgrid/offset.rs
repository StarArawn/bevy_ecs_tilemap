use crate::helpers::hexgrid::axial::AxialPos;
use crate::tiles::TilePos;
use crate::TilemapGridSize;
use bevy::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct RowOddPos {
    pub alpha: i32,
    pub beta: i32,
}

impl RowOddPos {
    pub fn to_world_pos(self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(self);
        axial_pos.to_world_pos_row(grid_size)
    }

    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowOddPos::from(axial_pos)
    }
}

impl TryFrom<&TilePos> for RowOddPos {
    type Error = String;

    fn try_from(tile_pos: &TilePos) -> Result<Self, Self::Error> {
        Ok(RowOddPos {
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

#[derive(Clone, Copy, Debug)]
pub struct RowEvenPos {
    pub alpha: i32,
    pub beta: i32,
}

impl RowEvenPos {
    pub fn to_world_pos(self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(self);
        axial_pos.to_world_pos_row(grid_size)
    }

    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        RowEvenPos::from(axial_pos)
    }
}

impl TryFrom<&TilePos> for RowEvenPos {
    type Error = String;

    fn try_from(tile_pos: &TilePos) -> Result<Self, Self::Error> {
        Ok(RowEvenPos {
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

#[derive(Clone, Copy, Debug)]
pub struct ColOddPos {
    pub alpha: i32,
    pub beta: i32,
}

impl ColOddPos {
    pub fn to_world_pos(self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(self);
        axial_pos.to_world_pos_col(grid_size)
    }

    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        ColOddPos::from(axial_pos)
    }
}

impl TryFrom<&TilePos> for ColOddPos {
    type Error = String;

    fn try_from(tile_pos: &TilePos) -> Result<Self, Self::Error> {
        Ok(ColOddPos {
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

#[derive(Clone, Copy, Debug)]
pub struct ColEvenPos {
    pub alpha: i32,
    pub beta: i32,
}

impl ColEvenPos {
    pub fn to_world_pos(self, grid_size: &TilemapGridSize) -> Vec2 {
        let axial_pos = AxialPos::from(self);
        axial_pos.to_world_pos_col(grid_size)
    }

    pub fn from_world_pos(world_pos: &Vec2, grid_size: &TilemapGridSize) -> Self {
        let axial_pos = AxialPos::from_world_pos_row(world_pos, grid_size);
        ColEvenPos::from(axial_pos)
    }
}

impl TryFrom<&TilePos> for ColEvenPos {
    type Error = String;

    fn try_from(tile_pos: &TilePos) -> Result<Self, Self::Error> {
        Ok(ColEvenPos {
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
