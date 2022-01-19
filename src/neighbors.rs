use crate::layer::{LayerId, MapTileError};
use crate::layer_builder::LayerBuilder;
use crate::map::MapId;
use crate::map_query::MapQuery;
use crate::tile::TileBundleTrait;
use crate::TilePos;

use bevy::prelude::Entity;

impl<T: TileBundleTrait> LayerBuilder<T> {
    /// Retrieves a list of neighbors in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    /// None will be returned if no valid entity is found at the appropriate coordinate,
    /// including if the tile is at the edge of the map.
    /// ```
    pub fn get_tile_neighbors(&self, tile_pos: TilePos) -> Vec<Option<(Option<Entity>, &T)>> {
        let neighboring_tile_pos = get_neighboring_pos(tile_pos);

        neighboring_tile_pos
            .iter()
            .map(|maybe_pos| match maybe_pos {
                Some(pos) => self.get_tile_full(*pos),
                None => None,
            })
            .collect::<Vec<_>>()
    }
}

impl<'w, 's> MapQuery<'w, 's> {
    /// Retrieves a list of neighbor entities in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    ///
    /// If a tile's coordinates are out of bounds, None will be returned in the Option<TilePos>
    /// If a tile's coordinates are valid but no tile entity is found, None will be returned in the Option<Entity>
    ///
    /// ## Example
    ///
    /// ```
    /// let neighbors = map.get_tile_neighbors(TilePos(0, 0));
    /// assert!(neighbors[1].1.is_none()); // Outside of tile bounds.
    /// assert!(neighbors[0].1.is_none()); // Entity returned inside bounds.
    /// ```
    pub fn get_tile_neighbors(
        &mut self,
        tile_pos: TilePos,
        map_id: impl MapId,
        layer_id: impl LayerId,
    ) -> Vec<Result<Entity, MapTileError>> {
        let mut neighboring_tile_pos = get_neighboring_pos(tile_pos);

        neighboring_tile_pos
            .iter_mut()
            .map(|maybe_pos| match maybe_pos {
                Some(pos) => self.get_tile_entity(*pos, map_id, layer_id),
                _ => Err(MapTileError::OutOfBounds),
            })
            .collect::<Vec<_>>()
    }
}

/// Gets the positions of the neighbors of the specified position
/// Order: N, S, W, E, NW, NE, SW, SE.
///
/// Tile positions are bounded between 0 and u32::MAX, so None may be returned
pub fn get_neighboring_pos(tile_pos: TilePos) -> [Option<TilePos>; 8] {
    let north = (tile_pos.1 != u32::MAX).then(|| TilePos(tile_pos.0, tile_pos.1 + 1));
    let south = (tile_pos.1 != 0).then(|| TilePos(tile_pos.0, tile_pos.1 - 1));
    let west = (tile_pos.0 != 0).then(|| TilePos(tile_pos.0 - 1, tile_pos.1));
    let east = (tile_pos.0 != u32::MAX).then(|| TilePos(tile_pos.0 + 1, tile_pos.1));
    let northwest = ((tile_pos.0 != 0) & (tile_pos.1 != u32::MAX))
        .then(|| TilePos(tile_pos.0 - 1, tile_pos.1 + 1));
    let northeast = ((tile_pos.0 != u32::MAX) & (tile_pos.1 != u32::MAX))
        .then(|| TilePos(tile_pos.0 + 1, tile_pos.1 + 1));
    let southwest =
        ((tile_pos.0 != 0) & (tile_pos.1 != 0)).then(|| TilePos(tile_pos.0 - 1, tile_pos.1 - 1));
    let southeast = ((tile_pos.0 != u32::MAX) & (tile_pos.1 != 0))
        .then(|| TilePos(tile_pos.0 + 1, tile_pos.1 - 1));
    [
        north, south, west, east, northwest, northeast, southwest, southeast,
    ]
}
