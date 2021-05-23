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
//! let layer_entity = commands.spawn().id();
//! let mut layer_builder = LayerBuilder::new(
//!     &mut commands,
//!     layer_entity,    
//!     LayerSettings::new(
//!         UVec2::new(2, 2),
//!         UVec2::new(8, 8),
//!         Vec2::new(16.0, 16.0),
//!         Vec2::new(96.0, 256.0),
//!     )
//! );
//!
//! layer_builder.set_all(TileBundle::default(), true);
//!
//! map_query.create_layer(&mut commands, layer_builder, material_handle);
//! ```

use bevy::prelude::*;
use chunk::{update_chunk_mesh, update_chunk_time, update_chunk_visibility};
use layer::update_chunk_hashmap_for_added_tiles;
use render::pipeline::add_tile_map_graph;

mod chunk;
mod layer;
mod layer_builder;
mod map_query;
mod mesher;
mod render;
mod tile;

pub use crate::chunk::{Chunk, ChunkSettings};
pub use crate::layer::{Layer, LayerBundle, LayerSettings, MapTileError};
pub use crate::tile::{GPUAnimated, RemoveTile, Tile};

/// Adds the default systems and pipelines used by bevy_ecs_tilemap.
#[derive(Default)]
pub struct TilemapPlugin;

/// Different hex coordinate systems. You can find out more at this link: https://www.redblobgames.com/grids/hexagons/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexType {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different iso coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoType {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilemapMeshType {
    Square,
    Hexagon(HexType),
    Isometric(IsoType),
}

/// The tilemap stage which runs before post update.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub struct TilemapStage;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut AppBuilder) {
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
            );
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub(crate) fn morton_index(tile_pos: UVec2) -> usize {
    morton_encoding::morton_encode([tile_pos.x as u64, tile_pos.y as u64]) as usize
}

// TODO: Hide this.
fn morton_pos(index: usize) -> UVec2 {
    let [x, y]: [u32; 2] = morton_encoding::morton_decode(index as u64);
    UVec2::new(x, y)
}

/// use bevy_ecs_tilemap::prelude::*; to import commonly used components, data structures, bundles, and plugins.
pub mod prelude {
    pub use crate::chunk::{Chunk, ChunkSettings};
    pub use crate::layer::{Layer, LayerBundle, LayerSettings, MapTileError};
    pub use crate::layer_builder::LayerBuilder;
    pub use crate::map_query::MapQuery;
    pub(crate) use crate::mesher::{SquareChunkMesher, TilemapChunkMesher};
    pub use crate::tile::{
        GPUAnimated,
        RemoveTile,
        Tile,
        TileBundle,
        TileBundleTrait,
        TileParent,
    };
    pub use crate::TilemapPlugin;
    pub use crate::{HexType, IsoType, TilemapMeshType};
}

pub(crate) fn round_to_power_of_two(value: f32) -> usize {
    1 << value.log2().ceil() as usize
}