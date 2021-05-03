use std::{task::{Context, Poll}};
use bevy::{prelude::*, render::{pipeline::RenderPipeline, render_graph::base::MainPass}, tasks::{AsyncComputeTaskPool, Task}};
use futures_util::FutureExt;
use crate::{map_vec2::MapVec2, morton_index, prelude::{SquareChunkMesher, TilemapChunkMesher}, render::pipeline::TILE_MAP_PIPELINE_HANDLE, tile::Tile};

/// TODO: DOCS
pub struct RemeshChunk;

/// TODO: DOCS
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

pub struct ChunkSettings {
    pub position: MapVec2,
    pub size: MapVec2,
    pub(crate) tile_size: Vec2,
    pub(crate) texture_size: Vec2,
    pub(crate) layer_id: u32,
    pub(crate) mesher: Box<dyn TilemapChunkMesher>,
    pub(crate) mesh_handle: Handle<Mesh>,
}

impl Clone for ChunkSettings {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            size: self.size,
            tile_size: self.tile_size,
            texture_size: self.texture_size,
            layer_id: self.layer_id,
            mesh_handle: self.mesh_handle.clone(),
            mesher: dyn_clone::clone_box(&*self.mesher),
        }
    }
}


/// TODO: DOCS
pub struct Chunk {
    #[allow(dead_code)]
    pub map_entity: Entity,
    pub settings: ChunkSettings,
    pub(crate) tiles: Vec<Option<Entity>>,
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            map_entity: self.map_entity,
            tiles: self.tiles.clone(),
            settings: self.settings.clone(),
        }
    }
}


impl Default for Chunk {
    fn default() -> Self {
        Self {
            map_entity: Entity::new(0),
            tiles: Vec::new(),
            settings: ChunkSettings {
                position: Default::default(),
                size: Default::default(),
                mesh_handle: Default::default(),
                texture_size: Vec2::ZERO,
                tile_size: Vec2::ZERO,
                layer_id: 0,
                mesher: Box::new(SquareChunkMesher),
            } 
        }
    }
}

impl Chunk {
    pub(crate) fn new(map_entity: Entity, position: MapVec2, chunk_size: MapVec2, tile_size: Vec2, texture_size: Vec2, mesh_handle: Handle<Mesh>, layer_id: u32, mesher: Box<dyn TilemapChunkMesher>) -> Self {
        let tiles = vec![None; chunk_size.x as usize * chunk_size.y as usize];
        Self {
            map_entity,
            tiles,
            settings: ChunkSettings {
                position,
                size: chunk_size,
                tile_size,
                texture_size,
                mesh_handle,
                layer_id,
                mesher,
            },
        }
    }

    pub(crate) fn build_tiles(&mut self, commands: &mut Commands, chunk_entity: Entity) {
        for x in 0..self.settings.size.x {
            for y in 0..self.settings.size.y {
                let tile_pos = MapVec2 {
                    x: (self.settings.position.x * self.settings.size.x) + x,
                    y: (self.settings.position.y * self.settings.size.y) + y,
                };
                let tile_entity = commands.spawn()
                    .insert(Tile {
                        chunk: chunk_entity,
                        ..Tile::default()
                    })
                    .insert(tile_pos).id();
                let morton_index = morton_index(MapVec2::new(x, y));
                self.tiles[morton_index] = Some(tile_entity);
            }
        }
    }

    pub fn get_tile_entity(&self, position: MapVec2) -> Option<Entity> {
        self.tiles[morton_index(position)]
    }

    pub fn to_chunk_pos(&self, position: MapVec2) -> MapVec2 {
        MapVec2::new(
            position.x - (self.settings.position.x * self.settings.size.x),
            position.y - (self.settings.position.y * self.settings.size.y),
        )
    }
}

pub(crate) fn update_chunk_mesh(
    mut commands: Commands,
    task_pool: Res<AsyncComputeTaskPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_query: Query<(&MapVec2, &Tile)>,
    mut query_mesh_task: Query<(Entity, &mut Task<(Handle<Mesh>, Mesh)>), With<Chunk>>,
    changed_chunks: Query<(Entity, &Chunk), Or<(With<RemeshChunk>, Added<Chunk>)>>,

) {
    // Update chunks that have been "marked" as needing re-meshing.
    // TODO: Eventually when par_for_each works better here we should use that.
    // Currently we can't pull data out of a par_for_each or use commands inside of it. :(
    changed_chunks.for_each(|(chunk_entity, chunk)| {
        log::info!("Re-meshing chunk at: {:?} layer id of: {}", chunk.settings.position, chunk.settings.layer_id);
        
        let mut tile_chunk_data = vec![None; (chunk.settings.size.x * chunk.settings.size.y) as usize];
        chunk.tiles.iter().for_each(|tile_entity| {
            if let Some(tile_entity) = tile_entity {
                if let Ok((tile_pos, tile)) = tile_query.get(*tile_entity) {
                    let tile_pos = chunk.to_chunk_pos(*tile_pos);
                    tile_chunk_data[morton_index(tile_pos)] = Some(*tile);
                }
            }
        });

        let mesher = dyn_clone::clone_box(&*chunk.settings.mesher);
        let new_mesh_task = task_pool.spawn(mesher.mesh(chunk.settings.clone(), tile_chunk_data));
        
        commands.entity(chunk_entity).insert(new_mesh_task);
        commands.entity(chunk_entity).remove::<RemeshChunk>();
    });
    let noop_waker = futures_util::task::noop_waker();
    let mut cx = Context::from_waker(&noop_waker);

    for (entity, mut task) in query_mesh_task.iter_mut() {
        match task.poll_unpin(&mut cx) {
            Poll::Ready((mesh_handle, new_mesh)) => {
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    *mesh = new_mesh;
                    commands.entity(entity).remove::<Task<(Handle<Mesh>, Mesh)>>();
                }
            },
            _ => ()
        }
    }

}
