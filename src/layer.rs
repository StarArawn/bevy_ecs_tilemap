use crate::{
    chunk::Chunk,
    morton_index,
    prelude::{ChunkMesher, Tile},
    round_to_power_of_two,
    tile::TileParent,
    TilemapMeshType,
};
use bevy::prelude::*;

/// A bevy bundle which contains: Map, Transform, and GlobalTransform components.
#[derive(Bundle, Default)]
pub struct LayerBundle {
    /// The map component for the tilemap.
    pub layer: Layer,
    /// The local transform of the tilemap entity.
    pub transform: Transform,
    /// The global transform of the tilemap entity.
    pub global_transform: GlobalTransform,
}

/// Various settings used to define the tilemap.
#[derive(Debug, Default, Copy, Clone)]
pub struct LayerSettings {
    /// Size of the tilemap in chunks
    pub map_size: UVec2,
    /// Size in tiles of each chunk.
    pub chunk_size: UVec2,
    /// Size in pixels of each tile.
    pub tile_size: Vec2,
    /// Size in pixels of the tilemap texture.
    pub texture_size: Vec2,
    /// The layer id associated with this map.
    pub layer_id: u16,
    /// The map id associated with this map.
    pub map_id: u16,
    /// The meshing algorithm used for the tilemap.
    pub mesh_type: TilemapMeshType,
    /// Cull the chunks in the map when they are off screen.
    pub cull: bool,
    /// Spacing around each tile in the atlas
    /// Note: This is ignored in array mode.
    pub tile_spacing: Vec2,
    pub(crate) mesher: ChunkMesher,
}

impl LayerSettings {
    pub fn new(map_size: UVec2, chunk_size: UVec2, tile_size: Vec2, texture_size: Vec2) -> Self {
        Self {
            map_size,
            chunk_size,
            tile_size,
            texture_size,
            layer_id: 0,
            map_id: 0,
            cull: true,
            mesh_type: TilemapMeshType::Square,
            tile_spacing: Vec2::ZERO,
            mesher: ChunkMesher,
        }
    }

    pub fn new_with_map_type(
        map_size: UVec2,
        chunk_size: UVec2,
        tile_size: Vec2,
        texture_size: Vec2,
        mesh_type: TilemapMeshType,
    ) -> Self {
        Self {
            map_size,
            chunk_size,
            tile_size,
            texture_size,
            layer_id: 0,
            map_id: 0,
            cull: true,
            mesh_type,
            tile_spacing: Vec2::ZERO,
            mesher: ChunkMesher,
        }
    }

    pub fn set_layer_id<L: Into<u16>>(&mut self, id: L) {
        self.layer_id = id.into();
    }

    pub fn set_map_id<M: Into<u16>>(&mut self, id: M) {
        self.map_id = id.into();
    }

    pub fn get_pixel_center(&self) -> Vec2 {
        Vec2::new(
            ((self.map_size.x * self.chunk_size.x) as f32 * self.tile_size.x) / 2.0,
            ((self.map_size.y * self.chunk_size.y) as f32 * self.tile_size.y) / 2.0,
        )
    }

    pub fn get_center(&self) -> UVec2 {
        UVec2::new(
            (self.map_size.x * self.chunk_size.x) / 2,
            (self.map_size.y * self.chunk_size.y) / 2,
        )
    }
}

/// A component which keeps information and a cache of tile/chunk entities for convenience.
#[derive(Default)]
pub struct Layer {
    /// The map information for the tilemap entity.
    pub settings: LayerSettings,
    pub(crate) chunks: Vec<Option<Entity>>,
}

/// General errors that are returned by bevy_ecs_tilemap.
#[derive(Debug, Copy, Clone)]
pub enum MapTileError {
    /// The tile was out of bounds.
    OutOfBounds,
    /// The tile already exists.
    AlreadyExists,
    /// Doesn't exist
    NonExistent,
}

impl Layer {
    /// Creates a new map component.
    ///
    /// - `settings`: The map settings struct.
    pub fn new(settings: LayerSettings) -> Self {
        let map_size_x = round_to_power_of_two(settings.map_size.x as f32);
        let map_size_y = round_to_power_of_two(settings.map_size.y as f32);
        let map_size = map_size_x.max(map_size_y);
        Self {
            settings,
            chunks: vec![None; map_size * map_size],
        }
    }

    pub fn get_chunk(&self, chunk_pos: UVec2) -> Option<Entity> {
        self.chunks[morton_index(chunk_pos)]
    }

    /// Gets the map's size in tiles just for convenience.
    pub fn get_layer_size_in_tiles(&self) -> UVec2 {
        UVec2::new(
            self.settings.map_size.x * self.settings.chunk_size.x,
            self.settings.map_size.y * self.settings.chunk_size.y,
        )
    }
}

// Adds new tiles to the chunk hash map.
pub(crate) fn update_chunk_hashmap_for_added_tiles(
    mut chunk_query: Query<&mut Chunk>,
    tile_query: Query<(Entity, &UVec2, &TileParent), Added<Tile>>,
) {
    if tile_query.iter().count() > 0 {
        log::info!("Updating tile cache.");
    }
    for (tile_entity, tile_pos, tile_parent) in tile_query.iter() {
        if let Ok(mut chunk) = chunk_query.get_mut(tile_parent.chunk) {
            let tile_pos = chunk.to_chunk_pos(*tile_pos);
            chunk.tiles[morton_index(tile_pos)] = Some(tile_entity);
        }
    }
}
