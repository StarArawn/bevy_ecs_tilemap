//! A tilemap rendering plugin for bevy which is more ECS friendly by having an entity per tile.
//!
//! ## Features
//!  - A tile per entity
//!  - Fast rendering using a chunked approach.
//!  - Layers and sparse tile maps.
//!  - Animations
//!  - Isometric and Hexagonal tile maps
//!
//! ## Upcoming Features
//!  - ~~Support for isometric and hexagon rendering~~ done
//!  - ~~Built in animation support.~~ done see animation example
//!  - Texture array support
//!  - ~~Layers and add/remove tiles. (High Priority)~~ done
//!
//! ## Example
//! ```
//! let texture_handle = asset_server.load("tiles.png");
//! let material_handle = materials.add(ColorMaterial::texture(texture_handle));
//!
//! // Create map entity and component:
//! let map_entity = commands.spawn().id();
//! let mut map = Map::new(0u16, map_entity);
//!
//! // Creates a new layer builder with a layer entity.
//! let (mut layer_builder, _) = LayerBuilder::new(
//!     &mut commands,
//!     LayerSettings::new(
//!         MapSize(2, 2),
//!         ChunkSize(8, 8),
//!         TileSize(16.0, 16.0),
//!         Vec2::new(96.0, 256.0),
//!     ),
//!     0u16,
//!     0u16,
//! );
//!
//! layer_builder.set_all(TileBundle::default());
//!
//! // Builds the layer.
//! // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
//! let layer_entity = map_query.build_layer(&mut commands, layer_builder, material_handle);
//!
//! // Required to keep track of layers for a map internally.
//! map.add_layer(&mut commands, 0u16, layer_entity);
//!
//! // Spawn Map
//! // Required in order to use map_query to retrieve layers/tiles.
//! commands.entity(map_entity)
//!     .insert(map)
//!     .insert(Transform::from_xyz(
//!         -128.0,
//!         -128.0,
//!         0.0
//!     ))
//!     .insert(GlobalTransform::default());
//! ```

use bevy::prelude::*;
use chunk::{update_chunk_mesh, update_chunk_time, update_chunk_visibility};
use layer::update_chunk_hashmap_for_added_tiles;

mod chunk;
mod layer;
mod layer_builder;
mod map;
mod map_query;
mod mesher;
mod neighbors;
mod render;
mod tile;

pub use crate::chunk::Chunk;
pub use crate::layer::{Layer, LayerBundle, LayerSettings, MapTileError};
pub use crate::layer_builder::LayerBuilder;
pub use crate::map::Map;
pub use crate::map_query::MapQuery;
pub use crate::tile::{GPUAnimated, Tile, TileBundle, TileBundleTrait, TileParent};

/// Adds the default systems and pipelines used by bevy_ecs_tilemap.
#[derive(Default)]
pub struct TilemapPlugin;

/// Different hex coordinate systems. You can find out more at this link: https://www.redblobgames.com/grids/hexagons/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexType {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different iso coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoType {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilemapMeshType {
    Square,
    Hexagon(HexType),
    Isometric(IsoType),
}

impl Default for TilemapMeshType {
    fn default() -> Self {
        Self::Square
    }
}

/// The tilemap stage which runs before post update.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub struct TilemapStage;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(CoreStage::PostUpdate, TilemapStage, SystemStage::parallel())
            .add_system_to_stage(TilemapStage, update_chunk_time.system())
            .add_system_to_stage(
                TilemapStage,
                update_chunk_hashmap_for_added_tiles
                    .system()
                    .label("hash_update_for_tiles"),
            )
            .add_system_to_stage(
                TilemapStage,
                update_chunk_visibility
                    .system()
                    .label("update_chunk_visibility"),
            )
            .add_system_to_stage(
                TilemapStage,
                update_chunk_mesh
                    .system()
                    .after("hash_update_for_tiles")
                    .after("update_chunk_visibility"),
            )
            .add_plugin(render::TilemapRenderPlugin);
    }
}

