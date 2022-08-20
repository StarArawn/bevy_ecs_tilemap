use bevy::{
    math::Vec2,
    prelude::{Commands, Entity, Transform},
};

use crate::{
    map::{HexCoordSystem, IsoCoordSystem, TilemapId, TilemapSize, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TilePos, TileStorage, TileTexture},
};

/// Converts a tile position into an index in a vector.
pub fn pos_2d_to_index(tile_pos: &TilePos, size: &TilemapSize) -> usize {
    ((tile_pos.y * size.x as u32) + tile_pos.x) as usize
}

/// Calculates a chunk position with the given information.
/// Note: The calculation is different depending on the tilemap's mesh type.
/// This calculation is mostly used internally for rendering but it might be helpful so it's exposed here.
pub fn get_chunk_2d_transform(
    chunk_position: Vec2,
    grid_size: Vec2,
    chunk_size: Vec2,
    z_index: u32,
    graph_type: TilemapType,
) -> Transform {
    let pos = match graph_type {
        TilemapType::Square { .. } => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * grid_size.x;
            let chunk_pos_y = chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapType::Hexagon(HexCoordSystem::Row) => {
            let chunk_pos_x = (chunk_position.y * chunk_size.x * (0.5 * grid_size.x).floor())
                + (chunk_position.x * chunk_size.x * grid_size.x);
            let chunk_pos_y = chunk_position.y * chunk_size.y * (0.75 * grid_size.y).floor();
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd)
        | TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * grid_size.x;
            let chunk_pos_y = chunk_position.y * chunk_size.y * (0.75 * grid_size.y).floor();
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapType::Hexagon(HexCoordSystem::Column) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * (0.75 * grid_size.x).floor();
            let chunk_pos_y = (chunk_position.x * chunk_size.y * (0.5 * grid_size.y).ceil())
                + chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
        | TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * (0.75 * grid_size.x).floor();
            let chunk_pos_y = chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapType::Isometric(IsoCoordSystem::Diamond) => project_iso_diamond(
            chunk_position.x,
            chunk_position.y,
            chunk_size.x * grid_size.x,
            chunk_size.y * grid_size.y,
        ),
        TilemapType::Isometric(IsoCoordSystem::Staggered) => project_iso_staggered(
            chunk_position.x,
            chunk_position.y,
            chunk_size.x * grid_size.x,
            chunk_size.y,
        ),
    };

    Transform::from_xyz(pos.x, pos.y, z_index as f32)
}

