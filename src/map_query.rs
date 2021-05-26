use crate::map::Map;
use crate::{morton_index, prelude::*};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// MapQuery is a useful bevy system param that provides a standard API for interacting with tiles.
/// It's not required that you use this, but it does provide a convenience.
/// Note: MapQuery doesn't directly change tile components. This is meant as a feature as you may
/// have your own tile data attached to each tile and a standard tile query wouldn't pull that data in.
#[derive(SystemParam)]
pub struct MapQuery<'a> {
    chunk_query_set: QuerySet<(
        Query<'a, (Entity, &'static mut Chunk)>,
        Query<'a, (Entity, &'static Chunk)>,
    )>,
    layer_query_set: QuerySet<(
        Query<'a, (Entity, &'static mut Layer)>,
        Query<'a, (Entity, &'static Layer)>,
    )>,
    map_query_set: QuerySet<(
        Query<'a, (Entity, &'static mut Map)>,
        Query<'a, (Entity, &'static Map)>,
    )>,
    meshes: ResMut<'a, Assets<Mesh>>,
}

impl<'a> MapQuery<'a> {
    /// Builds the tile map layer and returns the layer's entity.
    pub fn build_layer(
        &mut self,
        commands: &mut Commands,
        mut layer_builder: LayerBuilder<impl TileBundleTrait>,
        material_handle: Handle<ColorMaterial>,
    ) -> Entity {
        let layer_bundle = layer_builder.build(commands, &mut self.meshes, material_handle);
        let mut layer = layer_bundle.layer;
        let mut transform = layer_bundle.transform;
        layer.settings.layer_id = layer.settings.layer_id;
        transform.translation.z = layer.settings.layer_id as f32;
        commands
            .entity(layer_builder.layer_entity)
            .insert_bundle(LayerBundle {
                layer,
                transform,
                ..layer_bundle
            });
        layer_builder.layer_entity
    }

