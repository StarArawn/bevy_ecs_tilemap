use std::collections::{VecDeque};
use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, pipeline::{PrimitiveTopology}}};
use crate::{TilemapMeshType, chunk::{Chunk, ChunkBundle, RemeshChunk}, morton_index, prelude::{SquareChunkMesher, Tile, TilemapChunkMesher}, render::TilemapData, tile::RemoveTile};

pub(crate) struct SetTileEvent {
    pub entity: Entity,
    pub tile: Tile,
}

/// A bevy bundle which contains: Map, Transform, and GlobalTransform components.
#[derive(Bundle, Default)]
pub struct MapBundle {
    /// The map component for the tilemap.
    pub map: Map,
    /// The local transform of the tilemap entity.
    pub transform: Transform,
    /// The global transform of the tilemap entity.
    pub global_transform: GlobalTransform,
}

/// Various settings used to define the tilemap.
pub struct MapSettings {
    /// Size of the tilemap in chunks 
    pub map_size: UVec2,
    /// Size in tiles of each chunk.
    pub chunk_size: UVec2,
    /// Size in pixels of each tile.
    pub tile_size: Vec2,
    /// Size in pixels of the tilemap texture.
    pub texture_size: Vec2,
    /// The layer id associated with this map.
    pub layer_id: u32,
    /// The meshing algorithm used for the tilemap.
    pub mesh_type: TilemapMeshType,
    pub(crate) mesher: Box<dyn TilemapChunkMesher>,
}

impl MapSettings {
    pub fn new(map_size: UVec2, chunk_size: UVec2, tile_size: Vec2, texture_size: Vec2, layer_id: u32) -> Self {
        Self {
            map_size,
            chunk_size,
            tile_size,
            texture_size,
            layer_id,
            mesh_type: TilemapMeshType::Square,
            mesher: Box::new(SquareChunkMesher)
        }
    }
}

impl Clone for MapSettings {
    fn clone(&self) -> Self {
        Self {
            map_size: self.map_size,
            chunk_size: self.chunk_size,
            tile_size: self.tile_size,
            texture_size: self.texture_size,
            layer_id: self.layer_id,
            mesh_type: self.mesh_type,
            mesher: dyn_clone::clone_box(&*self.mesher),
        }
    }
}

/// A component which keeps information and a cache of tile/chunk entities for convenience.
pub struct Map {
    /// The map information for the tilemap entity.
    pub settings: MapSettings,
    chunks: Vec<Option<Entity>>,
    tiles: Vec<Option<Entity>>,
    events: VecDeque<SetTileEvent>
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
                mesher: Box::new(SquareChunkMesher),
            },
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
}

impl Map {
    /// Creates a new map component.
    /// 
    /// - `settings`: The map settings struct.
    pub fn new(settings: MapSettings) -> Self {
        let map_size_x = (1 << (settings.map_size.x as f32).log2().ceil() as i32) as usize;
        let map_size_y = (1 << (settings.map_size.y as f32).log2().ceil() as i32) as usize;
        let map_size = map_size_x * map_size_y;
        let tile_size_x = (1 << ((settings.map_size.x * settings.chunk_size.x) as f32).log2().ceil() as i32) as usize;
        let tile_size_y = (1 << ((settings.map_size.y * settings.chunk_size.y) as f32).log2().ceil() as i32) as usize;
        let tile_count = tile_size_x * tile_size_y;
        Self {
            settings,
            chunks: vec![None; map_size],
            tiles: vec![None; tile_count],
            events: VecDeque::new(),
        }
    }

    /// Tags the tile for removal
    /// - `commands`: Bevy's command buffer.
    /// - `tile_pos`: A `UVec2` of where the tile to remove in tilemap coords.
    ///
    /// You can also remove tile entities manually by doing:
    /// ```
    /// commands.entity(tile_entity).insert(RemoveTile)
    /// ```
    pub fn remove_tile(&self, commands: &mut Commands, tile_pos: UVec2) {
        if let Some(tile_entity) = self.get_tile(IVec2::new(tile_pos.x as i32, tile_pos.y as i32)) {
            commands.entity(tile_entity).insert(RemoveTile);
        }
    }