/// Fills an entire tile storage with the given tile.
pub fn fill_tilemap(
    tile_texture: TileTexture,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Fills a rectangular region with the given tile.
pub fn fill_tilemap_rect(
    tile_texture: TileTexture,
    pos: TilePos,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in pos.x..size.x {
        for y in pos.y..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Calculates a tilemap's centered position.
pub fn get_centered_transform_2d(
    size: &TilemapSize,
    tile_size: &TilemapTileSize,
    z_index: f32,
) -> Transform {
    Transform::from_xyz(
        -(size.x as f32 * tile_size.x as f32) / 2.0,
        -(size.y as f32 * tile_size.y as f32) / 2.0,
        z_index,
    )
}

/// Projects a 2D screen space point into isometric diamond space.
pub fn project_iso_diamond(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = (x - y) * pixel_width / 2.0;
    let new_y = (x + y) * pixel_height / 2.0;
    Vec2::new(new_x, -new_y)
}

/// Projects a 2D screen space point into isometric staggered space.
pub fn project_iso_staggered(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = x * pixel_width;
    let new_y = y * pixel_height;
    Vec2::new(new_x, new_y)
}

#[derive(Clone, Copy, Debug)]
pub enum NeighborDirection {
    North,
    NorthWest,
    West,
    SouthWest,
    South,
    SouthEast,
    East,
    NorthEast,
}

impl NeighborDirection {
    fn next_direction(&self) -> NeighborDirection {
        use NeighborDirection::*;
        match self {
            North => NorthWest,
            NorthWest => West,
            West => SouthWest,
            SouthWest => South,
            South => SouthEast,
            SouthEast => East,
            East => NorthEast,
            NorthEast => North,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Neighbors<T: Copy> {
    pub north: Option<T>,
    pub north_west: Option<T>,
    pub west: Option<T>,
    pub south_west: Option<T>,
    pub south: Option<T>,
    pub south_east: Option<T>,
    pub east: Option<T>,
    pub north_east: Option<T>,
}

pub struct NeighborsIntoIterator<T: Copy> {
    neighbors: Neighbors<T>,
    /// The next direction the iterator will output.
    cursor: Option<NeighborDirection>,
}

impl<T: Copy> NeighborsIntoIterator<T> {
    fn get_at_cursor(&self) -> Option<T> {
        self.cursor.and_then(|direction| match direction {
            NeighborDirection::North => self.neighbors.north,
            NeighborDirection::NorthWest => self.neighbors.north_west,
            NeighborDirection::West => self.neighbors.west,
            NeighborDirection::SouthWest => self.neighbors.south_west,
            NeighborDirection::South => self.neighbors.south,
            NeighborDirection::SouthEast => self.neighbors.south_east,
            NeighborDirection::East => self.neighbors.east,
            NeighborDirection::NorthEast => self.neighbors.north_east,
        })
    }
}

impl<T: Copy> Iterator for NeighborsIntoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.and_then(|direction| {
            let neighbor = self.get_at_cursor();
            match direction {
                NeighborDirection::NorthEast => {
                    self.cursor = None;
                    neighbor
                }
                direction => {
                    self.cursor = Some(direction.next_direction());
                    neighbor.or_else(|| self.next())
                }
            }
        })
    }
}

impl<T: Copy> IntoIterator for Neighbors<T> {
    type Item = T;
    type IntoIter = NeighborsIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        NeighborsIntoIterator {
            neighbors: self,
            cursor: Some(NeighborDirection::North),
        }
    }
}

impl<T: Copy> Neighbors<T> {
    pub fn count(&self) -> usize {
        self.into_iter().map(|_| 1).sum()
    }
}

impl Neighbors<Entity> {
    fn from_neighboring_pos(
        neighbors: &Neighbors<TilePos>,
        tile_storage: &TileStorage,
    ) -> Neighbors<Entity> {
        Neighbors {
            north: neighbors.north.and_then(|pos| tile_storage.get(&pos)),
            south: neighbors.south.and_then(|pos| tile_storage.get(&pos)),
            east: neighbors.east.and_then(|pos| tile_storage.get(&pos)),
            west: neighbors.west.and_then(|pos| tile_storage.get(&pos)),
            north_east: neighbors.north_east.and_then(|pos| tile_storage.get(&pos)),
            north_west: neighbors.north_west.and_then(|pos| tile_storage.get(&pos)),
            south_east: neighbors.south_east.and_then(|pos| tile_storage.get(&pos)),
            south_west: neighbors.south_west.and_then(|pos| tile_storage.get(&pos)),
        }
    }
}

/// Retrieves a list of neighbors for the given tile position.
///
/// If a particular neighboring position does not exist in the provided storage,
/// then it will not be returned.
pub fn get_tile_neighbors(
    tile_pos: &TilePos,
    tile_storage: &TileStorage,
    tilemap_type: TilemapType,
) -> Neighbors<Entity> {
    Neighbors::from_neighboring_pos(
        &get_neighboring_pos(tile_pos, &tile_storage.size, tilemap_type),
        tile_storage,
    )
}

/// Retrieves the positions of neighbors of the tile with the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn get_neighboring_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
    map_type: TilemapType,
) -> Neighbors<TilePos> {
    match map_type {
        TilemapType::Square {
            neighbors_include_diagonals,
        } => {
            if neighbors_include_diagonals {
                square_neighbor_pos_with_diagonals(tile_pos, tilemap_size)
            } else {
                square_neighbor_pos(tile_pos, tilemap_size)
            }
        }
        _ => unimplemented!(),
        // TilemapType::Hexagon(HexCoordSystem::Column) => {}
        // TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {}
        // TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {}
        // TilemapType::Hexagon(HexCoordSystem::Row) => {}
        // TilemapType::Hexagon(HexCoordSystem::RowEven) => {}
        // TilemapType::Hexagon(HexCoordSystem::RowOdd) => {}
        // TilemapType::Isometric(_) => {}
    }
}

impl TilePos {
    #[inline]
    fn plus_x(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y < tilemap_size.y - 1 {
            Some(TilePos {
                x: self.x,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_xy(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 && self.y < tilemap_size.y - 1 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_x(&self) -> Option<TilePos> {
        if self.x != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_y(&self) -> Option<TilePos> {
        if self.y != 0 {
            Some(TilePos {
                x: self.x,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_xy(&self) -> Option<TilePos> {
        if self.x != 0 && self.y != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn plus_x_minus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x < tilemap_size.x - 1 && self.y != 0 {
            Some(TilePos {
                x: self.x + 1,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[inline]
    fn minus_x_plus_y(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y < tilemap_size.y - 1 && self.x != 0 {
            Some(TilePos {
                x: self.x - 1,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    fn square_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_y(tilemap_size)
    }

    fn square_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn square_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn square_south_west(&self) -> Option<TilePos> {
        self.minus_xy()
    }

    fn square_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn square_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn square_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn square_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_xy(tilemap_size)
    }

    fn hex_row_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn hex_row_odd_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_y(tilemap_size)
        } else {
            self.minus_x_plus_y(tilemap_size)
        }
    }

    fn hex_row_even_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_y(tilemap_size)
        } else {
            self.minus_x_plus_y(tilemap_size)
        }
    }

    fn hex_row_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_odd_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_even_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_row_south_west(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_row_odd_south_west(&self) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.minus_y()
        } else {
            self.minus_xy()
        }
    }

    fn hex_row_even_south_west(&self) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.minus_y()
        } else {
            self.minus_xy()
        }
    }

    fn hex_row_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn hex_row_odd_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_x_minus_y(tilemap_size)
        } else {
            self.minus_y()
        }
    }

    fn hex_row_even_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_x_minus_y(tilemap_size)
        } else {
            self.minus_y()
        }
    }

    fn hex_row_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_odd_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_even_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_row_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_y(tilemap_size)
    }

    fn hex_row_odd_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 == 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_row_even_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.y % 2 != 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_col_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_odd_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_even_north(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.minus_x_plus_y(tilemap_size)
    }

    fn hex_col_odd_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.minus_x_plus_y(tilemap_size)
        } else {
            self.minus_x()
        }
    }

    fn hex_col_even_north_west(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.minus_x_plus_y(tilemap_size)
        } else {
            self.minus_x()
        }
    }

    fn hex_col_south_west(&self) -> Option<TilePos> {
        self.minus_x()
    }

    fn hex_col_odd_south_west(&self) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.minus_x()
        } else {
            self.minus_xy()
        }
    }

    fn hex_col_even_south_west(&self) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.minus_x()
        } else {
            self.minus_xy()
        }
    }

    fn hex_col_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_odd_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_even_south(&self) -> Option<TilePos> {
        self.minus_y()
    }

    fn hex_col_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x_minus_y(tilemap_size)
    }

    fn hex_col_odd_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.plus_x(tilemap_size)
        } else {
            self.plus_x_minus_y(tilemap_size)
        }
    }

    fn hex_col_even_south_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.plus_x(tilemap_size)
        } else {
            self.plus_x_minus_y(tilemap_size)
        }
    }

    fn hex_col_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        self.plus_x(tilemap_size)
    }

    fn hex_col_odd_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 == 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }

    fn hex_col_even_north_east(&self, tilemap_size: &TilemapSize) -> Option<TilePos> {
        if self.x % 2 != 0 {
            self.plus_xy(tilemap_size)
        } else {
            self.plus_x(tilemap_size)
        }
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap with square grid, where neighbors include tiles located
/// diagonally across from the specified position.
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn square_neighbor_pos_with_diagonals(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.square_north(tilemap_size),
        south: tile_pos.square_south(),
        east: tile_pos.square_east(tilemap_size),
        west: tile_pos.square_west(),
        north_east: tile_pos.square_north_east(tilemap_size),
        north_west: tile_pos.square_north_west(tilemap_size),
        south_east: tile_pos.square_south_east(tilemap_size),
        south_west: tile_pos.square_south_west(),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of [`TilemapType::Square`](crate::map::TilemapType::Square).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn square_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.square_north(tilemap_size),
        south: tile_pos.square_south(),
        east: tile_pos.square_east(tilemap_size),
        west: tile_pos.square_west(),
        north_east: None,
        north_west: None,
        south_east: None,
        south_west: None,
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon) with
/// [`HexCoordSystem::Row`](crate::map::HexCoordSystem::Row).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_north_west(tilemap_size),
        west: tile_pos.hex_row_west(),
        south_west: tile_pos.hex_row_south_west(),
        south: None,
        south_east: tile_pos.hex_row_south_east(tilemap_size),
        east: tile_pos.hex_row_east(tilemap_size),
        north_east: tile_pos.hex_row_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon)
