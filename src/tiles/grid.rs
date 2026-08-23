use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};

use crate::map::TilemapSize;

use super::TilePos;

/// A dense grid holding one `T` per tile position of a [`TilemapSize`].
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct TileGrid<T> {
    size: TilemapSize,
    cells: Vec<T>,
}

impl<T> Default for TileGrid<T> {
    fn default() -> Self {
        Self {
            size: TilemapSize::default(),
            cells: Vec::new(),
        }
    }
}

impl<T: MapEntities> MapEntities for TileGrid<T> {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.cells.map_entities(entity_mapper);
    }
}

impl<T> TileGrid<T> {
    pub fn positions(size: TilemapSize) -> impl Iterator<Item = TilePos> + use<T> {
        (0..size.y).flat_map(move |y| (0..size.x).map(move |x| TilePos { x, y }))
    }

    /// Creates a grid covering `size` where every cell is a clone of `value`.
    pub fn filled(size: TilemapSize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            cells: vec![value; size.count()],
            size,
        }
    }

    /// Creates a grid covering `size`, calling `f` once per position in row-major order.
    pub fn from_fn(size: TilemapSize, f: impl FnMut(TilePos) -> T) -> Self {
        // `positions` has no size hint, so reserve up front.
        let mut cells = Vec::with_capacity(size.count());
        cells.extend(Self::positions(size).map(f));
        Self { size, cells }
    }

    /// Creates a grid covering `size` from already flattened `cells`, in the row-major order of
    /// [`TilePos::to_index`].
    ///
    /// Returns `None` unless `cells.len() == size.count()`.
    pub fn from_cells(size: TilemapSize, cells: Vec<T>) -> Option<Self> {
        (cells.len() == size.count()).then_some(Self { size, cells })
    }

    pub const fn size(&self) -> TilemapSize {
        self.size
    }

    /// The number of cells in this grid, always equal to `self.size().count()`.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether this grid covers no tile positions.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Converts `tile_pos` into an index into `cells`, panicking if it lies outside the grid.
    #[inline]
    fn index(&self, tile_pos: &TilePos) -> usize {
        self.checked_index(tile_pos).unwrap_or_else(|| {
            panic!(
                "tile position {tile_pos:?} does not lie within the map extents {:?}",
                self.size
            )
        })
    }

    /// Converts `tile_pos` into an index into this grid's cells, or `None` if it lies outside the
    /// grid.
    #[inline]
    pub fn checked_index(&self, tile_pos: &TilePos) -> Option<usize> {
        tile_pos.to_checked_index(&self.size)
    }

    /// Gets the cell at the given tile position.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the grid.
    pub fn get(&self, tile_pos: &TilePos) -> &T {
        &self.cells[self.index(tile_pos)]
    }

    /// Mutably gets the cell at the given tile position.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the grid.
    pub fn get_mut(&mut self, tile_pos: &TilePos) -> &mut T {
        let index = self.index(tile_pos);
        &mut self.cells[index]
    }

    /// Gets the cell at the given tile position, or `None` if it lies outside the grid.
    pub fn checked_get(&self, tile_pos: &TilePos) -> Option<&T> {
        self.cells.get(self.checked_index(tile_pos)?)
    }

    /// Mutably gets the cell at the given tile position, or `None` if it lies outside the grid.
    pub fn checked_get_mut(&mut self, tile_pos: &TilePos) -> Option<&mut T> {
        let index = self.checked_index(tile_pos)?;
        self.cells.get_mut(index)
    }

    /// Overwrites the cell at the given tile position, returning the value it replaced.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the grid.
    pub fn set(&mut self, tile_pos: &TilePos, value: T) -> T {
        std::mem::replace(self.get_mut(tile_pos), value)
    }

    /// Overwrites the cell at the given tile position, returning the value it replaced, or `None`
    /// if it lies outside the grid.
    pub fn checked_set(&mut self, tile_pos: &TilePos, value: T) -> Option<T> {
        Some(std::mem::replace(self.checked_get_mut(tile_pos)?, value))
    }

    /// Returns an iterator over all of the cells in the grid, in row-major order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter()
    }

    /// Returns a mutable iterator over all of the cells in the grid, in row-major order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cells.iter_mut()
    }

    /// Returns an iterator over all of the positions the grid covers, in the same order as
    /// [`TileGrid::iter`].
    pub fn iter_pos(&self) -> impl Iterator<Item = TilePos> + use<T> {
        Self::positions(self.size)
    }

    /// Returns an iterator over every position in the grid paired with its cell.
    pub fn iter_with_pos(&self) -> impl Iterator<Item = (TilePos, &T)> {
        Self::positions(self.size).zip(self.cells.iter())
    }

    /// Returns a mutable iterator over every position in the grid paired with its cell.
    pub fn iter_mut_with_pos(&mut self) -> impl Iterator<Item = (TilePos, &mut T)> {
        Self::positions(self.size).zip(self.cells.iter_mut())
    }

    /// The grid's cells as a flat slice, in row-major order.
    pub fn as_slice(&self) -> &[T] {
        &self.cells
    }

    /// The grid's cells as a mutable flat slice, in row-major order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.cells
    }
}

