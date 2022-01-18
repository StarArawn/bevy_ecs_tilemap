use crate::layer::LayerId;
use crate::map::Map;
use crate::{morton_index, prelude::*};
use bevy::ecs::system::SystemParam;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;

/// MapQuery is a useful bevy system param that provides a standard API for interacting with tiles.
/// It's not required that you use this, but it does provide a convenience.
/// Note: MapQuery doesn't directly change tile components. This is meant as a feature as you may
/// have your own tile data attached to each tile and a standard tile query wouldn't pull that data in.
#[derive(SystemParam)]
pub struct MapQuery<'w, 's> {
    chunk_query_set: QuerySet<
        'w,
        's,
        (
            QueryState<(Entity, &'static mut Chunk)>,
            QueryState<(Entity, &'static Chunk)>,
        ),
    >,
    layer_query_set: QuerySet<
        'w,
        's,
        (
            QueryState<(Entity, &'static mut Layer)>,
            QueryState<(Entity, &'static Layer)>,
        ),
    >,
    map_query_set: QuerySet<
        'w,
        's,
        (
            QueryState<(Entity, &'static mut Map)>,
            QueryState<(Entity, &'static Map)>,
        ),
    >,
    meshes: ResMut<'w, Assets<Mesh>>,
}

impl<'w, 's> MapQuery<'w, 's> {
    /// Builds the tile map layer and returns the layer's entity.
    pub fn build_layer(
        &mut self,
        commands: &mut Commands,
        mut layer_builder: LayerBuilder<impl TileBundleTrait>,
        material_handle: Handle<Image>,
    ) -> Entity {
        let layer_bundle = layer_builder.build(commands, &mut self.meshes, material_handle);
        let layer = layer_bundle.layer;
        let mut transform = layer_bundle.transform;
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
    pub fn set_tile(
        &mut self,
        commands: &mut Commands,
        tile_pos: TilePos,
        tile: Tile,
        map_id: impl MapId,
        layer_id: impl LayerId,
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
                    let chunk_pos = ChunkPos(
                        tile_pos.0 / layer.settings.chunk_size.0,
                        tile_pos.1 / layer.settings.chunk_size.1,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) = self.chunk_query_set.q0().get_mut(chunk_entity)
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

    pub fn get_layer(
        &mut self,
        map_id: impl MapId,
        layer_id: impl LayerId,
    ) -> Option<(Entity, &Layer)> {
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
    pub fn get_tile_entity(
        &mut self,
        tile_pos: TilePos,
        map_id: impl MapId,
        layer_id: impl LayerId,
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
                    let chunk_pos = ChunkPos(
                        tile_pos.0 / layer.settings.chunk_size.0,
                        tile_pos.1 / layer.settings.chunk_size.1,
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

    pub fn update_chunk<F: FnMut(Mut<Chunk>)>(&mut self, chunk_entity: Entity, mut f: F) {
        if let Ok((_, chunk)) = self.chunk_query_set.q0().get_mut(chunk_entity) {
            f(chunk);
        }
    }

    /// Despawns the tile entity and removes it from the layer/chunk cache.
    pub fn despawn_tile(
        &mut self,
        commands: &mut Commands,
        tile_pos: TilePos,
        map_id: impl MapId,
        layer_id: impl LayerId,
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
                    let chunk_pos = ChunkPos(
                        tile_pos.0 / layer.settings.chunk_size.0,
                        tile_pos.1 / layer.settings.chunk_size.1,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) = self.chunk_query_set.q0().get_mut(chunk_entity)
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

    /// Despawns all of the tiles in a layer.
    /// Note: Doesn't despawn the layer.
    pub fn despawn_layer_tiles(
        &mut self,
        commands: &mut Commands,
        map_id: impl MapId,
        layer_id: impl LayerId,
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
                    for x in 0..layer.get_layer_size_in_tiles().0 {
                        for y in 0..layer.get_layer_size_in_tiles().1 {
                            let tile_pos = TilePos(x, y);
                            let chunk_pos = ChunkPos(
                                tile_pos.0 / layer.settings.chunk_size.0,
                                tile_pos.1 / layer.settings.chunk_size.1,
                            );
                            if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                                if let Ok((_, mut chunk)) =
                                    self.chunk_query_set.q0().get_mut(chunk_entity)
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
    pub fn despawn_layer(
        &mut self,
        commands: &mut Commands,
        map_id: impl MapId,
        layer_id: impl LayerId,
    ) {
        let map_id = map_id.into();
        let layer_id = layer_id.into();
        self.despawn_layer_tiles(commands, map_id, layer_id);
        if let Some((_, mut map)) = self
            .map_query_set
            .q0()
            .iter_mut()
            .find(|(_, map)| map.id == map_id)
        {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = self.layer_query_set.q1().get(*layer_entity) {
                    for x in 0..layer.settings.map_size.0 {
                        for y in 0..layer.settings.map_size.1 {
                            if let Some(chunk_entity) = layer.get_chunk(ChunkPos(x, y)) {
                                commands.entity(chunk_entity).despawn_recursive();
                            }
                        }
                    }
                }
                commands.entity(*layer_entity).despawn_recursive();
            }
            map.remove_layer(commands, layer_id);
        }
    }

    /// Despawn an entire map including all layers/tiles.
    pub fn despawn(&mut self, commands: &mut Commands, map_id: impl MapId) {
        let map_id: u16 = map_id.into();

        let layer_ids: Option<Vec<u16>> = if let Some((_, map)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            Some(map.layers.keys().copied().collect())
        } else {
            None
        };

        if let Some(layer_ids) = layer_ids {
            for layer_id in layer_ids.iter() {
                self.despawn_layer(commands, map_id, *layer_id);
            }
        }

        if let Some((entity, _)) = self
            .map_query_set
            .q1()
            .iter()
            .find(|(_, map)| map.id == map_id)
        {
            commands.entity(entity).despawn_recursive();
        }
    }

    /// Let's the internal systems know to "remesh" the chunk.
    pub fn notify_chunk(&mut self, chunk_entity: Entity) {
        if let Ok((_, mut chunk)) = self.chunk_query_set.q0().get_mut(chunk_entity) {
            chunk.needs_remesh = true;
        }
    }

    /// Let's the internal systems know to remesh the chunk for a given tile pos and layer_id.
    pub fn notify_chunk_for_tile<M: Into<u16>, L: Into<u16>>(
        &mut self,
        tile_pos: TilePos,
        map_id: M,
        layer_id: L,
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
                    let chunk_pos = ChunkPos(
                        tile_pos.0 / layer.settings.chunk_size.0,
                        tile_pos.1 / layer.settings.chunk_size.1,
                    );
                    if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                        if let Ok((_, mut chunk)) = self.chunk_query_set.q0().get_mut(chunk_entity)
                        {
                            chunk.needs_remesh = true;
                        }
                    }
                }
            }
        }
    }

    /// Gets the tiles z position for a given pixel position.
    /// This is a bit difficult to explain, but for isometric rendering this
    /// allows you to get a z position within the 2D isometric tilemap.
    /// Z positions are calculated as follows:
    /// 1. Snap pixel position to tile coords.
    /// 2. Use tile Y position to calculate z-index.
    /// 3. Z-index is scaled to be 0-1.
    /// 4. Add expected layer id to Z-index
    /// Note the layer_id in this case is past in by the user as pixel_position.z
    /// The user needs to handle what layer the sprite exists in within their own code.
    /// The primary use case for this function is to allow users to calculate
    /// a sprites z-index so it appears correctly either behind or in front
    /// of a given isometric tile. To see an example of this checkout:
    /// `examples/helpers/movement.rs`
    pub fn get_zindex_for_pixel_pos<M: Into<u16>, L: Into<u16>>(
        &mut self,
        pixel_position: Vec3,
        map_id: M,
        layer_id: L,
    ) -> f32 {
        let map_query = self.map_query_set.q1();
        let layer_query = self.layer_query_set.q1();

        let map_id = map_id.into();
        let layer_id = layer_id.into();
        if let Some((_, map)) = map_query.iter().find(|(_, map)| map.id == map_id) {
            if let Some(layer_entity) = map.get_layer_entity(layer_id) {
                if let Ok((_, layer)) = layer_query.get(*layer_entity) {
                    let grid_size = layer.settings.grid_size;
                    let layer_size_in_tiles: Vec2 = layer.get_layer_size_in_tiles().into();
                    let map_size: Vec2 = layer_size_in_tiles * grid_size;
                    let map_pos = unproject_iso(pixel_position.xy(), grid_size.x, grid_size.y);
                    let center = project_iso(
                        Vec2::new(map_pos.x, map_pos.y - 2.0),
                        grid_size.x,
                        grid_size.y,
                    );
                    dbg!(grid_size, layer_size_in_tiles, map_size, map_pos, center);

                    return pixel_position.z + (1.0 - (center.y / map_size.y));
                }
            }
        }

        0.0
    }
}

pub fn unproject_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let half_width = tile_width / 2.0;
    let half_height = tile_height / 2.0;
    let x = ((pos.x / half_width) + (-(pos.y) / half_height)) / 2.0;
    let y = ((-(pos.y) / half_height) - (pos.x / half_width)) / 2.0;
    Vec2::new(x.round(), y.round())
}

fn project_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    Vec2::new(x, -y)
}