    /// Adds or sets a new tile for a given layer.
    /// Returns an error if the tile is out of bounds.
    /// It's important to know that the new tile wont exist until bevy flushes
    /// the commands during a hard sync point(between stages).
    /// A better option for updating existing tiles would be the following:
    /// ```rust
    /// ...
    /// mut my_tile_query: Query<&mut Tile>,
    /// mut map_query: MapQuery,
    /// ...
    ///
    /// let tile_entity = map_query.get_tile_entity(tile_position, 0); // Zero represents layer_id.
    /// if let Ok(mut tile) = my_tile_query.get_mut(tile_entity) {
    ///   tile.texture_index = 10;
    /// }
    /// ```
    pub fn set_tile<T: Into<u16> + Copy>(
        &mut self,
        commands: &mut Commands,
        tile_pos: UVec2,
        tile: Tile,
        map_id: T,
        layer_id: T,
    ) -> Result<Entity, MapTileError> {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    let chunk_pos = UVec2::new(
                        tile_pos.x / layer.settings.chunk_size.x,
                        tile_pos.y / layer.settings.chunk_size.y,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) =
                            self.chunk_query_set.q0_mut().get_mut(chunk_entity)
                        {
                            let chunk_local_tile_pos = chunk.to_chunk_pos(tile_pos);

                            // If the tile exists throw error.
                            if let Some(existing) = chunk.tiles[morton_index(chunk_local_tile_pos)]
                            {
                                commands.entity(existing).despawn_recursive();
                            }

                            let mut tile_commands = commands.spawn();
                            tile_commands
                                .insert(tile)
                                .insert(TileParent {
                                    chunk: chunk_entity,
                                    layer_id,
                                    map_id: layer.settings.map_id,
                                })
                                .insert(tile_pos);
                            let tile_entity = tile_commands.id();
                            chunk.tiles[morton_index(chunk_local_tile_pos)] = Some(tile_entity);
                            return Ok(tile_entity);
                        }
                    }
                }
            }
        }
        Err(MapTileError::OutOfBounds)
    }

    pub fn get_layer<I: Into<u16>>(&self, map_id: I, layer_id: I) -> Option<(Entity, &Layer)> {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((entity, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    return Some((entity, layer));
                }
            }
        }

        None
    }

    /// Gets a tile entity for the given position and layer_id returns an error if OOB or the tile doesn't exist.
    pub fn get_tile_entity<T: Into<u16>>(
        &self,
        tile_pos: UVec2,
        map_id: T,
        layer_id: T,
    ) -> Result<Entity, MapTileError> {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    let chunk_pos = UVec2::new(
                        tile_pos.x / layer.settings.chunk_size.x,
                        tile_pos.y / layer.settings.chunk_size.y,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, chunk)) = self.chunk_query_set.q1().get(chunk_entity) {
                            if let Some(tile) = chunk.get_tile_entity(chunk.to_chunk_pos(tile_pos))
                            {
                                return Ok(tile);
                            } else {
                                return Err(MapTileError::NonExistent);
                            }
                        }
                    }
                }
            }
        }

        Err(MapTileError::OutOfBounds)
    }

    /// Despawns the tile entity and removes it from the layer/chunk cache.
    pub fn despawn_tile<T: Into<u16>>(
        &mut self,
        commands: &mut Commands,
        tile_pos: UVec2,
        map_id: T,
        layer_id: T,
    ) -> Result<(), MapTileError> {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    let chunk_pos = UVec2::new(
                        tile_pos.x / layer.settings.chunk_size.x,
                        tile_pos.y / layer.settings.chunk_size.y,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) =
                            self.chunk_query_set.q0_mut().get_mut(chunk_entity)
                        {
                            let chunk_tile_pos = chunk.to_chunk_pos(tile_pos);
                            if let Some(tile) = chunk.get_tile_entity(chunk_tile_pos) {
                                commands.entity(tile).despawn_recursive();
                                let morton_tile_index = morton_index(chunk_tile_pos);
                                chunk.tiles[morton_tile_index] = None;
                                return Ok(());
                            } else {
                                return Err(MapTileError::NonExistent);
                            }
                        }
                    }
                }
            }
        }
        Err(MapTileError::OutOfBounds)
    }

    /// Depsawns all of the tiles in a layer.
    /// Note: Doesn't despawn the layer.
    pub fn despawn_layer_tiles<T: Into<u16>>(
        &mut self,
        commands: &mut Commands,
        map_id: T,
        layer_id: T,
    ) {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    for x in 0..layer.get_layer_size_in_tiles().x {
                        for y in 0..layer.get_layer_size_in_tiles().y {
                            let tile_pos = UVec2::new(x, y);
                            let chunk_pos = UVec2::new(
                                tile_pos.x / layer.settings.chunk_size.x,
                                tile_pos.y / layer.settings.chunk_size.y,
                            );
                            if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                                if let Ok((_, mut chunk)) =
                                    self.chunk_query_set.q0_mut().get_mut(chunk_entity)
                                {
                                    let chunk_tile_pos = chunk.to_chunk_pos(tile_pos);
                                    if let Some(tile) = chunk.get_tile_entity(chunk_tile_pos) {
                                        commands.entity(tile).despawn_recursive();
                                        let morton_tile_index = morton_index(chunk_tile_pos);
                                        chunk.tiles[morton_tile_index] = None;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Despawns a layer completely including all tiles.
    pub fn depsawn_layer<T: Into<u16>>(&mut self, commands: &mut Commands, map_id: T, layer_id: T) {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        self.despawn_layer_tiles(commands, map_id, layer_id);
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    for x in 0..layer.settings.map_size.x {
                        for y in 0..layer.settings.map_size.y {
                            if let Some(chunk_entity) = layer.get_chunk(UVec2::new(x, y)) {
                                commands.entity(chunk_entity).despawn_recursive();
                            }
                        }
                    }
                }
                commands.entity(*layer_entity).despawn_recursive();
            }
        }
    }

    /// Retrieves a list of neighbor entities in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    /// The returned neighbors are tuples that have an tilemap coordinate and an Option<Entity>.
    ///
    /// A value of None will be returned for tiles that don't exist.
    ///
    /// ## Example
    ///
    /// ```
    /// let neighbors = map.get_tile_neighbors(UVec2::new(0, 0));
    /// assert!(neighbors[1].1.is_none()); // Outside of tile bounds.
    /// assert!(neighbors[0].1.is_none()); // Entity returned inside bounds.
    /// ```
    pub fn get_tile_neighbors<T: Into<u16>>(
        &self,
        tile_pos: UVec2,
        map_id: T,
        layer_id: T,
    ) -> [(IVec2, Option<Entity>); 8] {
        let n = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 + 1);
        let s = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 - 1);
        let w = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32);
        let e = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32);
        let nw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 + 1);
        let ne = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 + 1);
        let sw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 - 1);
        let se = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 - 1);
        let layer_id: u16 = layer_id.into();
        let map_id: u16 = map_id.into();
        [
            (n, self.get_tile_i(n, map_id, layer_id)),
            (s, self.get_tile_i(s, map_id, layer_id)),
            (w, self.get_tile_i(w, map_id, layer_id)),
            (e, self.get_tile_i(e, map_id, layer_id)),
            (nw, self.get_tile_i(nw, map_id, layer_id)),
            (ne, self.get_tile_i(ne, map_id, layer_id)),
            (sw, self.get_tile_i(sw, map_id, layer_id)),
            (se, self.get_tile_i(se, map_id, layer_id)),
        ]
    }

    fn get_tile_i(&self, tile_pos: IVec2, map_id: u16, layer_id: u16) -> Option<Entity> {
        if tile_pos.x < 0 || tile_pos.y < 0 {
            return None;
        }
        let tile_pos = tile_pos.as_u32();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    let chunk_pos = UVec2::new(
                        tile_pos.x / layer.settings.chunk_size.x,
                        tile_pos.y / layer.settings.chunk_size.y,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, chunk)) = self.chunk_query_set.q1().get(chunk_entity) {
                            if let Some(tile) = chunk.get_tile_entity(chunk.to_chunk_pos(tile_pos))
                            {
                                return Some(tile);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Let's the internal systems know to "remesh" the chunk.
    pub fn notify_chunk(&mut self, chunk_entity: Entity) {
        if let Ok((_, mut chunk)) = self.chunk_query_set.q0_mut().get_mut(chunk_entity) {
            chunk.needs_remesh = true;
        }
    }

    /// Let's the internal systems know to remesh the chunk for a given tile pos and layer_id.
    pub fn notify_chunk_for_tile<T: Into<u16>>(&mut self, tile_pos: UVec2, map_id: T, layer_id: T) {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    let chunk_pos = UVec2::new(
                        tile_pos.x / layer.settings.chunk_size.x,
                        tile_pos.y / layer.settings.chunk_size.y,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) =
                            self.chunk_query_set.q0_mut().get_mut(chunk_entity)
                        {
                            chunk.needs_remesh = true;
                        }
                    }
                }
            }
        }
    }
}
