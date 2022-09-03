use bevy::prelude::*;

use crate::map::TilemapSize;

use super::TilePos;

/// Used to store tile entities for fast look up.
/// Tile entities are stored in a grid. The grid is always filled with None.
#[derive(Component, Default, Debug, Clone)]
pub struct TileStorage {
    tiles: Vec<Option<Entity>>,
    pub size: TilemapSize,
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
    pub fn iter(&self) -> impl Iterator<Item = &Option<Entity>> {
        self.tiles.iter()
    }

    /// Returns an immutable iterator with all of the positions in the grid.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<Entity>> {
        self.tiles.iter_mut()
    }
}
