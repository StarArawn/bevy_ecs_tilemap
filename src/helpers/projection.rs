use crate::helpers::hexgrid::axial::AxialPos;
use crate::helpers::hexgrid::offset::{ColEvenPos, ColOddPos, RowEvenPos, RowOddPos};
use crate::map::{HexCoordSystem, IsoCoordSystem};
use crate::tiles::TilePos;
use crate::{TilemapGridSize, TilemapSize, TilemapType};
use bevy::math::{Mat2, Vec2};

/// The worlds ("world space") that our grids live in are defined by the Cartesian coordinate
/// system. This is the coordinate system behind pixels and mouse positions.
///
/// Grids, whether square, isometric, or hexagonal, induce their own coordinate systems.
/// Our grids are all two dimensional, so their coordinate systems are all defined two basis
/// vectors. These basis vectors define how we map from grid space to world space, and vice versa.
///
/// For square (rectangular) grids, the basis vectors are parallel to those of the background
/// Cartesian coordinate system. If each tile has height `h`, and width `w`, the basis vectors are
/// `[w, 0]` and `[0, h]` (parallel to `[1, 0]` and `[0, 1]`). The inverse basis vectors are `[1.0 / w, 0.0]`
/// and `[0.0, 1.0 / h]`.
///
/// Hexagonal grids have basis vectors that are 60 degrees (`pi/3`) apart instead of right angled.
/// Note: ideally, the relationship between a hexagon's width (`w`)/height (`h`) and radius (`r`) should be:
///     * for row-oriented hexagonal grids (i.e. "pointy topped" per Red Blob Games): `w = r * sqrt(3.0)` and `h = r * 2.0`
///     * for column-oriented hexagonal grids (i.e. "flat topped" per Red Blob Games): `w = r * 2.0` and `h = r * sqrt(3.0)`.
/// See [Size and Spacing](https://www.redblobgames.com/grids/hexagons/#size-and-spacing)
/// at Red Blob Games for an interactive visual explanation.
///
/// The basis vectors for going from hex space to world space (the background Cartesian space) are:
///     * for row-oriented hexagonal grids (i.e. "pointy topped" per Red Blob Games), the basis
/// vectors are:
///         * if defining using a hexagonal grid radius: `[r * sqrt(3.0), 0.0]` and `[r * sqrt(3.0)/2.0, r * 3.0 / 2.0]`;
///         * if defining using a hexagon width (`w`) and height (`h`): `[w, 0.0]` and `[0.5 * w, 0.75 * h]`.
///     * for column-oriented hexagonal grids (i.e. "flat topped" per Red Blob Games), the basis
/// vectors are:
///         * if defining using a hexagonal grid radius: `[0.0, r * sqrt(3.0)]` and `[r * 3.0 / 2.0, r * sqrt(3.0)/2.0]`;
///         * if defining using a hexagon width (`w`) and height (`h`): `[0.0, h]` and `[0.75 * w, 0.5 * h]`.
/// Note: we can factor out `r` in the hexagonal grid radius based vectors, and this is what we will do.
/// This enables easy stretching/squashing of hexagon grids.
///
/// The inverse basis vectors, for going from world space to hex space are:
///     * for row-oriented hexagonal grids (i.e. "pointy topped" per Red Blob Games), the basis
/// vectors are:
///         * if defining using a hexagonal grid radius: `[1.0 / (r * sqrt(3.0)), 0.0]` and `[-1.0 / (3.0 * r), 2.0 / (3.0 * r)]`;
///         * if defining using a hexagon width (`w`) and height (`h`): `[1.0/w, 0.0]` and `[-2.0 / (3.0 * h), 4.0 / (3.0 * h)]`.
///     * for column-oriented hexagonal grids (i.e. "flat topped" per Red Blob Games), the basis
/// vectors are:
///         * if defining using a hexagonal grid radius: `[-1.0 / (3.0 * r), 2.0 / (3.0 * r)]` and `[1.0 / (r * sqrt(3.0)), 0.0]`;
///         * if defining using a hexagon width (`w`) and height (`h`): `[-2.0 / (3.0 * w), 4.0 / (3.0 * w)]` and `[1.0 / h, 0.0]`
///
/// For diamond isometric grids with tiles of width `w` and height `h`, the basis vectors are
/// `[0.5 * w, -0.5 * h]` and `[0.5 * w, 0.5 * h]`. The inverse basis vectors are `[1.0 / w, 1.0 / w]`
/// and `[-1.0 / h, 1.0 / h]`.  
///
/// For staggered isometric grids with tiles of width `w` and height `h`, the basis vectors are
/// `[w, 0]` and `[0.5 * w, 0.5 * h]`.
///
/// For staggered variations of hexagonal/isometric systems (e.g.
/// [`ColumnOdd`](crate::map::HexCoordSystem::ColumnOdd), or
/// [`Staggered`](crate::map::IsoCoordSystem::Staggered)), the same basis vectors are used, but
/// indexing of tiles is slightly different.

