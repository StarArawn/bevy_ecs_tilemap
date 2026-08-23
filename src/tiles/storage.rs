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
            *entity = entity_mapper.get_mapped(*entity);
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

    /// Converts `tile_pos` into an index into `tiles`, panicking if it lies outside the map.
    #[inline]
    fn index(&self, tile_pos: &TilePos) -> usize {
        self.checked_index(tile_pos).unwrap_or_else(|| {
            panic!(
                "tile position {tile_pos:?} does not lie within the map extents {:?}",
                self.size
            )
        })
    }

    /// Converts `tile_pos` into an index into `tiles`, or `None` if it lies outside the map.
    #[inline]
    fn checked_index(&self, tile_pos: &TilePos) -> Option<usize> {
        tile_pos.to_checked_index(&self.size)
    }

    /// Gets a tile entity for the given tile position, if an entity is associated with that tile
    /// position.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn get(&self, tile_pos: &TilePos) -> Option<Entity> {
        self.tiles[self.index(tile_pos)]
    }

    /// Gets a tile entity for the given tile position, if:
    /// 1) the tile position lies within the underlying tile map's extents *and*
    /// 2) there is an entity associated with that tile position;
    ///
    /// otherwise it returns `None`.
    pub fn checked_get(&self, tile_pos: &TilePos) -> Option<Entity> {
        *self.tiles.get(self.checked_index(tile_pos)?)?
    }

    /// Sets a tile entity for the given tile position.
    ///
    /// If there is an entity already at that position, it will be replaced.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        let index = self.index(tile_pos);
        self.tiles[index].replace(tile_entity);
    }

    /// Sets a tile entity for the given tile position, if the tile position lies within the
    /// underlying tile map's extents.
    ///
    /// If there is an entity already at that position, it will be replaced.
    pub fn checked_set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        if let Some(index) = self.checked_index(tile_pos) {
            self.tiles[index].replace(tile_entity);
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
        let index = self.index(tile_pos);
        self.tiles[index].take()
    }

    /// Remove any stored `Entity` at the given tile position, leaving `None` in its place and
    /// returning the `Entity`.
    ///
    /// Checks that the given `tile_pos` lies within the extents of the underlying map.
    pub fn checked_remove(&mut self, tile_pos: &TilePos) -> Option<Entity> {
        let index = self.checked_index(tile_pos)?;
        self.tiles.get_mut(index)?.take()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u32) -> Entity {
        Entity::from_raw_u32(id).unwrap()
    }

    const ALIASING_POS: TilePos = TilePos { x: 5, y: 0 };
    const ALIASED_POS: TilePos = TilePos { x: 1, y: 1 };

    fn storage_4x4() -> TileStorage {
        TileStorage::empty(TilemapSize { x: 4, y: 4 })
    }

    #[test]
    fn checked_accessors_round_trip_in_bounds() {
        let pos = TilePos::new(1, 1);
        let mut storage = storage_4x4();

        storage.checked_set(&pos, entity(7));
        assert_eq!(storage.checked_get(&pos), Some(entity(7)));
        assert_eq!(storage.checked_remove(&pos), Some(entity(7)));
        assert_eq!(storage.checked_get(&pos), None);
        assert_eq!(storage.checked_remove(&pos), None);
    }

    #[test]
    fn checked_accessors_reject_out_of_bounds_positions() {
        for out_of_bounds in [ALIASING_POS, TilePos::new(0, u32::MAX)] {
            let mut storage = storage_4x4();
            storage.set(&ALIASED_POS, entity(7));

            assert_eq!(storage.checked_get(&out_of_bounds), None);
            storage.checked_set(&out_of_bounds, entity(9));
            assert_eq!(storage.checked_remove(&out_of_bounds), None);

            assert_eq!(
                storage.iter().flatten().copied().collect::<Vec<_>>(),
                vec![entity(7)],
                "{out_of_bounds:?} must not read, write or remove another tile's entity"
            );
        }
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn get_panics_on_aliasing_position() {
        storage_4x4().get(&ALIASING_POS);
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn set_panics_on_aliasing_position() {
        storage_4x4().set(&ALIASING_POS, entity(7));
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn remove_panics_on_aliasing_position() {
        storage_4x4().remove(&ALIASING_POS);
    }
}
