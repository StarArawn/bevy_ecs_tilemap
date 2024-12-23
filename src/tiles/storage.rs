use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};

use crate::map::TilemapSize;

use super::TilePos;

/// Used to store tile entities for fast look up.
/// Tile entities are stored in a grid. The grid is always filled with None.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component, MapEntities)]
pub struct TileStorage {
    tiles: Vec<Option<Entity>>,
    pub size: TilemapSize,
}

impl MapEntities for TileStorage {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in self.tiles.iter_mut().flatten() {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

impl TileStorage {
    /// Creates a new tile storage that is empty.
    pub fn empty(size: TilemapSize) -> Self {
        Self {
            tiles: vec![None; size.count()],
            size,
        }
    }

    /// Gets a tile entity for the given tile position, if an entity is associated with that tile
    /// position.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn get(&self, tile_pos: &TilePos) -> Option<Entity> {
        self.tiles[tile_pos.to_index(&self.size)]
    }

    /// Gets a tile entity for the given tile position, if:
    /// 1) the tile position lies within the underlying tile map's extents *and*
    /// 2) there is an entity associated with that tile position;
    ///
    /// otherwise it returns `None`.
    pub fn checked_get(&self, tile_pos: &TilePos) -> Option<Entity> {
        if tile_pos.within_map_bounds(&self.size) {
            self.tiles[tile_pos.to_index(&self.size)]
        } else {
            None
        }
    }

    /// Sets a tile entity for the given tile position.
    ///
    /// If there is an entity already at that position, it will be replaced.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        self.tiles[tile_pos.to_index(&self.size)].replace(tile_entity);
    }

    /// Sets a tile entity for the given tile position, if the tile position lies within the
    /// underlying tile map's extents.
    ///
    /// If there is an entity already at that position, it will be replaced.
    pub fn checked_set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        if tile_pos.within_map_bounds(&self.size) {
            self.tiles[tile_pos.to_index(&self.size)].replace(tile_entity);
        }
    }

    /// Returns an iterator with all of the positions in the grid.
    pub fn iter(&self) -> impl Iterator<Item = &Option<Entity>> {
        self.tiles.iter()
    }

    /// Returns mutable iterator with all of the positions in the grid.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<Entity>> {
        self.tiles.iter_mut()
    }

    /// Removes any stored `Entity` at the given tile position, leaving `None` in its place and
    /// returning the `Entity`.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn remove(&mut self, tile_pos: &TilePos) -> Option<Entity> {
        self.tiles[tile_pos.to_index(&self.size)].take()
    }

    /// Remove any stored `Entity` at the given tile position, leaving `None` in its place and
    /// returning the `Entity`.
    ///
    /// Checks that the given `tile_pos` lies within the extents of the underlying map.
    pub fn checked_remove(&mut self, tile_pos: &TilePos) -> Option<Entity> {
        if tile_pos.within_map_bounds(&self.size) {
        self.tiles.get_mut(tile_pos.to_index(&self.size))?.take()
    }

    /// Removes all stored `Entity`s, leaving `None` in their place and
    /// returning them in an iterator.
    ///
    /// Example:
    /// ```
    /// # use bevy::prelude::Commands;
    /// # use bevy_ecs_tilemap::prelude::{TilemapSize, TileStorage};
    /// # fn example(mut commands: Commands) {
    /// # let mut storage = TileStorage::empty(TilemapSize { x: 16, y: 16 });
    /// for entity in storage.drain() {
    ///   commands.entity(entity).despawn();
    /// }
    /// # }
    /// ```
    pub fn drain(&mut self) -> impl Iterator<Item = Entity> + use<'_> {
        self.tiles.iter_mut().filter_map(|opt| opt.take())
    }
}