impl TilePos {
    pub fn to_world_pos(self, grid_size: &TilemapGridSize, map_type: &TilemapType) -> Vec2 {
        match map_type {
            TilemapType::Square { .. } => {
                Vec2::new(grid_size.x * (self.x as f32), grid_size.y * (self.y as f32))
            }
            TilemapType::Hexagon(hex_coord_sys) => match hex_coord_sys {
                HexCoordSystem::RowEven => {
                    let pos = RowEvenPos::try_from(&self).unwrap();
                    pos.to_world_pos(grid_size)
                }
                HexCoordSystem::RowOdd => {
                    let pos = RowOddPos::try_from(&self).unwrap();
                    pos.to_world_pos(grid_size)
                }
                HexCoordSystem::ColumnEven => {
                    let pos = ColEvenPos::try_from(&self).unwrap();
                    pos.to_world_pos(grid_size)
                }
                HexCoordSystem::ColumnOdd => {
                    let pos = ColOddPos::try_from(&self).unwrap();
                    pos.to_world_pos(grid_size)
                }
                HexCoordSystem::Row => {
                    let pos = AxialPos::try_from(&self).unwrap();
                    pos.to_world_pos_row(grid_size)
                }
                HexCoordSystem::Column => {
                    let pos = AxialPos::try_from(&self).unwrap();
                    pos.to_world_pos_col(grid_size)
                }
            },
            TilemapType::Isometric { coord_system, .. } => match coord_system {
                IsoCoordSystem::Diamond => {
                    diamond_pos_to_world_pos(self.x as f32, self.y as f32, grid_size.x, grid_size.y)
                }
                IsoCoordSystem::Staggered => staggered_pos_to_world_pos(
                    self.x as f32,
                    self.y as f32,
                    grid_size.x,
                    grid_size.y,
                ),
            },
        }
    }

    fn from_i32_pair(x: i32, y: i32, map_size: &TilemapSize) -> Option<TilePos> {
        let x_u32 = x as u32;
        let y_u32 = y as u32;

        if x < 0 || x_u32 > (map_size.x - 1) || y < 0 || y_u32 > (map_size.y - 1) {
            None
        } else {
            Some(TilePos { x: x_u32, y: y_u32 })
        }
    }

    pub fn from_world_pos(
        world_pos: &Vec2,
        map_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        map_type: &TilemapType,
    ) -> Option<TilePos> {
        match map_type {
            TilemapType::Square { .. } => {
                let x = (world_pos.x / grid_size.x).floor() as i32;
                let y = (world_pos.y / grid_size.y).floor() as i32;

                TilePos::from_i32_pair(x, y, map_size)
            }
            TilemapType::Hexagon(hex_coord_sys) => match hex_coord_sys {
                HexCoordSystem::RowEven => {
                    let pos = RowEvenPos::from_world_pos(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
                HexCoordSystem::RowOdd => {
                    let pos = RowOddPos::from_world_pos(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
                HexCoordSystem::ColumnEven => {
                    let pos = ColEvenPos::from_world_pos(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
                HexCoordSystem::ColumnOdd => {
                    let pos = ColOddPos::from_world_pos(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
                HexCoordSystem::Row => {
                    let pos = AxialPos::from_world_pos_row(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
                HexCoordSystem::Column => {
                    let pos = AxialPos::from_world_pos_col(world_pos, grid_size);
                    TilePos::from_i32_pair(pos.alpha, pos.beta, map_size)
                }
            },
            TilemapType::Isometric { coord_system, .. } => match coord_system {
                IsoCoordSystem::Diamond => world_pos_to_diamond_pos(world_pos, grid_size, map_size),
                IsoCoordSystem::Staggered => {
                    world_pos_to_staggered_pos(world_pos, grid_size, map_size)
                }
            },
        }
    }
}

/// The matrix mapping from tile positions in the diamond isometric system to world space.
pub const DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(0.5, -0.5), Vec2::new(0.5, 0.5));

/// The inverse of [`DIAMOND_BASIS`](DIAMOND_BASIS).
pub const INV_DIAMOND_BASIS: Mat2 = Mat2::from_cols(Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0));

/// Projects an isometric diamond tile position into 2D world space.
///
/// `grid_width` and `grid_height` are the dimensions of the grid in pixels.
pub fn diamond_pos_to_world_pos(x: f32, y: f32, grid_width: f32, grid_height: f32) -> Vec2 {
    let pos = DIAMOND_BASIS * Vec2::new(x, y);
    let scale = Mat2::from_diagonal([grid_width, grid_height].into());
    scale * pos
}

pub fn world_pos_to_diamond_pos(
    world_pos: &Vec2,
    grid_size: &TilemapGridSize,
    map_size: &TilemapSize,
) -> Option<TilePos> {
    let inv_scale = Mat2::from_diagonal(Vec2::new(1.0 / grid_size.x, 1.0 / grid_size.y));
    let pos_f32 = INV_DIAMOND_BASIS * inv_scale * (*world_pos);
    TilePos::from_i32_pair(pos_f32.x as i32, pos_f32.y as i32, map_size)
}

/// Projects an isometric staggered tile position into 2D world space.
///
/// `grid_width` and `grid_height` are the dimensions of the grid in pixels.
pub fn staggered_pos_to_world_pos(x: f32, y: f32, grid_width: f32, grid_height: f32) -> Vec2 {
    diamond_pos_to_world_pos(x, y + x, grid_width, grid_height)
}

pub fn world_pos_to_staggered_pos(
    world_pos: &Vec2,
    grid_size: &TilemapGridSize,
    map_size: &TilemapSize,
) -> Option<TilePos> {
    let inv_scale = Mat2::from_diagonal(Vec2::new(1.0 / grid_size.x, 1.0 / grid_size.y));
    let pos_f32 = INV_DIAMOND_BASIS * inv_scale * (*world_pos);
    let x = pos_f32.x as i32;
    let y = pos_f32.y as i32;
    TilePos::from_i32_pair(x, y - x, map_size)
}
