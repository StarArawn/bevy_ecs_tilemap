use crate::{
    morton_index, morton_pos,
    render::TilemapUniformData,
    round_to_power_of_two,
    tile::{GPUAnimated, Tile},
    ChunkPos, LayerImage, LayerSettings, LocalTilePos, TilePos, TilemapMeshType,
};
use bevy::{
    core::Time,
    math::{Vec2, Vec4},
    prelude::*,
    render::camera::CameraPlugin,
    tasks::AsyncComputeTaskPool,
};
use std::sync::Mutex;

#[derive(Bundle)]
pub(crate) struct ChunkBundle {
    pub chunk: Chunk,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub tilemap_data: TilemapUniformData,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for ChunkBundle {
    fn default() -> Self {
        Self {
            chunk: Chunk::default(),
            mesh: Handle::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            tilemap_data: TilemapUniformData::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

/// A component that stores information about a specific chunk in the tile map.
#[derive(Debug, Component, Clone)]
pub struct Chunk {
    /// The specific location x,y of the chunk in the tile map in chunk coords.
    pub position: ChunkPos,
    /// The map entity that parents the chunk.
    pub map_entity: Entity,
    /// Chunk specific settings.
    pub settings: LayerSettings,
    /// Tells internal systems that this chunk should be remeshed(send new data to the GPU)
    pub needs_remesh: bool,
    /// Tells the renderer which image to use for the tilemap.
    pub material: Handle<LayerImage>,
    pub(crate) tiles: Vec<Option<Entity>>,
    pub(crate) mesh_handle: Handle<Mesh>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            map_entity: Entity::new(0),
            material: Default::default(),
            mesh_handle: Default::default(),
            needs_remesh: true,
            position: Default::default(),
            settings: Default::default(),
            tiles: Vec::new(),
        }
    }
}

impl Chunk {
    pub(crate) fn new(
        map_entity: Entity,
        layer_settings: LayerSettings,
        position: ChunkPos,
        mesh_handle: Handle<Mesh>,
        material: Handle<LayerImage>,
    ) -> Self {
        let tile_size_x = round_to_power_of_two(layer_settings.chunk_size.0 as f32);
        let tile_size_y = round_to_power_of_two(layer_settings.chunk_size.1 as f32);
        let tile_count = tile_size_x.max(tile_size_y);
        let tiles = vec![None; tile_count * tile_count];

        Self {
            map_entity,
            material,
            mesh_handle,
            needs_remesh: true,
            position,
            settings: layer_settings,
            tiles,
        }
    }

    pub(crate) fn build_tiles<F>(&mut self, chunk_entity: Entity, mut f: F)
    where
        F: FnMut(TilePos, Entity) -> Option<Entity>,
    {
        for x in 0..self.settings.chunk_size.0 {
            for y in 0..self.settings.chunk_size.1 {
                let tile_pos = TilePos(
                    (self.position.0 * self.settings.chunk_size.0) + x,
                    (self.position.1 * self.settings.chunk_size.1) + y,
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
            global_tile_position.0 - (self.position.0 * self.settings.chunk_size.0),
            global_tile_position.1 - (self.position.1 * self.settings.chunk_size.1),
        )
    }
}

pub(crate) fn update_chunk_mesh(
    task_pool: Res<AsyncComputeTaskPool>,
    meshes: ResMut<Assets<Mesh>>,
    tile_query: Query<(&TilePos, &Tile, Option<&GPUAnimated>)>,
    mut changed_chunks: Query<
        (&mut Chunk, &Visibility),
        Or<(Added<Chunk>, Changed<Chunk>, Changed<Visibility>)>,
    >,
) {
    let threaded_meshes = Mutex::new(meshes);

    changed_chunks.par_for_each_mut(&task_pool, 5, |(mut chunk, visibility)| {
        if chunk.needs_remesh && visibility.is_visible {
            log::trace!(
                "Re-meshing chunk at: {:?} layer id of: {}",
                chunk.position,
                chunk.settings.layer_id
            );

            let mut meshes = threaded_meshes.lock().unwrap();
            chunk
                .settings
                .mesher
                .mesh(&chunk, &chunk.tiles, &tile_query, &mut meshes);

            chunk.needs_remesh = false;
        }
    });
}

pub(crate) fn update_chunk_visibility(
    camera: Query<(&Camera, &OrthographicProjection, &Transform)>,
    mut chunks: Query<(&GlobalTransform, &Chunk, &mut Visibility)>,
) {
    if let Some((_current_camera, ortho, camera_transform)) = camera.iter().find(|data| {
        if let Some(name) = &data.0.name {
            name == CameraPlugin::CAMERA_2D
        } else {
            false
        }
    }) {
        // Transform camera into world space.
        let left =
            camera_transform.translation.x + (ortho.left * ortho.scale * camera_transform.scale.x);
        let right =
            camera_transform.translation.x + (ortho.right * ortho.scale * camera_transform.scale.x);
        let bottom = camera_transform.translation.y
            + (ortho.bottom * ortho.scale * camera_transform.scale.y);
        let top =
            camera_transform.translation.y + (ortho.top * ortho.scale * camera_transform.scale.y);

        let camera_bounds = Vec4::new(left, right, bottom, top);

        for (global_transform, chunk, mut visibility) in chunks.iter_mut() {
            if chunk.settings.mesh_type != TilemapMeshType::Square || !chunk.settings.cull {
                continue;
            }

            let bounds_size = Vec2::new(
                chunk.settings.chunk_size.0 as f32
                    * chunk.settings.tile_size.0
                    * global_transform.scale.x,
                chunk.settings.chunk_size.1 as f32
                    * chunk.settings.tile_size.1
                    * global_transform.scale.y,
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
                    if visibility.is_visible {
                        log::trace!("Hiding chunk @: {:?}", bounds);
                        visibility.is_visible = false;
                    }
                } else {
                    if !visibility.is_visible {
                        log::trace!("Showing chunk @: {:?}", bounds);
                        visibility.is_visible = true;
                    }
                }
            } else {
                if visibility.is_visible {
                    log::trace!(
                        "Hiding chunk @: {:?}, with camera_bounds: {:?}, bounds_size: {:?}",
                        bounds,
                        padded_camera_bounds,
                        bounds_size
                    );
                    visibility.is_visible = false;
                }
            }
        }
    }
}

pub(crate) fn update_chunk_time(time: Res<Time>, mut query: Query<&mut TilemapUniformData>) {
    for mut data in query.iter_mut() {
        data.time = time.seconds_since_startup() as f32;
    }
}
