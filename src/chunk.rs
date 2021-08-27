use crate::{
    morton_index, morton_pos,
    prelude::ChunkMesher,
    render::TilemapData,
    round_to_power_of_two,
    tile::{GPUAnimated, Tile},
    ChunkPos, ChunkSize, LocalTilePos, TextureSize, TilePos, TileSize, TilemapMeshType,
};
use bevy::{
    prelude::*,
    render::{
        camera::{Camera, OrthographicProjection},
        render_graph::base::{camera::CAMERA_2D, MainPass},
    },
    tasks::AsyncComputeTaskPool,
};
use std::sync::Mutex;

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
#[derive(Clone, Debug)]
pub struct ChunkSettings {
    pub position: ChunkPos,
    pub size: ChunkSize,
    pub tile_size: TileSize,
    /// The size of the texture in pixels.
    pub texture_size: TextureSize,
    /// What map layer the chunk lives in.
    pub layer_id: u16,
    /// How much spacing between each tile in the atlas.
    pub spacing: Vec2,
    /// Cull the chunks in the map when they are off screen.
    pub cull: bool,
    pub mesh_type: TilemapMeshType,
    pub(crate) mesher: ChunkMesher,
    pub(crate) mesh_handle: Handle<Mesh>,
}

/// A component that stores information about a specific chunk in the tile map.
pub struct Chunk {
    /// The map entity that parents the chunk.
    pub map_entity: Entity,
    /// Chunk specific settings.
    pub settings: ChunkSettings,
    /// Tells internal systems that this chunk should be remeshed(send new data to the GPU)
    pub needs_remesh: bool,
    pub(crate) tiles: Vec<Option<Entity>>,
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            map_entity: self.map_entity,
            needs_remesh: self.needs_remesh,
            settings: self.settings.clone(),
            tiles: self.tiles.clone(),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            map_entity: Entity::new(0),
            needs_remesh: true,
            settings: ChunkSettings {
                position: Default::default(),
                size: Default::default(),
                mesh_handle: Default::default(),
                texture_size: TextureSize(0.0, 0.0),
                tile_size: TileSize::default(),
                layer_id: 0,
                spacing: Vec2::ZERO,
                cull: true,
                mesh_type: TilemapMeshType::Square,
                mesher: ChunkMesher,
            },
            tiles: Vec::new(),
        }
    }
}

impl Chunk {
    pub(crate) fn new(
        map_entity: Entity,
        position: ChunkPos,
        chunk_size: ChunkSize,
        tile_size: TileSize,
        texture_size: TextureSize,
        tile_spacing: Vec2,
        mesh_handle: Handle<Mesh>,
        layer_id: u16,
        mesh_type: TilemapMeshType,
        mesher: ChunkMesher,
        cull: bool,
    ) -> Self {
        let tile_size_x = round_to_power_of_two(chunk_size.0 as f32);
        let tile_size_y = round_to_power_of_two(chunk_size.1 as f32);
        let tile_count = tile_size_x.max(tile_size_y);
        let tiles = vec![None; tile_count * tile_count];
        let settings = ChunkSettings {
            position,
            size: chunk_size,
            tile_size,
            texture_size,
            mesh_handle,
            layer_id,
            mesh_type,
            spacing: tile_spacing,
            mesher,
            cull,
        };
        Self {
            map_entity,
            needs_remesh: true,
            settings,
            tiles,
        }
    }

    pub(crate) fn build_tiles<F>(&mut self, chunk_entity: Entity, mut f: F)
    where
        F: FnMut(TilePos, Entity) -> Option<Entity>,
    {
        for x in 0..self.settings.size.0 {
            for y in 0..self.settings.size.1 {
                let tile_pos = TilePos(
                    (self.settings.position.0 * self.settings.size.0) + x,
                    (self.settings.position.1 * self.settings.size.1) + y,
                );
                if let Some(tile_entity) = f(tile_pos, chunk_entity) {
                    let morton_i = morton_index(TilePos(x, y));
                    self.tiles[morton_i] = Some(tile_entity);
                }
            }
        }
    }

    pub fn get_tile_entity(&self, position: LocalTilePos) -> Option<Entity> {
        let morton_tile_index = morton_index(position);
        if morton_tile_index < self.tiles.capacity() {
            return self.tiles[morton_tile_index];
        }
        None
    }

