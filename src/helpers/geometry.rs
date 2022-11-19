use crate::helpers::hex_grid::neighbors::{HexColDirection, HexRowDirection};
use crate::helpers::square_grid::neighbors::SquareDirection;
use crate::map::{HexCoordSystem, IsoCoordSystem, TilemapType};
use crate::prelude::axial_system::AxialPos;
use crate::prelude::diamond_system::DiamondPos;
use crate::prelude::staggered_system::StaggeredPos;
use crate::prelude::SquarePos;
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, Transform};
use bevy::math::Vec2;

/// Calculates a [`Transform`] for a tilemap that places it so that its center is at
/// `(0.0, 0.0, 0.0)` in world space.
pub fn get_tilemap_center_transform(
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    z: f32,
) -> Transform {
    let low = TilePos::new(0, 0).center_in_world(grid_size, map_type);
    let high = TilePos::new(size.x - 1, size.y - 1).center_in_world(grid_size, map_type);

    let diff = high - low;

    Transform::from_xyz(-diff.x / 2., -diff.y / 2., z)
}
