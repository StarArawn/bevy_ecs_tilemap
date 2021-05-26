use bevy::prelude::*;
use std::{collections::HashMap, vec::IntoIter};

/// A simple component used to keep track of layer entities.
#[derive(Clone)]
pub struct Map {
<<<<<<< HEAD
    pub map_entity: Entity,
    pub id: u16,
    pub(crate) layers: HashMap<u16, Entity>,
=======
    /// The map information for the tilemap entity.
    pub settings: MapSettings,
    map_entity: Option<Entity>,
    chunks: Vec<Option<Entity>>,
    tiles: Vec<Option<Entity>>,
    events: VecDeque<SetTileEvent>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            settings: MapSettings {
                map_size: UVec2::default(),
                chunk_size: UVec2::default(),
                tile_size: Vec2::default(),
                texture_size: Vec2::default(),
                layer_id: 0,
                mesh_type: TilemapMeshType::Square,
                tile_spacing: Vec2::default(),
                cull: true,
                mesher: Box::new(SquareChunkMesher),
            },
            map_entity: None,
            chunks: Vec::new(),
            tiles: Vec::new(),
            events: VecDeque::new(),
        }
    }
}

/// General errors that are returned by bevy_ecs_tilemap.
#[derive(Debug, Copy, Clone)]
pub enum MapTileError {
    /// A tile you attempted to grab does not exist.
    OutOfBounds,
>>>>>>> main
}

impl Map {
    /// Creates a new map component
    pub fn new<T: Into<u16>>(id: T, map_entity: Entity) -> Self {
        Self {
<<<<<<< HEAD
            map_entity,
            id: id.into(),
            layers: HashMap::new(),
=======
            settings,
            map_entity: None,
            chunks: vec![None; map_size],
            tiles: vec![None; tile_count],
            events: VecDeque::new(),
>>>>>>> main
        }
    }

    /// Creates a new layer.
    pub fn add_layer<T: Into<u16>>(
        &mut self,
        commands: &mut Commands,
<<<<<<< HEAD
        layer_id: T,
        layer_entity: Entity,
    ) {
        commands
            .entity(self.map_entity)
            .push_children(&[layer_entity]);
        self.layers.insert(layer_id.into(), layer_entity);
=======
        tile_pos: UVec2,
        tile: Tile,
        visible: bool,
    ) -> Result<Entity, MapTileError> {
        // First find chunk tile should live in:
        let mut possible_tile_event = None;

        let chunk_pos = UVec2::new(
            tile_pos.x / self.settings.chunk_size.x,
            tile_pos.y / self.settings.chunk_size.y,
        );
        if let Some(chunk) = self.get_chunk(chunk_pos) {
            if let Some(tile_entity) = self.tiles[morton_index(tile_pos)] {
                // If the tile already exists we need to queue an event.
                // We do this because we have good way of changing the actual tile data from here.
                // Another possibility is having the user pass in a query of tiles and matching based off of entity,
                // however the tile may not exist yet in this current frame even though it exists in the Commands
                // buffer.
                possible_tile_event = Some(SetTileEvent {
                    entity: tile_entity,
                    tile: Tile {
                        chunk: chunk,
                        ..tile
                    },
                });
            } else {
                let mut tile_commands = commands.spawn();
                tile_commands
                    .insert(Parent(self.map_entity.unwrap()))
                    .insert(Tile {
                        chunk: chunk,
                        ..tile
                    })
                    .insert(tile_pos);
                if visible {
                    tile_commands.insert(crate::prelude::VisibleTile);
                }
                let tile_entity = tile_commands.id();
                self.tiles[morton_index(tile_pos)] = Some(tile_entity);

                return Ok(tile_entity);
            }
        }

        if let Some(event) = possible_tile_event {
            let tile_entity = event.entity;
            self.events.push_back(event);
            return Ok(tile_entity);
        }

        Err(MapTileError::OutOfBounds)
    }

    fn get_chunk(&self, chunk_pos: UVec2) -> Option<Entity> {
        self.chunks[morton_index(chunk_pos)]
    }