/// with [`HexCoordSystem::RowOdd`](crate::map::HexCoordSystem::RowOdd).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_odd_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_odd_north_west(tilemap_size),
        west: tile_pos.hex_row_odd_west(),
        south_west: tile_pos.hex_row_odd_south_west(),
        south: None,
        south_east: tile_pos.hex_row_odd_south_east(tilemap_size),
        east: tile_pos.hex_row_odd_east(tilemap_size),
        north_east: tile_pos.hex_row_odd_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon) with
/// [`HexCoordSystem::Row`](crate::map::HexCoordSystem::RowEven).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_row_even_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: None,
        north_west: tile_pos.hex_row_north_west(tilemap_size),
        west: tile_pos.hex_row_west(),
        south_west: tile_pos.hex_row_south_west(),
        south: None,
        south_east: tile_pos.hex_row_south_east(tilemap_size),
        east: tile_pos.hex_row_east(tilemap_size),
        north_east: tile_pos.hex_row_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon) with
/// [`HexCoordSystem::Col`](crate::map::HexCoordSystem::Col).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_neighbor_pos(tile_pos: &TilePos, tilemap_size: &TilemapSize) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_north(tilemap_size),
        north_west: tile_pos.hex_col_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_south_west(),
        south: tile_pos.hex_col_south(),
        south_east: tile_pos.hex_col_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon) with