    pub fn for_each_tile_entity<F>(&self, mut f: F)
    where
        F: FnMut((TilePos, &Option<Entity>)),
    {
        self.tiles.iter().enumerate().for_each(|(index, entity)| {
            let chunk_tile_pos = morton_pos(index);
            f((chunk_tile_pos.into(), entity));
        });
    }

    /// Returns the local coordinates of a tile
    ///
    /// Coordinates are relative to the origin of the chunk that this method is called on
    pub fn to_chunk_pos(&self, global_tile_position: TilePos) -> LocalTilePos {
        LocalTilePos(
            global_tile_position.0 - (self.settings.position.0 * self.settings.size.0),
            global_tile_position.1 - (self.settings.position.1 * self.settings.size.1),
        )
    }
}

pub(crate) fn update_chunk_mesh(
    task_pool: Res<AsyncComputeTaskPool>,
    meshes: ResMut<Assets<Mesh>>,
    tile_query: Query<(&TilePos, &Tile, Option<&GPUAnimated>)>,
    mut changed_chunks: Query<(&mut Chunk, &Visible), Or<(Changed<Visible>, Changed<Chunk>)>>,
) {
    let threaded_meshes = Mutex::new(meshes);

    changed_chunks.par_for_each_mut(&task_pool, 5, |(mut chunk, visible)| {
        if visible.is_visible && chunk.needs_remesh {
            log::trace!(
                "Re-meshing chunk at: {:?} layer id of: {}",
                chunk.settings.position,
                chunk.settings.layer_id
            );

            let mut meshes = threaded_meshes.lock().unwrap();
            chunk.settings.mesher.mesh(
                chunk.settings.clone(),
                &chunk.tiles,
                &tile_query,
                &mut meshes,
            );

            chunk.needs_remesh = false;
        }
    });
}

pub(crate) fn update_chunk_visibility(
    camera: Query<(&Camera, &OrthographicProjection, &Transform)>,
    mut chunks: Query<(&GlobalTransform, &Chunk, &mut Visible)>,
) {
    if let Some((_current_camera, ortho, camera_transform)) = camera.iter().find(|data| {
        if let Some(name) = &data.0.name {
            name == CAMERA_2D
        } else {
            false
        }
    }) {
        // Transform camera into world space.
        let left = camera_transform.translation.x + (ortho.left * camera_transform.scale.x);
        let right = camera_transform.translation.x + (ortho.right * camera_transform.scale.x);
        let bottom = camera_transform.translation.y + (ortho.bottom * camera_transform.scale.y);
        let top = camera_transform.translation.y + (ortho.top * camera_transform.scale.y);

        let camera_bounds = Vec4::new(left, right, bottom, top);

        for (global_transform, chunk, mut visible) in chunks.iter_mut() {
            if chunk.settings.mesh_type != TilemapMeshType::Square || !chunk.settings.cull {
                continue;
            }

            let bounds_size = Vec2::new(
                chunk.settings.size.0 as f32 * chunk.settings.tile_size.0,
                chunk.settings.size.1 as f32 * chunk.settings.tile_size.1,
            );

            let bounds = Vec4::new(
                global_transform.translation.x,
                global_transform.translation.x + bounds_size.x,
                global_transform.translation.y,
                global_transform.translation.y + bounds_size.y,
            );

            let padded_camera_bounds = Vec4::new(
                camera_bounds.x - (bounds_size.x),
                camera_bounds.y + (bounds_size.x),
                camera_bounds.z - (bounds_size.y),
                camera_bounds.w + (bounds_size.y),
            );

            if (bounds.x >= padded_camera_bounds.x) && (bounds.y <= padded_camera_bounds.y) {
                if (bounds.z < padded_camera_bounds.z) || (bounds.w > padded_camera_bounds.w) {
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
                    log::trace!(
                        "Hiding chunk @: {:?}, with camera_bounds: {:?}, bounds_size: {:?}",
                        bounds,
                        padded_camera_bounds,
                        bounds_size
                    );
                    visible.is_visible = false;
                }
            }
        }
    }
}

pub(crate) fn update_chunk_time(time: Res<Time>, mut query: Query<&mut TilemapData>) {
    for mut data in query.iter_mut() {
        data.time = time.seconds_since_startup() as f32;
    }
}