    /// Adds a new tile to the map if the tile already exists this code will do nothing.
    ///
    /// Returning `MapTileError` when the tile position is outside of teh tile map bounds.
    /// - `commands`: Bevy's command buffer.
    /// - `tile_pos`: A `UVec2` of where the tile to remove in tilemap coords.
    /// - `tile`: The tile component data.
    /// - `visible`: A boolean which if true will add the [`crate::VisibleTile`] tag.
    pub fn add_tile(&mut self, commands: &mut Commands, tile_pos: UVec2, tile: Tile, visible: bool) -> Result<Entity, MapTileError> {
        // First find chunk tile should live in:
        let mut possible_tile_event = None;

        let chunk_pos = UVec2::new(
            tile_pos.x / self.settings.chunk_size.x,
            tile_pos.y / self.settings.chunk_size.y
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
            return Ok(tile_entity)
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
            position.y / self.settings.chunk_size.y
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
    pub fn get_tiles<T: IntoIterator<Item = IVec2>>(&self, tile_positions: T) -> Vec<Option<Entity>> {
        tile_positions.into_iter().map(|pos| self.get_tile(pos)).collect()
    }

    /// Retrieves a tile entity from the map. None will be returned for tiles that don't exist.
    /// - `tile_pos` - A position in the tilemap.
    pub fn get_tile(&self, tile_pos: IVec2) -> Option<Entity> {
        let map_size = &self.get_map_size_in_tiles();
        if tile_pos.x >= 0 && tile_pos.x <= map_size.x as i32 && tile_pos.y >= 0 && tile_pos.y <= map_size.y as i32 {
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
    }

    /// Builds the map's chunks and tiles if populate_chunks is true. It's important to call this function after creating a tilemap with `new`.
    /// - `commands`: Bevy's command buffer.
    /// - `meshes`: The bevy mesh asset resource.
    /// - `material`: The bevy color material asset resource.
    /// - `map_entity`: The entity attached to the map component. Chunks are children of this entity.
    /// - `populate_chunks`: Creates tile components for each tile in the tilemap. Note: This makes it difficult to change a tile in the same system that runs `build`.
    ///
    /// Note: This should always be called right after creating a map.
    /// 
    pub fn build(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<ColorMaterial>,
        map_entity: Entity,
        populate_chunks: bool,
    ) {
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
                let mesh_handle =  meshes.add(mesh);
                let mut chunk = Chunk::new(map_entity, chunk_pos, self.settings.chunk_size, self.settings.tile_size, self.settings.texture_size, mesh_handle.clone(), self.settings.layer_id, self.settings.mesh_type, dyn_clone::clone_box(&*self.settings.mesher));

                if populate_chunks {
                    chunk.build_tiles(commands, chunk_entity, &mut self.tiles, &mut |_| {
                        Tile::default()
                    });
                }

                let index = morton_index(chunk_pos);
                self.chunks[index] = Some(chunk_entity);

                let transform = Transform::from_xyz(
                    chunk_pos.x as f32 * self.settings.chunk_size.x as f32 * self.settings.tile_size.x,
                    chunk_pos.y as f32 * self.settings.chunk_size.y as f32 * self.settings.tile_size.y,
                    0.0
                );

                let tilemap_data = TilemapData::from(&chunk.settings);

                commands.entity(chunk_entity)
                    .insert_bundle(ChunkBundle {
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
    }
    
    pub fn build_iter<F>(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<ColorMaterial>,
        map_entity: Entity,
        mut f: F,
    ) where F: FnMut(UVec2) -> Tile {
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
                let mesh_handle =  meshes.add(mesh);
                let mut chunk = Chunk::new(map_entity, chunk_pos, self.settings.chunk_size, self.settings.tile_size, self.settings.texture_size, mesh_handle.clone(), self.settings.layer_id, self.settings.mesh_type, dyn_clone::clone_box(&*self.settings.mesher));

                chunk.build_tiles(commands, chunk_entity, &mut self.tiles, |p| f(p));

                let index = morton_index(chunk_pos);
                self.chunks[index] = Some(chunk_entity);

                let transform = Transform::from_xyz(
                    chunk_pos.x as f32 * self.settings.chunk_size.x as f32 * self.settings.tile_size.x,
                    chunk_pos.y as f32 * self.settings.chunk_size.y as f32 * self.settings.tile_size.y,
                    0.0
                );

                let tilemap_data = TilemapData::from(&chunk.settings);

                commands.entity(chunk_entity)
                    .insert_bundle(ChunkBundle {
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
    }
}

// Adds new tiles to the chunk hash map.
pub(crate) fn update_chunk_hashmap_for_added_tiles(
    mut chunk_query: Query<&mut Chunk>,
    tile_query: Query<(Entity, &Tile, &UVec2), Added<Tile>>,
) {
    if tile_query.iter().count() > 0 {
        log::info!("Updating tile cache.");
    }
    for (tile_entity, new_tile, tile_pos) in tile_query.iter() {
        if let Ok(mut chunk) = chunk_query.get_mut(new_tile.chunk) {
            let tile_pos = chunk.to_chunk_pos(*tile_pos);
            chunk.tiles[morton_index(tile_pos)] = Some(tile_entity);
        }
    }
}

// Removes tiles that have been removed from the chunk.
pub(crate) fn update_chunk_hashmap_for_removed_tiles(
    mut commands: Commands,
    mut map_query: Query<&mut Map>,
    mut chunk_query: Query<&mut Chunk>,
    removed_tiles_query: Query<(Entity, &Tile, &UVec2), With<RemoveTile>>
) {
    // For now only loop over 1 map.
    if let Some(mut map) = map_query.iter_mut().next() {
        for (tile_entity, removed_tile, tile_pos) in removed_tiles_query.iter() {

            // Remove tile from chunk tiles cache.
            if let Ok(mut tile_chunk) = chunk_query.get_mut(removed_tile.chunk) {
                let tile_pos = tile_chunk.to_chunk_pos(*tile_pos);
                tile_chunk.tiles[morton_index(tile_pos)] = None;
            }

            // Remove tile from map tiles cache.
            map.tiles[morton_index(*tile_pos)] = None;

            // Remove tile entity
            commands.entity(tile_entity).despawn_recursive();
            commands.entity(removed_tile.chunk).insert(RemeshChunk);
        }
    }
}

pub(crate) fn update_tiles(
    mut commands: Commands,
    mut map: Query<&mut Map>,
) {

    for mut map in map.iter_mut() {
        while let Some(event) = map.events.pop_front() {
            commands.entity(event.tile.chunk).insert(RemeshChunk);
            commands.entity(event.entity).remove::<Tile>().insert(event.tile);
        }
    }

}