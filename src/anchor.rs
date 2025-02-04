use bevy::prelude::*;
use crate::{TilemapSize, TilemapType, TilemapGridSize, TilePos};

#[derive(Debug, Clone, Copy, Component, Default)]
pub enum TilemapAnchor {
    #[default]
    None,
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    Custom(Vec2),
    // TileCenter(TilePos),
}

impl TilemapAnchor {
    pub fn from_map(&self, map_size: &TilemapSize, grid_size: &TilemapGridSize, map_type: &TilemapType) -> Transform {
        match self {
            TilemapAnchor::None => {
                Transform::IDENTITY
            }
            TilemapAnchor::TopLeft => {
                let y = map_size.y as f32 * grid_size.y;
                Transform::from_xyz(grid_size.x / 2.0, -y + grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::TopRight => {
                let y = map_size.y as f32 * grid_size.y;
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(-x + grid_size.x / 2.0, -y + grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::TopCenter => {
                let y = map_size.y as f32 * grid_size.y;
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(
                    -x / 2.0 + grid_size.x / 2.0,
                    -y + grid_size.y / 2.0,
                    0.0,
                )
            }
            TilemapAnchor::CenterRight => {
                let y = map_size.y as f32 * grid_size.y;
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(
                    -x + grid_size.x / 2.0,
                    -y / 2.0 + grid_size.y / 2.0,
                    0.0,
                )
            }
            TilemapAnchor::CenterLeft => {
                let y = map_size.y as f32 * grid_size.y;
                Transform::from_xyz(grid_size.x / 2.0, -y / 2.0 + grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::BottomLeft => {
                Transform::from_xyz(grid_size.x / 2.0, grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::BottomRight => {
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(grid_size.x / 2.0 - x, grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::BottomCenter => {
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(grid_size.x / 2.0 - x / 2.0, grid_size.y / 2.0, 0.0)
            }
            TilemapAnchor::Center => {
                let low = TilePos::new(0, 0).center_in_world(grid_size, map_type);
                let high = TilePos::new(map_size.x - 1, map_size.y - 1)
                    .center_in_world(grid_size, map_type);
                let diff = high - low;
                Transform::from_xyz(-diff.x / 2., -diff.y / 2., 0.0)
            }
            TilemapAnchor::Custom(v) => {
                let y = map_size.y as f32 * grid_size.y;
                let x = map_size.x as f32 * grid_size.x;
                Transform::from_xyz(
                    x * (-0.5 - v.x) + grid_size.x / 2.0,
                    y * (-0.5 - v.y) + grid_size.y / 2.0,
                    0.0,
                )
            }
        }
    }
}
