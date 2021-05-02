use crate::{
    chunk::{Chunk, ChunkBundle, RemeshChunk},
    map_vec2::MapVec2,
    prelude::{SquareChunkMesher, Tile, TilemapChunkMesher},
};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use std::collections::HashMap;

#[derive(Bundle, Default)]
pub struct MapBundle {
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct Map {
    pub map_settings: MapSettings,
    pub layer_id: u32,
    chunks: HashMap<MapVec2, (Entity, HashMap<MapVec2, Entity>)>,
    pub mesher: Box<dyn TilemapChunkMesher>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            map_settings: MapSettings::default(),
            layer_id: 0,
            chunks: HashMap::new(),
            mesher: Box::new(SquareChunkMesher),
        }
    }
}

///The settings used to create a [Map](Map)
///
///You could store this as a Resource for reusability
#[derive(Default, Debug, Clone, Copy)]
pub struct MapSettings {
    pub map_size: MapVec2,
    pub chunk_size: MapVec2,
    pub tile_size: Vec2,
    pub texture_size: Vec2,
}

#[derive(Debug, Copy, Clone)]
pub enum MapTileError {
    TileAlreadyExists,
    UnableToFindChunk,
}

/// A tag that allows you to remove a tile from the world;
pub struct RemoveTile;

impl Map {
    pub fn new(map_settings: MapSettings, layer_id: u32) -> Self {
        Self {
            map_settings,
            layer_id,
            chunks: HashMap::new(),
            mesher: Box::new(SquareChunkMesher),
        }
    }

    /// Tags the tile for removal
    /// You can also remove tile entities manually by doing:
    /// `commands.entity(tile_entity).insert(RemoveTile)`
    pub fn remove_tile(&self, commands: &mut Commands, tile_pos: MapVec2) {
        if let Some(tile_entity) = self.get_tile(tile_pos) {
            commands.entity(*tile_entity).insert(RemoveTile);
        }
    }

    /// Adds a new tile to the map if the tile already exists this code will do nothing.
    pub fn add_tile(
        &mut self,
        commands: &mut Commands,
        tile_pos: MapVec2,
        tile: Tile,
    ) -> Result<(), MapTileError> {
        // First find chunk tile should live in:
        if let Some(chunk_data) = self.get_chunk_mut(MapVec2::new(
            tile_pos.x / self.map_settings.chunk_size.x,
            tile_pos.y / self.map_settings.chunk_size.y,
        )) {
            if chunk_data.1.contains_key(&tile_pos) {
                return Err(MapTileError::TileAlreadyExists);
            }
            let tile_entity = commands
                .spawn()
                .insert(Tile {
                    chunk: chunk_data.0,
                    ..tile
                })
                .insert(tile_pos)
                .id();
            chunk_data.1.insert(tile_pos, tile_entity);

            return Ok(());
        }

        Err(MapTileError::UnableToFindChunk)
    }

    fn get_chunk_mut(
        &mut self,
        chunk_pos: MapVec2,
    ) -> Option<&mut (Entity, HashMap<MapVec2, Entity>)> {
        self.chunks.get_mut(&chunk_pos)
    }