/// [`HexCoordSystem::ColOdd`](crate::map::HexCoordSystem::ColOdd).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_odd_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_odd_north(tilemap_size),
        north_west: tile_pos.hex_col_odd_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_odd_south_west(),
        south: tile_pos.hex_col_odd_south(),
        south_east: tile_pos.hex_col_odd_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_odd_north_east(tilemap_size),
    }
}

/// Retrieves the positions of neighbors of the tile with the specified position, assuming
/// the tile exists on a tilemap of type [`TilemapType::Hexagon`](crate::map::TilemapType::Hexagon) with
/// [`HexCoordSystem::ColEven`](crate::map::HexCoordSystem::ColEven).
///
/// Tile positions are bounded:
///     * between `0` and `tilemap_size.x` in the `x` position,
///     * between `0` and `tilemap_size.y` in the `y` position.
/// Directions in the returned [`Neighbor`](crate::helpers::Neighbor) struct with tile coordinates that violate these requirements will be set to `None`.
pub fn hex_col_even_neighbor_pos(
    tile_pos: &TilePos,
    tilemap_size: &TilemapSize,
) -> Neighbors<TilePos> {
    Neighbors {
        north: tile_pos.hex_col_even_north(tilemap_size),
        north_west: tile_pos.hex_col_even_north_west(tilemap_size),
        west: None,
        south_west: tile_pos.hex_col_even_south_west(),
        south: tile_pos.hex_col_even_south(),
        south_east: tile_pos.hex_col_even_south_east(tilemap_size),
        east: None,
        north_east: tile_pos.hex_col_even_north_east(tilemap_size),
    }
}

/// Returns the bottom-left coordinate of the tile associated with the specified `tile_pos`.
fn get_tile_pos_in_world_space(
    tile_pos: &TilePos,
    grid_size: Vec2,
    tilemap_type: &TilemapType,
) -> Vec2 {
    let tile_pos_f32 = Vec2::new(tile_pos.x as f32, tile_pos.y as f32);
    let mut pos = Vec2::default();
    pos.x = grid_size.x * tile_pos_f32.x;
    pos.y = grid_size.y * tile_pos_f32.y;

    match tilemap_type {
        TilemapType::Hexagon(HexCoordSystem::Row) => {
            let x_offset = tile_pos_f32.y * (0.5 * grid_size.x).floor();
            let y_offset = -1.0 * tile_pos_f32.y * (0.25 * grid_size.y).ceil();
            pos.x += x_offset;
            pos.y += y_offset;
        }
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            let offset = (0.25 * grid_size.x).floor();
            if tile_pos.y % 2 == 0 {
                pos.x = pos.x - offset;
            } else {
                pos.x = pos.x + offset;
            }
            pos.y = pos.y - tile_pos_f32.y * (0.25 * grid_size.y as f32).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            let offset = (0.25 * grid_size.x).floor();
            if tile_pos.y % 2 == 0 {
                pos.x = pos.x + offset;
            } else {
                pos.x = pos.x - offset;
            }
            pos.y = pos.y - tile_pos_f32.y * (0.25 * grid_size.y).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::Column) => {
            let x_offset = -1.0 * tile_pos_f32.x * (0.25 * grid_size.x).floor();
            let y_offset = tile_pos_f32.x * (0.5 * grid_size.y).ceil();
            pos.x += x_offset;
            pos.y += y_offset;
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            let offset = (0.25 * grid_size.y).floor();
            if tile_pos.x % 2 == 0 {
                pos.y = pos.y - offset;
            } else {
                pos.y = pos.y + offset;
            }
            pos.x = pos.x - tile_pos_f32.x * (0.25 * grid_size.x as f32).ceil();
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            let offset = (0.25 * grid_size.y).floor();
            if tile_pos.x % 2 == 0 {
                pos.y = pos.y + offset;
            } else {
                pos.y = pos.y - offset;
            }
            pos.x = pos.x - tile_pos_f32.x * (0.25 * grid_size.x).ceil();
        }
        _ => unimplemented!(
            "get_tile_pos_in_world_space is unimplemented for graph type: {tilemap_type:?}"
        ),
    };
    pos
}
