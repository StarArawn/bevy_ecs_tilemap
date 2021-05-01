use std::collections::{HashMap, VecDeque};
use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, pipeline::{PrimitiveTopology}}};
use crate::{chunk::{Chunk, ChunkBundle, RemeshChunk}, map_vec2::MapVec2, prelude::{SquareChunkMesher, Tile, TilemapChunkMesher}};

pub(crate) struct SetTileEvent {
    pub entity: Entity,
    pub tile: Tile,
}

#[derive(Bundle, Default)]
pub struct MapBundle {
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct Map {
    pub size: MapVec2,
    pub chunk_size: MapVec2,
    pub tile_size: Vec2,
    pub texture_size: Vec2,
    pub layer_id: u32,
    chunks: HashMap<MapVec2, (Entity, HashMap<MapVec2, Entity>)>,
    pub mesher: Box<dyn TilemapChunkMesher>,
    events: VecDeque<SetTileEvent>
}

impl Default for Map {
    fn default() -> Self {
        Self {
            size: MapVec2::default(),
            chunk_size: MapVec2::default(),
            tile_size: Vec2::default(),
            texture_size: Vec2::default(),
            layer_id: 0,
            chunks: HashMap::new(),
            mesher: Box::new(SquareChunkMesher),
            events: VecDeque::new(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MapTileError {
    OutOfBounds,
}

/// A tag that allows you to remove a tile from the world;
pub struct RemoveTile;

impl Map {
    pub fn new(size: MapVec2, chunk_size: MapVec2, tile_size: Vec2, texture_size: Vec2, layer_id: u32) -> Self {
        Self {
            size,
            chunk_size,
            tile_size,
            texture_size,
            layer_id,
            chunks: HashMap::new(),
            mesher: Box::new(SquareChunkMesher),
            events: VecDeque::new(),
        }
    }

    /// Tags the tile for removal
    /// You can also remove tile entities manually by doing:
    /// `commands.entity(tile_entity).insert(RemoveTile)`
    pub fn remove_tile(&self, commands: &mut Commands, tile_pos: MapVec2) {
        if let Some(tile_entity) = self.get_tile(tile_pos) {
            commands.entity(tile_entity).insert(RemoveTile);
        }
    }

    /// Adds a new tile to the map if the tile already exists this code will do nothing.
    pub fn add_tile(&mut self, commands: &mut Commands, tile_pos: MapVec2, tile: Tile) -> Result<Entity, MapTileError> {
        // First find chunk tile should live in:
        let mut possible_tile_event = None;

        if let Some(chunk_data) = self.get_chunk_mut(MapVec2::new(
            tile_pos.x / self.chunk_size.x,
            tile_pos.y / self.chunk_size.y
        )) {
            if let Some(tile_entity) = chunk_data.1.get(&tile_pos) {
                // If the tile already exists we need to queue an event.
                // We do this because we have good way of changing the actual tile data from here.
                // Another possibility is having the user pass in a query of tiles and matching based off of entity,
                // however the tile may not exist yet in this current frame even though it exists in the Commands
                // buffer.
                possible_tile_event = Some(SetTileEvent {
                    entity: *tile_entity,
                    tile: Tile {
                        chunk: chunk_data.0,
                        ..tile
                    },
                });
            } else {
                let tile_entity = commands.spawn()
                    .insert(Tile {
                        chunk: chunk_data.0,
                        ..tile
                    })
                    .insert(tile_pos).id();
                chunk_data.1.insert(tile_pos, tile_entity);

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
 
    fn get_chunk_mut(&mut self, chunk_pos: MapVec2) -> Option<&mut (Entity, HashMap<MapVec2, Entity>)> {
        self.chunks.get_mut(&chunk_pos)
    }

    /// Retrieves a list of neighbor entities in the following order:
    /// N, S, W, E, NW, NE, SW, SE. None will be returned for tiles that don't exist.
    pub fn get_tile_neighbors(&self, tile_pos: MapVec2) -> Vec<(MapVec2, Option<Entity>)> {
        let mut neighbors = Vec::new();
        let n = MapVec2::new(tile_pos.x, tile_pos.y + 1);
        neighbors.push((n, self.get_tile(n)));
        let s = MapVec2::new(tile_pos.x, tile_pos.y - 1);
        neighbors.push((s, self.get_tile(s)));
        let w = MapVec2::new(tile_pos.x - 1, tile_pos.y);
        neighbors.push((w, self.get_tile(w)));
        let e = MapVec2::new(tile_pos.x + 1, tile_pos.y);
        neighbors.push((e, self.get_tile(e)));
        let nw = MapVec2::new(tile_pos.x - 1, tile_pos.y + 1);
        neighbors.push((nw, self.get_tile(nw)));
        let ne = MapVec2::new(tile_pos.x + 1, tile_pos.y + 1);
        neighbors.push((ne, self.get_tile(ne)));
        let sw = MapVec2::new(tile_pos.x - 1, tile_pos.y - 1);
        neighbors.push((sw, self.get_tile(sw)));
        let se = MapVec2::new(tile_pos.x + 1, tile_pos.y - 1);
        neighbors.push((se, self.get_tile(se)));
        neighbors
    }

    /// Returns a list of tile entities base off of the positions requested
    pub fn get_tiles(&self, tile_positions: Vec<MapVec2>) -> Vec<Option<Entity>> {
        let mut tiles = Vec::new();
        for pos in tile_positions {
            tiles.push(
                self.get_tile(pos)
            );
        }
        tiles
    }

    /// Retrieves a tile entity from the map. None will be returned for tiles that don't exist.
    pub fn get_tile(&self, tile_pos: MapVec2) -> Option<Entity> {
        let map_size = &self.get_map_size_in_tiles();
        if tile_pos.x >= 0 && tile_pos.y >= 0 && tile_pos.x <= map_size.x && tile_pos.y <= map_size.y {
            let chunk_pos = MapVec2::new(
                tile_pos.x / self.chunk_size.x,
                tile_pos.y / self.chunk_size.y,
            );

            let chunk = self.chunks.get(&chunk_pos);
            if chunk.is_some() {
                let (_, tiles) = chunk.unwrap();
                let tile = tiles.get(&tile_pos);
                return match tile {
                    Some(tile) => Some(*tile),
                    None => None,
                };
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
            self.size.x * self.chunk_size.x,
            self.size.y * self.chunk_size.y,
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
        for x in 0..self.size.x {
            for y in 0..self.size.y {
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
                let mesh_handle =  meshes.add(mesh);
                let mut chunk = Chunk::new(map_entity, chunk_pos, self.chunk_size, self.tile_size, self.texture_size, mesh_handle.clone(), self.layer_id, dyn_clone::clone_box(&*self.mesher));

                if populate_chunks {
                    chunk.build_tiles(commands, chunk_entity);
                }

                self.chunks.insert(chunk_pos, (chunk_entity, chunk.tiles.clone()));

                commands.entity(chunk_entity)
                    .insert_bundle(ChunkBundle {
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
    removed_tiles_query: Query<(Entity, &Tile, &MapVec2), With<RemoveTile>>
) {
    // For now only loop over 1 map.
    if let Some(mut map) = map_query.iter_mut().next() {
        for (tile_entity, removed_tile, tile_pos) in removed_tiles_query.iter() {

            // Remove tile from chunk tiles cache.
            if let Ok(mut tile_chunk) = chunk_query.get_mut(removed_tile.chunk) {
                tile_chunk.tiles.remove(&tile_pos);
            }

            let chunk_coords = MapVec2::new(
                tile_pos.x / map.chunk_size.x,
                tile_pos.y / map.chunk_size.y,
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

pub(crate) fn update_tiles(
    mut commands: Commands,
    mut map: Query<&mut Map>,
) {

    for mut map in map.iter_mut() {
        while let Some(event) = map.events.pop_front() {
            commands.entity(event.entity).remove::<Tile>().insert(event.tile);
        }
    }

}