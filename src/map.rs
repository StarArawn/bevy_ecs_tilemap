use std::collections::HashMap;
use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, pipeline::{PrimitiveTopology}}};
use crate::{chunk::{Chunk, ChunkBundle}, map_vec2::MapVec2};

#[derive(Debug, Default)]
pub struct Map {
    size: MapVec2,
    chunk_size: MapVec2,
    tile_size: Vec2,
    texture_size: Vec2,
    chunks: HashMap<MapVec2, (Entity, HashMap<MapVec2, Entity>)>,
}

impl Map {
    pub fn new(size: MapVec2, chunk_size: MapVec2, tile_size: Vec2, texture_size: Vec2) -> Self {
        Self {
            size,
            chunk_size,
            tile_size,
            texture_size,
            chunks: HashMap::new(),
        }
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
        if tile_pos.x >= 0 && tile_pos.y >= 0 && tile_pos.x <= map_size.x && tile_pos.y <= map_size.y {
            let chunk_pos = MapVec2::new(
                tile_pos.x / self.chunk_size.x,
                tile_pos.y / self.chunk_size.y,
            );

            let chunk = self.chunks.get(&chunk_pos);
            if chunk.is_some() {
                let (_, tiles) = chunk.unwrap();
                return tiles.get(&tile_pos)
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn get_all_tiles(&self) -> Vec<&Entity> {
        self.chunks.values().flat_map(|(_, tiles)| tiles.values()).collect()
    }

    pub fn get_map_size_in_tiles(&self) -> MapVec2 {
        MapVec2::new(
            self.size.x * self.chunk_size.x,
            self.size.y * self.chunk_size.y,
        )
    }

    pub fn build(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<ColorMaterial>,
        map_entity: Entity
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
                let mut chunk = Chunk::new(map_entity, chunk_pos, self.chunk_size, self.tile_size, self.texture_size, mesh_handle.clone());
                chunk.build_tiles(commands, chunk_entity);

                self.chunks.insert(chunk_pos, (chunk_entity, chunk.tiles.clone()));

                commands.entity(chunk_entity)
                    .insert_bundle(ChunkBundle {
                        chunk,
                        mesh: mesh_handle,
                        material: material.clone(),
                        transform: Transform::from_xyz((chunk_pos.x * self.size.x * self.tile_size.x as i32) as f32, (chunk_pos.y * self.size.y * self.tile_size.y as i32) as f32, 0.0),
                        ..Default::default()
                    });
            }
        }
    }
}