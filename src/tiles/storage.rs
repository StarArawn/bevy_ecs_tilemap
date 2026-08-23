use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};

use crate::map::TilemapSize;

use super::{TileGrid, TilePos};

/// Used to store tile entities for fast look up.
/// Tile entities are stored in a grid. The grid is always filled with None.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component, MapEntities)]
pub struct TileStorage(TileGrid<Option<Entity>>);

impl MapEntities for TileStorage {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0.map_entities(entity_mapper);
    }
}

impl From<TileGrid<Option<Entity>>> for TileStorage {
    fn from(grid: TileGrid<Option<Entity>>) -> Self {
        Self(grid)
    }
}

impl TileStorage {
    /// Creates a new tile storage that is empty.
    pub fn empty(size: TilemapSize) -> Self {
        Self(TileGrid::empty(size))
    }

    pub const fn size(&self) -> TilemapSize {
        self.0.size()
    }

    pub const fn grid(&self) -> &TileGrid<Option<Entity>> {
        &self.0
    }

    pub const fn grid_mut(&mut self) -> &mut TileGrid<Option<Entity>> {
        &mut self.0
    }

    pub fn into_inner(self) -> TileGrid<Option<Entity>> {
        self.0
    }

    /// Gets a tile entity for the given tile position, if an entity is associated with that tile
    /// position.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn get(&self, tile_pos: &TilePos) -> Option<Entity> {
        *self.0.get(tile_pos)
    }

    /// Gets a tile entity for the given tile position, if:
    /// 1) the tile position lies within the underlying tile map's extents *and*
    /// 2) there is an entity associated with that tile position;
    ///
    /// otherwise it returns `None`.
    pub fn checked_get(&self, tile_pos: &TilePos) -> Option<Entity> {
        self.0.checked_get(tile_pos).copied().flatten()
    }

    /// Sets a tile entity for the given tile position.
    ///
    /// If there is an entity already at that position, it will be replaced.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        self.0.insert(tile_pos, tile_entity);
    }

    /// Sets a tile entity for the given tile position, if the tile position lies within the
    /// underlying tile map's extents.
    ///
    /// If there is an entity already at that position, it will be replaced.
    pub fn checked_set(&mut self, tile_pos: &TilePos, tile_entity: Entity) {
        self.0.checked_set(tile_pos, Some(tile_entity));
    }

    /// Returns an iterator with all of the entities in the grid, present or not.
    pub fn iter(&self) -> impl Iterator<Item = &Option<Entity>> {
        self.0.iter()
    }

    /// Returns mutable iterator with all of the entities in the grid, present or not.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<Entity>> {
        self.0.iter_mut()
    }

    /// Removes any stored `Entity` at the given tile position, leaving `None` in its place and
    /// returning the `Entity`.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the underlying tile map.
    pub fn remove(&mut self, tile_pos: &TilePos) -> Option<Entity> {
        self.0.take(tile_pos)
    }

    /// Remove any stored `Entity` at the given tile position, leaving `None` in its place and
    /// returning the `Entity`.
    ///
    /// Checks that the given `tile_pos` lies within the extents of the underlying map.
    pub fn checked_remove(&mut self, tile_pos: &TilePos) -> Option<Entity> {
        self.0.checked_take(tile_pos)
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
        self.0.drain()
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::TypeRegistry;

    use super::*;

    fn entity(id: u32) -> Entity {
        Entity::from_raw_u32(id).unwrap()
    }

    // Offsets so a remap is unmistakable.
    struct OffsetMapper;

    impl EntityMapper for OffsetMapper {
        fn get_mapped(&mut self, source: Entity) -> Entity {
            entity(source.index_u32() + 100)
        }

        fn set_mapped(&mut self, _source: Entity, _target: Entity) {}
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

    #[test]
    fn map_entities_remaps_stored_entities_in_place() {
        let mut storage = storage_4x4();
        storage.set(&TilePos::new(0, 0), entity(1));
        storage.set(&TilePos::new(3, 3), entity(2));

        storage.map_entities(&mut OffsetMapper);

        assert_eq!(storage.get(&TilePos::new(0, 0)), Some(entity(101)));
        assert_eq!(storage.get(&TilePos::new(3, 3)), Some(entity(102)));
        assert_eq!(
            storage.iter().flatten().count(),
            2,
            "vacant tile positions must stay vacant"
        );
    }

    // Scene loading reaches `map_entities` by reflection, which the newtype must not break.
    #[test]
    fn reflected_map_entities_stays_registered() {
        let mut registry = TypeRegistry::new();
        registry.register::<TileStorage>();

        let registration = registry.get(std::any::TypeId::of::<TileStorage>()).unwrap();

        assert!(registration.data::<ReflectMapEntities>().is_some());
    }

    #[test]
    fn storage_and_grid_are_interchangeable() {
        let size = TilemapSize { x: 4, y: 4 };
        let mut storage = TileStorage::from(TileGrid::empty(size));
        storage.set(&ALIASED_POS, entity(7));

        assert_eq!(storage.size(), size);
        assert_eq!(storage.grid().len(), size.count());
        assert_eq!(storage.grid().get(&ALIASED_POS), &Some(entity(7)));

        assert_eq!(storage.grid_mut().take(&ALIASED_POS), Some(entity(7)));
        assert!(storage.into_inner().iter().all(Option::is_none));
    }
}