pub(crate) fn morton_index(tile_pos: impl Into<UVec2>) -> usize {
    let tile_pos: UVec2 = tile_pos.into();
    morton_encoding::morton_encode([tile_pos.x, tile_pos.y]) as usize
}

// TODO: Hide this.
fn morton_pos(index: usize) -> UVec2 {
    let [x, y]: [u32; 2] = morton_encoding::morton_decode(index as u64);
    UVec2::new(x, y)
}

/// use bevy_ecs_tilemap::prelude::*; to import commonly used components, data structures, bundles, and plugins.
pub mod prelude {
    pub use crate::chunk::Chunk;
    pub use crate::layer::{Layer, LayerBundle, LayerSettings, MapTileError};
    pub use crate::layer_builder::LayerBuilder;
    pub use crate::map::{Map, MapId};
    pub use crate::map_query::MapQuery;
    pub(crate) use crate::mesher::ChunkMesher;
    pub use crate::tile::{GPUAnimated, Tile, TileBundle, TileBundleTrait, TileParent};
    pub use crate::TilemapPlugin;
    pub use crate::{HexType, IsoType, TilemapMeshType};

    pub use crate::{ChunkPos, ChunkSize, LocalTilePos, MapSize, TextureSize, TilePos, TileSize};

    pub use crate::neighbors::get_neighboring_pos;
}

pub(crate) fn round_to_power_of_two(value: f32) -> usize {
    1 << value.log2().ceil() as usize
}

/// The size of the map, in chunks
#[derive(Default, Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapSize(pub u32, pub u32);

impl From<Vec2> for MapSize {
    fn from(vec: Vec2) -> Self {
        MapSize(vec.x as u32, vec.y as u32)
    }
}

impl Into<Vec2> for MapSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}

/// The size of each chunk, in tiles
#[derive(Default, Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChunkSize(pub u32, pub u32);

impl From<Vec2> for ChunkSize {
    fn from(vec: Vec2) -> Self {
        ChunkSize(vec.x as u32, vec.y as u32)
    }
}

impl Into<Vec2> for ChunkSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}

/// The size of each tile, in pixels
#[derive(Default, Component, Clone, Copy, PartialEq, Debug)]
pub struct TileSize(pub f32, pub f32);

impl From<Vec2> for TileSize {
    fn from(vec: Vec2) -> Self {
        TileSize(vec.x, vec.y)
    }
}

impl Into<Vec2> for TileSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.0, self.1)
    }
}

/// The size of a texture in pixels
#[derive(Default, Component, Clone, Copy, PartialEq, Debug)]
pub struct TextureSize(pub f32, pub f32);

impl Into<Vec2> for TextureSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.0, self.1)
    }
}

/// The position of a tile, in map coordinates
///
/// Coordinates start at (0, 0) from the bottom-left tile of the map.
#[derive(Default, Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct TilePos(pub u32, pub u32);

impl Into<UVec2> for TilePos {
    fn into(self) -> UVec2 {
        UVec2::new(self.0, self.1)
    }
}

impl Into<TilePos> for UVec2 {
    fn into(self) -> TilePos {
        TilePos(self.x, self.y)
    }
}

/// The position of a tile, in chunk coordinates
///
/// Coordinates start at (0, 0) from the bottom-left tile of the chunk.
#[derive(Default, Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LocalTilePos(pub u32, pub u32);

impl Into<UVec2> for LocalTilePos {
    fn into(self) -> UVec2 {
        UVec2::new(self.0, self.1)
    }
}

/// The position of a chunk within a map
///
/// Coordinates start at (0, 0) from the bottom-left chunk of the map.
/// Note that these coordinates are measured in terms of chunks, not tiles.
#[derive(Default, Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ChunkPos(pub u32, pub u32);

impl Into<UVec2> for ChunkPos {
    fn into(self) -> UVec2 {
        UVec2::new(self.0, self.1)
    }
}

impl Into<Vec2> for ChunkPos {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}
