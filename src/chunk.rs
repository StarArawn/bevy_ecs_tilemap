use std::{sync::{Arc, Mutex}, task::{Context, Poll}};
use bevy::{prelude::*, render::{camera::{Camera, OrthographicProjection}, pipeline::RenderPipeline, render_graph::base::{MainPass, camera::CAMERA_2D}}, tasks::{AsyncComputeTaskPool, Task}};
use futures_util::FutureExt;
use crate::{morton_index, prelude::{SquareChunkMesher, TilemapChunkMesher}, render::pipeline::TILE_MAP_PIPELINE_HANDLE, tile::{self, Tile}};

/// A tag that causes a specific chunk to be "re-meshed" when re-meshing is started the tag is removed.
pub struct RemeshChunk;

#[derive(Bundle)]
pub(crate) struct ChunkBundle {
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

/// Chunk specific settings.
#[derive(Debug)]
pub struct ChunkSettings {
    /// The specific location x,y of the chunk in the tile map in chunk coords.
    pub position: UVec2,
    /// The size of the chunk.
    pub size: UVec2,
    /// The size of each tile in pixels.
    pub tile_size: Vec2,
    /// The size of the texture in pixels.
    pub texture_size: Vec2,
    /// What map layer the chunk lives in.
    pub layer_id: u32,
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


/// A component that stores information about a specific chunk in the tile map. 
pub struct Chunk {
    /// The map entity that parents the chunk.
    pub map_entity: Entity,
    /// Chunk specific settings.
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
    pub(crate) fn new(map_entity: Entity, position: UVec2, chunk_size: UVec2, tile_size: Vec2, texture_size: Vec2, mesh_handle: Handle<Mesh>, layer_id: u32, mesher: Box<dyn TilemapChunkMesher>) -> Self {
        let tiles = vec![None; chunk_size.x as usize * chunk_size.y as usize];
        let settings = ChunkSettings {
            position,
            size: chunk_size,
            tile_size,
            texture_size,
            mesh_handle,
            layer_id,
            mesher,
        };
        Self {
            map_entity,
            tiles,
            settings,
        }
    }

    pub(crate) fn build_tiles<F>(
        &mut self,
        commands: &mut Commands,
        chunk_entity: Entity,
        map_tiles: &mut Vec<Option<Entity>>,
        mut f: F,
    ) where F: FnMut(UVec2) -> Tile {
        for x in 0..self.settings.size.x {
            for y in 0..self.settings.size.y {
                let tile_pos = UVec2::new(
                    (self.settings.position.x * self.settings.size.x) + x,
                    (self.settings.position.y * self.settings.size.y) + y,
                );
                let tile_entity = commands.spawn()
                    .insert(Tile {
                        chunk: chunk_entity,
                        ..f(tile_pos)
                    })
                    .insert(tile::VisibleTile)
                    .insert(tile_pos).id();
                let morton_i = morton_index(UVec2::new(x, y));
                self.tiles[morton_i] = Some(tile_entity);
                map_tiles[morton_index(tile_pos)] = Some(tile_entity);
            }
        }
    }

    pub fn get_tile_entity(&self, position: UVec2) -> Option<Entity> {
        self.tiles[morton_index(position)]
    }

    pub fn to_chunk_pos(&self, position: UVec2) -> UVec2 {
        UVec2::new(
            position.x - (self.settings.position.x * self.settings.size.x),
            position.y - (self.settings.position.y * self.settings.size.y),
        )
    }
}

pub(crate) fn update_chunk_mesh(
    mut commands: Commands,
    task_pool: Res<AsyncComputeTaskPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_query: Query<(&UVec2, &Tile), With<tile::VisibleTile>>,
    mut query_mesh_task: Query<(Entity, &mut Task<(Handle<Mesh>, Mesh)>), With<Chunk>>,
    changed_chunks: Query<(Entity, &Chunk, &Visible), Or<(Changed<Visible>, With<RemeshChunk>, Added<Chunk>)>>,

) {
    // Update chunks that have been "marked" as needing re-meshing.
    // TODO: Eventually when par_for_each works better here we should use that.
    // Currently we can't pull data out of a par_for_each or use commands inside of it. :(
    changed_chunks.for_each(|(chunk_entity, chunk, visible)| {
        if visible.is_visible  {
            log::trace!("Re-meshing chunk at: {:?} layer id of: {}", chunk.settings.position, chunk.settings.layer_id);
            
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
        }
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

pub(crate) fn update_chunk_visibility(
    camera: Query<(&Camera, &OrthographicProjection, &Transform)>,
    mut chunks: Query<(&Chunk, &mut Visible)>,
) {
    if let Some((_current_camera, ortho, camera_transform)) = camera.iter().find(|data|
        if let Some(name) = &data.0.name {
            name == CAMERA_2D
        } else { false }
    ) {
        // Transform camera into world space.
        let left = camera_transform.translation.x + (ortho.left * camera_transform.scale.x);
        let right = camera_transform.translation.x + (ortho.right * camera_transform.scale.x);
        let bottom = camera_transform.translation.y + (ortho.bottom * camera_transform.scale.x);
        let top = camera_transform.translation.y + (ortho.top * camera_transform.scale.x);

        let camera_bounds = Vec4::new(left, right, bottom, top);
        
        for (chunk, mut visible) in chunks.iter_mut() {

            if !chunk.settings.mesher.should_cull() {
                continue;
            }

            let bounds_size = Vec2::new(
                chunk.settings.size.x as f32 * chunk.settings.tile_size.x,
                chunk.settings.size.y as f32 * chunk.settings.tile_size.y,
            );

            let bounds = Vec4::new(
                chunk.settings.position.x as f32 * bounds_size.x,
                (chunk.settings.position.x as f32 + 1.0) * bounds_size.x,
                chunk.settings.position.y as f32 * bounds_size.y,
                (chunk.settings.position.y as f32 + 1.0) * bounds_size.y,
            );

            let padded_camera_bounds = Vec4::new(
                camera_bounds.x - (bounds_size.x),
                camera_bounds.y + (bounds_size.x),
                camera_bounds.z - (bounds_size.y),
                camera_bounds.w + (bounds_size.y),
            );

            if (bounds.x >= padded_camera_bounds.x) && (bounds.y <= padded_camera_bounds.y)
            {
                if (bounds.z < padded_camera_bounds.z) || (bounds.w > padded_camera_bounds.w)
                {
                    if visible.is_visible {
                        log::trace!("Hiding chunk @: {:?}", bounds);
                        visible.is_visible = false;
                    }
                } else {
                    if !visible.is_visible {
                        log::trace!("Showing chunk @: {:?}", bounds);
                        visible.is_visible = true;
                    }
                }
            } else {
                if visible.is_visible {
                    log::trace!("Hiding chunk @: {:?}, with camera_bounds: {:?}, bounds_size: {:?}", bounds, padded_camera_bounds, bounds_size);
                    visible.is_visible = false;
                }
            }
        }
    }
}