    /// Notify the map renderer that the tile is updated and to mesh the chunk again.
    /// - `commands`: Bevy's command buffer.
    /// - `tile_pos`: A `UVec2` of where the tile to remove in tilemap coords.
    ///
    /// Note: This just adds a Tag to the chunk entity to be re-meshed.
    pub fn notify(&self, commands: &mut Commands, position: UVec2) {
        if let Some(chunk_entity) = self.get_chunk(UVec2::new(
            position.x / self.settings.chunk_size.x,
            position.y / self.settings.chunk_size.y,
        )) {
            commands.entity(chunk_entity).insert(RemeshChunk);
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
    pub fn get_tile_neighbors(&self, tile_pos: UVec2) -> [(IVec2, Option<Entity>); 8] {
        let n = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 + 1);
        let s = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 - 1);
        let w = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32);
        let e = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32);
        let nw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 + 1);
        let ne = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 + 1);
        let sw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 - 1);
        let se = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 - 1);
        [
            (n, self.get_tile(n)),
            (s, self.get_tile(s)),
            (w, self.get_tile(w)),
            (e, self.get_tile(e)),
            (nw, self.get_tile(nw)),
            (ne, self.get_tile(ne)),
            (sw, self.get_tile(sw)),
            (se, self.get_tile(se)),
        ]
    }

    /// Returns a list of tile entities base off of the positions requested
    ///
    /// None is returned for tiles that don't exist.
    /// - `tile_positions` An Iterator of tile positions in the tilemap.
    ///
    /// ## Example
    /// ```
    /// let tiles = map.get_tiles(vec![IVec2::new(0, 0), IVec2::new(0, 1)]);
    /// assert!(tiles.len() == 2);
    /// ```
    pub fn get_tiles<T: IntoIterator<Item = IVec2>>(
        &self,
        tile_positions: T,
    ) -> Vec<Option<Entity>> {
        tile_positions
            .into_iter()
            .map(|pos| self.get_tile(pos))
            .collect()
    }

    /// Retrieves a tile entity from the map. None will be returned for tiles that don't exist.
    /// - `tile_pos` - A position in the tilemap.
    pub fn get_tile(&self, tile_pos: IVec2) -> Option<Entity> {
        let map_size = &self.get_map_size_in_tiles();
        if tile_pos.x >= 0
            && tile_pos.x <= map_size.x as i32
            && tile_pos.y >= 0
            && tile_pos.y <= map_size.y as i32
        {
            return self.tiles[morton_index(UVec2::new(tile_pos.x as u32, tile_pos.y as u32))];
        } else {
            return None;
        }
    }

    /// Gets a list of all of the tile entities for the tilemap.
    /// Returns none for tiles that don't exist in the tilemap.
    pub fn get_all_tiles(&self) -> &Vec<Option<Entity>> {
        &self.tiles
    }

    /// Gets the map's size in tiles just for convenience.
    pub fn get_map_size_in_tiles(&self) -> UVec2 {
        UVec2::new(
            self.settings.map_size.x * self.settings.chunk_size.x,
            self.settings.map_size.y * self.settings.chunk_size.y,
        )
>>>>>>> main
    }

    /// Adds multiple layers to the map.
    pub fn add_layers<I: Into<u16>>(
        &mut self,
        commands: &mut Commands,
        layers: IntoIter<(I, Entity)>,
    ) {
<<<<<<< HEAD
        let layers: Vec<(u16, Entity)> = layers.map(|(id, entity)| (id.into(), entity)).collect();
        let entities: Vec<Entity> = layers.iter().map(|(_, entity)| *entity).collect();
        self.layers.extend(layers);
        commands.entity(self.map_entity).push_children(&entities);
=======
        self.map_entity = Some(map_entity);

        for x in 0..self.settings.map_size.x {
            for y in 0..self.settings.map_size.y {
                let mut chunk_entity = None;
                commands.entity(map_entity).with_children(|child_builder| {
                    chunk_entity = Some(child_builder.spawn().id());
                });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = UVec2::new(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(vec![]));
                mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Int4(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let mut chunk = Chunk::new(
                    map_entity,
                    chunk_pos,
                    self.settings.chunk_size,
                    self.settings.tile_size,
                    self.settings.texture_size,
                    self.settings.tile_spacing,
                    mesh_handle.clone(),
                    self.settings.layer_id,
                    self.settings.mesh_type,
                    dyn_clone::clone_box(&*self.settings.mesher),
                    self.settings.cull,
                );

                if populate_chunks {
                    chunk.build_tiles(commands, chunk_entity, &mut self.tiles, &mut |_| {
                        Tile::default()
                    });
                }

                let index = morton_index(chunk_pos);
                self.chunks[index] = Some(chunk_entity);

                let transform = Transform::from_xyz(
                    chunk_pos.x as f32
                        * self.settings.chunk_size.x as f32
                        * self.settings.tile_size.x,
                    chunk_pos.y as f32
                        * self.settings.chunk_size.y as f32
                        * self.settings.tile_size.y,
                    0.0,
                );

                let tilemap_data = TilemapData::from(&chunk.settings);

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    material: material.clone(),
                    transform,
                    tilemap_data,
                    render_pipeline: self.settings.mesh_type.into(),
                    ..Default::default()
                });
            }
        }
>>>>>>> main
    }

    /// Removes the layer from the map and despawns the layer entity.
    /// Note: Does not despawn the tile entities. Please use MapQuery instead.
    pub fn remove_layer<I: Into<u16>>(&mut self, commands: &mut Commands, layer_id: I) {
        if let Some(layer_entity) = self.layers.remove(&layer_id.into()) {
            commands.entity(layer_entity).despawn_recursive();
        }
    }

    /// Removes the layers from the map and despawns the layer entities.
    /// Note: Does not despawn the tile entities. Please use MapQuery instead.
    pub fn remove_layers<I: Into<u16>>(&mut self, commands: &mut Commands, layers: IntoIter<I>) {
        layers.for_each(|id| {
            let id: u16 = id.into();
            self.remove_layer(commands, id);
        });
    }

    /// Retrieves the entity for a given layer id.
    pub fn get_layer_entity<T: Into<u16>>(&self, layer_id: T) -> Option<&Entity> {
        self.layers.get(&layer_id.into())
    }

    /// Despawns a map. Better to call `map_query.despawn_map` as it will despawn layers/tiles as well.
    pub fn despawn(&self, commands: &mut Commands) {
        commands.entity(self.map_entity).despawn_recursive();
    }
}
