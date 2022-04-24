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
}
