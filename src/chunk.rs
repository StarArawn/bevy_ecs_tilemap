use std::sync::Mutex;
use bevy::{prelude::*, render::{camera::{Camera, OrthographicProjection}, render_graph::base::{MainPass, camera::CAMERA_2D}}, tasks::{AsyncComputeTaskPool, Task}};
use crate::{TilemapMeshType, morton_index, prelude::{SquareChunkMesher, TilemapChunkMesher}, render::{TilemapData}, tile::{self, GPUAnimated, Tile}};

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
    pub tilemap_data: TilemapData,
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
            render_pipeline: TilemapMeshType::Square.into(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            tilemap_data: TilemapData::default(),
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
    /// How much spacing between each tile in the atlas.
    pub spacing: Vec2,
    pub mesh_type: TilemapMeshType,
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
            spacing: self.spacing,
            mesh_handle: self.mesh_handle.clone(),
            mesh_type: self.mesh_type.clone(),
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
                spacing: Vec2::ZERO,
                mesh_type: TilemapMeshType::Square,
                mesher: Box::new(SquareChunkMesher),
            } 
        }
    }
}

impl Chunk {
    pub(crate) fn new(map_entity: Entity, position: UVec2, chunk_size: UVec2, tile_size: Vec2, texture_size: Vec2, mesh_handle: Handle<Mesh>, layer_id: u32, mesh_type: TilemapMeshType, mesher: Box<dyn TilemapChunkMesher>) -> Self {
        let tiles = vec![None; chunk_size.x as usize * chunk_size.y as usize];
        let settings = ChunkSettings {
            position,
            size: chunk_size,
            tile_size,
            texture_size,
            mesh_handle,
            layer_id,
            mesh_type,
            spacing: Vec2::ZERO, // TODO: Allow spacing to be passed in from map settings,
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
                let mut tile = f(tile_pos);
                tile.chunk = chunk_entity;
                let tile_entity = commands.spawn()
                    .insert(tile)
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
    tile_query: Query<(&UVec2, &Tile, Option<&GPUAnimated>), With<tile::VisibleTile>>,
    mut query_mesh_task: Query<(Entity, &mut Task<(Handle<Mesh>, Mesh)>), With<Chunk>>,
    changed_chunks: Query<(Entity, &Chunk, &Visible), Or<(Changed<Visible>, With<RemeshChunk>, Added<Chunk>)>>,

) {
    let threaded_commands = Mutex::new(commands);
    let threaded_meshes = Mutex::new(meshes);

    // Update chunks that have been "marked" as needing re-meshing.
    // TODO: Eventually when par_for_each works better here we should use that.
    // Currently we can't pull data out of a par_for_each or use commands inside of it. :(
    changed_chunks.par_for_each(&task_pool, 10, |(chunk_entity, chunk, visible)| {
    //changed_chunks.for_each(|(chunk_entity, chunk, visible)| {
        if visible.is_visible  {
            log::trace!("Re-meshing chunk at: {:?} layer id of: {}", chunk.settings.position, chunk.settings.layer_id);
            
            // let instant = std::time::Instant::now();
            // let mut tile_chunk_data = vec![None; (chunk.settings.size.x * chunk.settings.size.y) as usize];
            // chunk.tiles.iter().enumerate().for_each(|(index, tile_entity)| {
            //     if let Some(tile_entity) = tile_entity {
            //         if let Ok(tile) = tile_query.get(*tile_entity) {
            //             tile_chunk_data[index] = Some(*tile);
            //         }
            //     }
            // });

            let (mesh_handle, new_mesh) = chunk.settings.mesher.mesh(chunk.settings.clone(), &chunk.tiles, &tile_query);
            let mut meshes = threaded_meshes.lock().unwrap();
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                *mesh = new_mesh;
            }

            let mut commands = threaded_commands.lock().unwrap();
            commands.entity(chunk_entity).remove::<RemeshChunk>();
            //dbg!(instant.elapsed());
        }
    });
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

            if chunk.settings.mesh_type != TilemapMeshType::Square {
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

pub(crate) fn update_chunk_time(
    time: Res<Time>,
    mut query: Query<&mut TilemapData>,
) {
    for mut data in query.iter_mut() {
        data.time = time.seconds_since_startup() as f32;
    }
}