    /// Retrieves a list of neighbor entities in the following order:
    /// N, S, W, E, NW, NE, SW, SE. None will be returned for OOBs.
    pub fn get_tile_neighbors(&self, tile_pos: MapVec2) -> Vec<Option<&Entity>> {
        let mut neighbors = Vec::new();
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x, tile_pos.y + 1)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x, tile_pos.y - 1)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x - 1, tile_pos.y)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x + 1, tile_pos.y)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x - 1, tile_pos.y + 1)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x + 1, tile_pos.y + 1)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x - 1, tile_pos.y - 1)));
        neighbors.push(self.get_tile(MapVec2::new(tile_pos.x + 1, tile_pos.y - 1)));
        neighbors
    }

    /// Retrieves a tile entity from the map. None will be returned for OOBs.
    pub fn get_tile(&self, tile_pos: MapVec2) -> Option<&Entity> {
        let map_size = &self.get_map_size_in_tiles();
        if tile_pos.x >= 0
            && tile_pos.y >= 0
            && tile_pos.x <= map_size.x
            && tile_pos.y <= map_size.y
        {
            let chunk_pos = MapVec2::new(
                tile_pos.x / self.map_settings.chunk_size.x,
                tile_pos.y / self.map_settings.chunk_size.y,
            );

            let chunk = self.chunks.get(&chunk_pos);
            if chunk.is_some() {
                let (_, tiles) = chunk.unwrap();
                return tiles.get(&tile_pos);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    /// Gets a list of all of the tile entities for the map.
    /// Note: This list could be slightly out of date.
    pub fn get_all_tiles(&self) -> Vec<(&MapVec2, &Entity)> {
        self.chunks.values().flat_map(|(_, tiles)| tiles).collect()
    }

    /// Gets the map's size in tiles just for convenience.
    pub fn get_map_size_in_tiles(&self) -> MapVec2 {
        MapVec2::new(
            self.map_settings.map_size.x * self.map_settings.chunk_size.x,
            self.map_settings.map_size.y * self.map_settings.chunk_size.y,
        )
    }

    /// Builds the map's chunks and tiles if populate_chunks is true.
    /// Note: This should always be called right after creating a map.
    pub fn build(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<ColorMaterial>,
        map_entity: Entity,
        populate_chunks: bool,
    ) {
        for x in 0..self.map_settings.map_size.x {
            for y in 0..self.map_settings.map_size.y {
                let mut chunk_entity = None;
                commands.entity(map_entity).with_children(|child_builder| {
                    chunk_entity = Some(child_builder.spawn().id());
                });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = MapVec2::new(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(vec![]));
                mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(vec![]));
                mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let mut chunk = Chunk::new(
                    map_entity,
                    chunk_pos,
                    self.map_settings.chunk_size,
                    self.map_settings.tile_size,
                    self.map_settings.texture_size,
                    mesh_handle.clone(),
                    self.layer_id,
                    dyn_clone::clone_box(&*self.mesher),
                );

                if populate_chunks {
                    chunk.build_tiles(commands, chunk_entity);
                }

                self.chunks
                    .insert(chunk_pos, (chunk_entity, chunk.tiles.clone()));

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    material: material.clone(),
                    transform: Transform::default(),
                    ..Default::default()
                });
            }
        }
    }
}

// Adds new tiles to the chunk hash map.
pub(crate) fn update_chunk_hashmap_for_added_tiles(
    mut chunk_query: Query<&mut Chunk>,
    tile_query: Query<(Entity, &Tile, &MapVec2), Added<Tile>>,
) {
    for (tile_entity, new_tile, tile_pos) in tile_query.iter() {
        if let Ok(mut chunk) = chunk_query.get_mut(new_tile.chunk) {
            chunk.tiles.insert(*tile_pos, tile_entity);
        }
    }
}

// Removes tiles that have been removed from the chunk.
pub(crate) fn update_chunk_hashmap_for_removed_tiles(
    mut commands: Commands,
    mut map_query: Query<&mut Map>,
    mut chunk_query: Query<&mut Chunk>,
    removed_tiles_query: Query<(Entity, &Tile, &MapVec2), With<RemoveTile>>,
) {
    // For now only loop over 1 map.
    if let Some(mut map) = map_query.iter_mut().next() {
        for (tile_entity, removed_tile, tile_pos) in removed_tiles_query.iter() {
            // Remove tile from chunk tiles cache.
            if let Ok(mut tile_chunk) = chunk_query.get_mut(removed_tile.chunk) {
                tile_chunk.tiles.remove(&tile_pos);
            }

            let chunk_coords = MapVec2::new(
                tile_pos.x / map.map_settings.chunk_size.x,
                tile_pos.y / map.map_settings.chunk_size.y,
            );

            // Remove tile from map tiles cache.
            if let Some(map_chunk) = map.chunks.get_mut(&chunk_coords) {
                map_chunk.1.remove(tile_pos);
            }

            // Remove tile entity
            commands.entity(tile_entity).despawn_recursive();
            commands.entity(removed_tile.chunk).insert(RemeshChunk);
        }
    }
}
