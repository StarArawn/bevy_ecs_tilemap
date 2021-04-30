use std::collections::HashMap;
use bevy::{prelude::*, render::{pipeline::RenderPipeline, render_graph::base::MainPass}};
use crate::{map_vec2::MapVec2, prelude::{SquareChunkMesher, TilemapChunkMesher}, render::pipeline::TILE_MAP_PIPELINE_HANDLE, tile::Tile};

pub struct RemeshChunk;

#[derive(Bundle)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub main_pass: MainPass,
    pub material: Handle<ColorMaterial>,
    pub render_pipeline: RenderPipelines,
    pub visible: Visible,
    pub draw: Draw,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChunkBundle {
    fn default() -> Self {
        Self {
            chunk: Chunk::default(),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            draw: Draw::default(),
            main_pass: MainPass,
            mesh: Handle::default(),
            material: Handle::default(),
            render_pipeline: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                TILE_MAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub struct Chunk {
    #[allow(dead_code)]
    map_entity: Entity,
    pub position: MapVec2,
    pub size: MapVec2,
    pub(crate) tiles: HashMap<MapVec2, Entity>,
    pub(crate) mesh_handle: Handle<Mesh>,
    pub(crate) tile_size: Vec2,
    pub(crate) texture_size: Vec2,
    pub(crate) layer_id: u32,
    mesher: Box<dyn TilemapChunkMesher>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            map_entity: Entity::new(0),
            position: Default::default(),
            size: Default::default(),
            tiles: HashMap::new(),
            mesh_handle: Default::default(),
            texture_size: Vec2::ZERO,
            tile_size: Vec2::ZERO,
            layer_id: 0,
            mesher: Box::new(SquareChunkMesher)
        }
    }
}

impl Chunk {
    pub(crate) fn new(map_entity: Entity, position: MapVec2, chunk_size: MapVec2, tile_size: Vec2, texture_size: Vec2, mesh_handle: Handle<Mesh>, layer_id: u32, mesher: Box<dyn TilemapChunkMesher>) -> Self {
        let tiles = HashMap::new();
        Self {
            map_entity,
            position,
            tiles,
            size: chunk_size,
            tile_size,
            texture_size,
            mesh_handle,
            layer_id,
            mesher,
        }
    }

    pub(crate) fn build_tiles(&mut self, commands: &mut Commands, chunk_entity: Entity) {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let tile_pos = MapVec2 {
                    x: (self.position.x * self.size.x) + x,
                    y: (self.position.y * self.size.y) + y,
                };
                let tile_entity = commands.spawn()
                    .insert(Tile {
                        chunk: chunk_entity,
                        ..Tile::default()
                    })
                    .insert(tile_pos).id();
                self.tiles.insert(tile_pos, tile_entity);
            }
        }
    }

    pub fn get_tile_entity(&self, position: MapVec2) -> Option<&Entity> {
        self.tiles.get(&position)
    }
}

pub(crate) fn update_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_change_query: Query<&Tile, Or<(Changed<MapVec2>, Changed<Tile>)>>,
    tile_query: Query<(&MapVec2, &Tile)>,
    mut chunk_query: Query<(Entity, &Chunk)>,
    changed_chunks: Query<(Entity, &Chunk), With<RemeshChunk>>,
) {
    // TODO: use a hash set or something here instead. Perhaps the chunk entity itself?
    let mut updated_chunks = Vec::new();
    // If a tile has changed.
    for tile in tile_change_query.iter() {
        if let Ok((chunk_entity, chunk)) = chunk_query.get_mut(tile.chunk) {
            if !updated_chunks.iter().any(|(position, layer_id)| chunk.position == *position && chunk.layer_id == *layer_id) {
                log::info!("Re-meshing chunk at: {:?} layer id of: {}", chunk.position, chunk.layer_id);
                
                // Rebuild tile map mesh.
                chunk.mesher.mesh(chunk, &mut meshes, &tile_query);

                // Make sure we don't recalculate the chunk until the next time this system updates at least.
                updated_chunks.push((chunk.position, chunk.layer_id));

                commands.entity(chunk_entity).remove::<RemeshChunk>();
            }
        }
    }

    // Update chunks that have been "marked" as needing remeshing.
    for (chunk_entity, chunk) in changed_chunks.iter() {
        if !updated_chunks.iter().any(|(position, layer_id)| chunk.position == *position && chunk.layer_id == *layer_id) {
            log::info!("Re-meshing chunk at: {:?} layer id of: {}", chunk.position, chunk.layer_id);
            chunk.mesher.mesh(chunk, &mut meshes, &tile_query);

            commands.entity(chunk_entity).remove::<RemeshChunk>();
        }
    }

}
