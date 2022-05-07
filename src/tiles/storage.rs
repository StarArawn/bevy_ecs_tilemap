use bevy::prelude::*;

use crate::map::Tilemap2dSize;

use super::TilePos2d;

#[derive(Component, Default, Debug, Clone)]
pub struct Tile2dStorage {
    pub tiles: Vec<Option<Entity>>,
    size: Tilemap2dSize,
}

impl Tile2dStorage {
    pub fn empty(size: Tilemap2dSize) -> Self {
        Self {
            tiles: vec![None; size.count()],
            size,
        }
    }

    pub fn get(&self, tile_pos: &TilePos2d) -> Option<Entity> {
        self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size)]
    }

    pub fn set(&mut self, tile_pos: &TilePos2d, tile_entity: Option<Entity>) {
        self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size)] = tile_entity;
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = &Option<Entity>> {
        self.tiles.iter()
    }

    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut Option<Entity>> {
        self.tiles.iter_mut()
    }

    /// Retrieves a list of neighbors in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    /// None will be returned if no valid entity is found at the appropriate coordinate,
    /// including if the tile is at the edge of the map.
    /// ```
    pub fn get_tile_neighbors(&self, tile_pos: &TilePos2d) -> Vec<Option<Entity>> {
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
    pub fn get_neighboring_pos(&self, tile_pos: &TilePos2d) -> [Option<TilePos2d>; 8] {
        let north = if tile_pos.y < self.size.y - 1 {
            Some(TilePos2d {
                x: tile_pos.x,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let south = if tile_pos.y != 0 {
            Some(TilePos2d {
                x: tile_pos.x,
                y: tile_pos.y - 1,
            })
        } else {
            None
        };

        let west = if tile_pos.x != 0 {
            Some(TilePos2d {
                x: tile_pos.x - 1,
                y: tile_pos.y,
            })
        } else {
            None
        };

        let east = if tile_pos.x < self.size.x - 1 {
            Some(TilePos2d {
                x: tile_pos.x + 1,
                y: tile_pos.y,
            })
        } else {
            None
        };

        let northwest = if (tile_pos.x != 0) & (tile_pos.y < self.size.y - 1) {
            Some(TilePos2d {
                x: tile_pos.x - 1,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let northeast = if (tile_pos.x < self.size.x - 1) & (tile_pos.y < self.size.y - 1) {
            Some(TilePos2d {
                x: tile_pos.x + 1,
                y: tile_pos.y + 1,
            })
        } else {
            None
        };

        let southwest = if (tile_pos.x != 0) & (tile_pos.y != 0) {
            Some(TilePos2d {
                x: tile_pos.x - 1,
                y: tile_pos.y - 1,
            })
        } else {
            None
        };

        let southeast = if (tile_pos.x < self.size.x - 1) & (tile_pos.y != 0) {
            Some(TilePos2d {
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
