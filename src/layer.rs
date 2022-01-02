use crate::{
    chunk::Chunk,
    morton_index,
    prelude::{ChunkMesher, Tile},
    round_to_power_of_two,
    tile::TileParent,
    ChunkPos, ChunkSize, MapSize, TextureSize, TilePos, TileSize, TilemapMeshType,
};
use bevy::prelude::*;
use std::hash::Hash;

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
#[derive(Debug, Default, Component, Copy, Clone)]
pub struct LayerSettings {
    /// Size of the tilemap in chunks
    pub map_size: MapSize,
    /// Size in tiles of each chunk.
    pub chunk_size: ChunkSize,
    /// Size in pixels of each tile.
    pub tile_size: TileSize,
    /// Size in pixels of the grid(default is tile_size)
    pub grid_size: Vec2,
    /// Size in pixels of the tilemap texture.
    pub texture_size: TextureSize,
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
    pub fn new(
        map_size: MapSize,
        chunk_size: ChunkSize,
        tile_size: TileSize,
        texture_size: TextureSize,
    ) -> Self {
        Self {
            map_size,
            chunk_size,
            tile_size,
            grid_size: tile_size.into(),
            texture_size,
            cull: true,
            mesh_type: TilemapMeshType::Square,
            tile_spacing: Vec2::ZERO,
            mesher: ChunkMesher,
        }
    }

    pub fn get_pixel_center(&self) -> Vec2 {
        Vec2::new(
            ((self.map_size.0 * self.chunk_size.0) as f32 * self.tile_size.0) / 2.0,
            ((self.map_size.1 * self.chunk_size.1) as f32 * self.tile_size.1) / 2.0,
        )
    }

    pub fn get_center(&self) -> TilePos {
        TilePos(
            (self.map_size.0 * self.chunk_size.0) / 2,
            (self.map_size.1 * self.chunk_size.1) / 2,
        )
    }
}

/// A component which keeps information and a cache of tile/chunk entities for convenience.
#[derive(Default, Component)]
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
        let map_size_x = round_to_power_of_two(settings.map_size.0 as f32);
        let map_size_y = round_to_power_of_two(settings.map_size.1 as f32);
        let map_size = map_size_x.max(map_size_y);
        Self {
            settings,
            chunks: vec![None; map_size * map_size],
        }
    }

    pub fn get_chunk(&self, chunk_pos: ChunkPos) -> Option<Entity> {
        match self.chunks.get(morton_index(chunk_pos)) {
            Some(Some(chunk)) => Some(*chunk),
            _ => None,
        }
    }

    /// Gets the map's size in tiles just for convenience.
    pub fn get_layer_size_in_tiles(&self) -> MapSize {
        MapSize(
            self.settings.map_size.0 * self.settings.chunk_size.0,
            self.settings.map_size.1 * self.settings.chunk_size.1,
        )
    }
}

// Adds new tiles to the chunk hash map.
pub(crate) fn update_chunk_hashmap_for_added_tiles(
    mut chunk_query: Query<&mut Chunk>,
    tile_query: Query<(Entity, &TilePos, &TileParent), Added<Tile>>,
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

/// A type that can be used to identify which layer a tile is on.
///
/// These are ultimately converted to u16; if you're using more than one type with this trait in your game,
/// ensure that their u16 conversions do not unintentionally overlap.
pub trait LayerId: Clone + Copy + PartialEq + Eq + Hash + Into<u16> {}

impl LayerId for u16 {}