impl<T> TileGrid<Option<T>> {
    /// Creates a grid covering `size` with every cell vacant.
    pub fn empty(size: TilemapSize) -> Self {
        Self {
            // Not `vec![None; _]`: that would require `T: Clone`.
            cells: (0..size.count()).map(|_| None).collect(),
            size,
        }
    }

    /// Stores `value` at the given tile position, returning whatever it replaced.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the grid.
    pub fn insert(&mut self, tile_pos: &TilePos, value: T) -> Option<T> {
        self.set(tile_pos, Some(value))
    }

    /// Removes the value at the given tile position, leaving the cell vacant and returning what
    /// was there.
    ///
    /// Panics if the given `tile_pos` doesn't lie within the extents of the grid.
    pub fn take(&mut self, tile_pos: &TilePos) -> Option<T> {
        self.get_mut(tile_pos).take()
    }

    /// Removes the value at the given tile position, leaving the cell vacant and returning what
    /// was there, or `None` if it lies outside the grid.
    pub fn checked_take(&mut self, tile_pos: &TilePos) -> Option<T> {
        self.checked_get_mut(tile_pos)?.take()
    }

    /// Vacates every cell, returning the values that were stored in an iterator.
    pub fn drain(&mut self) -> impl Iterator<Item = T> + use<'_, T> {
        self.cells.iter_mut().filter_map(|cell| cell.take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct NotClonable(u32);

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

    const SIZE_4X4: TilemapSize = TilemapSize { x: 4, y: 4 };

    // `(5, 0)` flattens to `5` as well, aliasing `(1, 1)`, so it must be rejected.
    const ALIASING_POS: TilePos = TilePos { x: 5, y: 0 };
    const ALIASED_POS: TilePos = TilePos { x: 1, y: 1 };

    #[test]
    fn constructors_cover_exactly_one_cell_per_position() {
        assert_eq!(TileGrid::filled(SIZE_4X4, 0u8).len(), 16);
        assert_eq!(TileGrid::from_fn(SIZE_4X4, |pos| pos.x).len(), 16);
        assert_eq!(TileGrid::<Option<NotClonable>>::empty(SIZE_4X4).len(), 16);

        for grid in [
            TileGrid::filled(SIZE_4X4, 0u8),
            TileGrid::from_fn(SIZE_4X4, |_| 0u8),
        ] {
            assert_eq!(grid.len(), grid.size().count());
        }
    }

    #[test]
    fn default_grid_is_empty() {
        let grid = TileGrid::<u8>::default();

        assert!(grid.is_empty());
        assert_eq!(grid.size(), TilemapSize { x: 0, y: 0 });
        assert_eq!(grid.checked_get(&TilePos::new(0, 0)), None);
    }

    #[test]
    fn from_cells_rejects_a_mismatched_cell_count() {
        assert!(TileGrid::from_cells(SIZE_4X4, vec![0u8; 16]).is_some());
        assert!(TileGrid::from_cells(SIZE_4X4, vec![0u8; 15]).is_none());
        assert!(TileGrid::from_cells(SIZE_4X4, vec![0u8; 17]).is_none());
    }

    #[test]
    fn from_fn_and_iter_with_pos_agree_on_row_major_order() {
        let grid = TileGrid::from_fn(SIZE_4X4, |pos| pos);

        assert!(grid.iter_with_pos().all(|(pos, cell)| pos == *cell));
        assert_eq!(
            grid.iter_pos().take(5).collect::<Vec<_>>(),
            vec![
                TilePos::new(0, 0),
                TilePos::new(1, 0),
                TilePos::new(2, 0),
                TilePos::new(3, 0),
                TilePos::new(0, 1),
            ]
        );
        assert_eq!(grid.as_slice()[5], TilePos::new(1, 1));
    }

    #[test]
    fn positions_are_callable_without_a_grid() {
        assert_eq!(
            TileGrid::<u8>::positions(SIZE_4X4).collect::<Vec<_>>(),
            TileGrid::from_fn(SIZE_4X4, |pos| pos).as_slice()
        );
    }

    #[test]
    fn accessors_round_trip_in_bounds() {
        let pos = TilePos::new(1, 1);
        let mut grid = TileGrid::filled(SIZE_4X4, 0u8);

        assert_eq!(grid.set(&pos, 7), 0);
        assert_eq!(grid.get(&pos), &7);
        assert_eq!(grid.checked_get(&pos), Some(&7));
        assert_eq!(grid.checked_set(&pos, 9), Some(7));
        *grid.get_mut(&pos) += 1;
        assert_eq!(grid.checked_get_mut(&pos), Some(&mut 10));
    }

    #[test]
    fn option_accessors_round_trip_in_bounds() {
        let pos = TilePos::new(1, 1);
        let mut grid = TileGrid::empty(SIZE_4X4);

        assert_eq!(grid.insert(&pos, NotClonable(7)), None);
        assert_eq!(grid.get(&pos), &Some(NotClonable(7)));
        assert_eq!(grid.take(&pos), Some(NotClonable(7)));
        assert_eq!(grid.checked_take(&pos), None);

        assert_eq!(grid.insert(&pos, NotClonable(9)), None);
        assert_eq!(grid.checked_take(&pos), Some(NotClonable(9)));
    }

    #[test]
    fn drain_vacates_every_cell() {
        let mut grid = TileGrid::empty(SIZE_4X4);
        grid.insert(&TilePos::new(0, 0), 1u8);
        grid.insert(&TilePos::new(3, 3), 2u8);

        assert_eq!(grid.drain().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(grid.len(), 16, "draining must not shrink the grid");
        assert!(grid.iter().all(Option::is_none));
    }

    #[test]
    fn checked_accessors_reject_out_of_bounds_positions() {
        for out_of_bounds in [ALIASING_POS, TilePos::new(0, u32::MAX)] {
            let mut grid = TileGrid::empty(SIZE_4X4);
            grid.insert(&ALIASED_POS, 7u8);

            assert_eq!(grid.checked_index(&out_of_bounds), None);
            assert_eq!(grid.checked_get(&out_of_bounds), None);
            assert_eq!(grid.checked_get_mut(&out_of_bounds), None);
            assert_eq!(grid.checked_set(&out_of_bounds, Some(9)), None);
            assert_eq!(grid.checked_take(&out_of_bounds), None);

            assert_eq!(
                grid.iter().flatten().copied().collect::<Vec<_>>(),
                vec![7],
                "{out_of_bounds:?} must not read, write or remove another tile's cell"
            );
        }
    }

    // `map_entities` must reach bare entities, not just ones behind an `Option`.
    #[test]
    fn map_entities_remaps_grids_of_bare_entities() {
        let mut grid = TileGrid::filled(SIZE_4X4, entity(1));
        grid.set(&ALIASED_POS, entity(2));

        grid.map_entities(&mut OffsetMapper);

        assert_eq!(grid.get(&ALIASED_POS), &entity(102));
        assert_eq!(grid.iter().filter(|cell| **cell == entity(101)).count(), 15);
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn get_panics_on_aliasing_position() {
        TileGrid::filled(SIZE_4X4, 0u8).get(&ALIASING_POS);
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn get_mut_panics_on_aliasing_position() {
        TileGrid::filled(SIZE_4X4, 0u8).get_mut(&ALIASING_POS);
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn set_panics_on_aliasing_position() {
        TileGrid::filled(SIZE_4X4, 0u8).set(&ALIASING_POS, 7);
    }

    #[test]
    #[should_panic(expected = "does not lie within the map extents")]
    fn take_panics_on_aliasing_position() {
        TileGrid::<Option<u8>>::empty(SIZE_4X4).take(&ALIASING_POS);
    }
}
