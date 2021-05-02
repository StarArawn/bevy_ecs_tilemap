use std::{collections::HashMap, task::{Context, Poll}};
use bevy::{prelude::*, render::{pipeline::RenderPipeline, render_graph::base::MainPass}, tasks::{AsyncComputeTaskPool, Task}};
use futures_util::FutureExt;
use crate::{map_vec2::MapVec2, prelude::{SquareChunkMesher, TilemapChunkMesher}, render::pipeline::TILE_MAP_PIPELINE_HANDLE, tile::Tile};

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
/// TODO: DOCS
pub struct Chunk {
    #[allow(dead_code)]
    pub map_entity: Entity,
    pub position: MapVec2,
    pub size: MapVec2,
    pub(crate) tiles: HashMap<MapVec2, Entity>,
    pub(crate) mesh_handle: Handle<Mesh>,
    pub(crate) tile_size: Vec2,
    pub(crate) texture_size: Vec2,
    pub(crate) layer_id: u32,
    mesher: Box<dyn TilemapChunkMesher>,
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            map_entity: self.map_entity,
            position: self.position,
            size: self.size,
            tiles: self.tiles.clone(),
            mesh_handle: self.mesh_handle.clone(),
            tile_size: self.tile_size,
            texture_size: self.texture_size,
            layer_id: self.layer_id,
            mesher: dyn_clone::clone_box(&*self.mesher),
        }
    }
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
    task_pool: Res<AsyncComputeTaskPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_query: Query<(&MapVec2, &Tile)>,
    mut query_mesh_task: Query<(Entity, &mut Task<(Handle<Mesh>, Mesh)>), With<Chunk>>,
    changed_chunks: Query<(Entity, &Chunk), Or<(With<RemeshChunk>, Added<Chunk>)>>,

) {
    // Update chunks that have been "marked" as needing re-meshing.
    for (chunk_entity, chunk) in changed_chunks.iter() {
        log::info!("Re-meshing chunk at: {:?} layer id of: {}", chunk.position, chunk.layer_id);
        
        let mut tile_chunk_data = HashMap::<MapVec2, Tile>::new();
        chunk.tiles.values().for_each(|tile_entity| {
            if let Ok((tile_pos, tile)) = tile_query.get(*tile_entity) {
                tile_chunk_data.insert(*tile_pos, *tile);
            }
        });

        let mesher = dyn_clone::clone_box(&*chunk.mesher);
        let new_mesh_task = task_pool.spawn(mesher.mesh(chunk.clone(), tile_chunk_data));
        
        commands.entity(chunk_entity).insert(new_mesh_task);
        commands.entity(chunk_entity).remove::<RemeshChunk>();
    }
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
