use crate::layer::MapTileError;
use crate::layer_builder::LayerBuilder;
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

impl<'a> MapQuery<'a> {
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
    pub fn get_tile_neighbors<M: Into<u16> + Copy, L: Into<u16> + Copy>(
        &self,
        tile_pos: TilePos,
        map_id: M,
        layer_id: L,
    ) -> Vec<Result<Entity, MapTileError>> {
        let neighboring_tile_pos = get_neighboring_pos(tile_pos);

        neighboring_tile_pos
            .iter()
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
    let north = if tile_pos.1 != u32::MAX {
        Some(TilePos(tile_pos.0, tile_pos.1 + 1))
    } else {
        None
    };

    let south = if tile_pos.1 != 0 {
        Some(TilePos(tile_pos.0, tile_pos.1 - 1))
    } else {
        None
    };

    let west = if tile_pos.0 != 0 {
        Some(TilePos(tile_pos.0 - 1, tile_pos.1))
    } else {
        None
    };

    let east = if tile_pos.0 != u32::MAX {
        Some(TilePos(tile_pos.0 + 1, tile_pos.1))
    } else {
        None
    };

    let northwest = if (tile_pos.0 != 0) & (tile_pos.1 != u32::MAX) {
        Some(TilePos(tile_pos.0 - 1, tile_pos.1 + 1))
    } else {
        None
    };

    let northeast = if (tile_pos.0 != u32::MAX) & (tile_pos.1 != u32::MAX) {
        Some(TilePos(tile_pos.0 + 1, tile_pos.1 + 1))
    } else {
        None
    };

    let southwest = if (tile_pos.0 != 0) & (tile_pos.1 != 0) {
        Some(TilePos(tile_pos.0 - 1, tile_pos.1 - 1))
    } else {
        None
    };

    let southeast = if (tile_pos.0 != u32::MAX) & (tile_pos.1 != 0) {
        Some(TilePos(tile_pos.0 + 1, tile_pos.1 - 1))
    } else {
        None
    };

    [
        north, south, west, east, northwest, northeast, southwest, southeast,
    ]
}
