use bevy::prelude::*;

use crate::map::TilemapSize;

use super::TilePos;

/// Used to store tile entities for fast look up.
/// Tile entities are stored in a grid. The grid is always filled with None.
#[derive(Component, Default, Debug, Clone)]
pub struct TileStorage {
    tiles: Vec<Option<Entity>>,
    size: TilemapSize,
}

impl TileStorage {
    /// Creates a new tile storage that is empty.
    pub fn empty(size: TilemapSize) -> Self {
        Self {
            tiles: vec![None; size.count()],
            size,
        }
    }

    /// Gets a tile entity for the given tile position.
    pub fn get(&self, tile_pos: &TilePos) -> Option<Entity> {
        self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size)]
    }

    /// Sets a tile entity for the given tile position.
    pub fn set(&mut self, tile_pos: &TilePos, tile_entity: Option<Entity>) {
        self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size)] = tile_entity;
    }

    /// Returns an iterator with all of the positions in the grid.
    pub fn iter(&self) -> impl std::iter::Iterator<Item = &Option<Entity>> {
        self.tiles.iter()
    }

    /// Returns an immutable iterator with all of the positions in the grid.
    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut Option<Entity>> {
        self.tiles.iter_mut()
    }

    /// Retrieves a list of neighbors in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    /// None will be returned if no valid entity is found at the appropriate coordinate,
    /// including if the tile is at the edge of the map.
    ///
    pub fn get_tile_neighbors(&self, tile_pos: &TilePos) -> Vec<Option<Entity>> {
        let neighboring_tile_pos = self.get_neighboring_pos(tile_pos);

        neighboring_tile_pos
            .iter()
            .map(|maybe_pos| match maybe_pos {
                Some(pos) => self.get(pos),
                None => None,
            })
            .collect::<Vec<_>>()
    }

    /// Gets the positions of the neighbors of the specified position
    /// Order: N, S, W, E, NW, NE, SW, SE.
    ///
    /// Tile positions are bounded between 0 and u32::MAX, so None may be returned
    pub fn get_neighboring_pos(&self, tile_pos: &TilePos) -> [Option<TilePos>; 8] {
        let north = if tile_pos.y < self.size.y - 1 {
            Some(TilePos {
                x: tile_pos.x,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let south = if tile_pos.y != 0 {
            Some(TilePos {
                x: tile_pos.x,
                y: tile_pos.y - 1,
            })
        } else {
            None
        };

        let west = if tile_pos.x != 0 {
            Some(TilePos {
                x: tile_pos.x - 1,
                y: tile_pos.y,
            })
        } else {
            None
        };

        let east = if tile_pos.x < self.size.x - 1 {
            Some(TilePos {
                x: tile_pos.x + 1,
                y: tile_pos.y,
            })
        } else {
            None
        };

        let northwest = if (tile_pos.x != 0) & (tile_pos.y < self.size.y - 1) {
            Some(TilePos {
                x: tile_pos.x - 1,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let northeast = if (tile_pos.x < self.size.x - 1) & (tile_pos.y < self.size.y - 1) {
            Some(TilePos {
                x: tile_pos.x + 1,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let southwest = if (tile_pos.x != 0) & (tile_pos.y != 0) {
            Some(TilePos {
                x: tile_pos.x - 1,
                y: tile_pos.y - 1,
            })
        } else {
            None
        };

        let southeast = if (tile_pos.x < self.size.x - 1) & (tile_pos.y != 0) {
            Some(TilePos {
                x: tile_pos.x + 1,
                y: tile_pos.y - 1,
            })
        } else {
            None
        };

        [
            north, south, west, east, northwest, northeast, southwest, southeast,
        ]
    }
}
