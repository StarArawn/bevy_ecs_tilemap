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
//! let mut map = Map::new(
//!     UVec2::new(2, 2),
//!     UVec2::new(8, 8),
//!     Vec2::new(16.0, 16.0),
//!     Vec2::new(96.0, 256.0),
//!     0
//! );
//! let map_entity = commands.spawn().id();
//! map.build(&mut commands, &mut meshes, material_handle, map_entity, true);
//! commands.entity(map_entity).insert_bundle(MapBundle {
//!     map,
//!    ..Default::default()
//! });
//! ```

use bevy::prelude::*;
use chunk::{update_chunk_mesh, update_chunk_visibility};
use map::{update_chunk_hashmap_for_added_tiles, update_chunk_hashmap_for_removed_tiles, update_tiles};
use render::pipeline::add_tile_map_graph;

mod tile;
mod chunk;
mod map;
mod render;
mod mesher;

pub use crate::tile::{Tile, VisibleTile, RemoveTile};
pub use crate::chunk::Chunk;
pub use crate::map::{Map, MapBundle, MapSettings, MapTileError};
pub use crate::mesher::{SquareChunkMesher, IsoChunkMesher, HexChunkMesher, HexType, TilemapChunkMesher};

/// Adds the default systems and pipelines used by bevy_ecs_tilemap.
#[derive(Default)]
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // TODO: Perhaps use a stage here instead?
            .add_system(update_tiles.system().label("update_tile_data"))
            .add_system(update_chunk_hashmap_for_added_tiles.system().label("hash_update_for_tiles").after("update_tile_data"))
            .add_system(update_chunk_hashmap_for_removed_tiles.system().label("hash_update_for_tiles_removal"))
            .add_system(update_chunk_visibility.system().label("update_chunk_visibility"))
            .add_system(update_chunk_mesh.system().after("hash_update_for_tiles").after("hash_update_for_tiles_removal").after("update_chunk_visibility"));
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub(crate) fn morton_index(tile_pos: UVec2) -> usize {
    morton_encoding::morton_encode([tile_pos.x as u64, tile_pos.y as u64]) as usize
}

// TODO: Hide this.
pub fn morton_pos(index: usize) -> UVec2 {
    let [x, y]: [u32; 2] = morton_encoding::morton_decode(index as u64);
    UVec2::new(x, y)
}

/// use bevy_ecs_tilemap::prelude::*; to import commonly used components, data structures, bundles, and plugins.
pub mod prelude {
    pub use crate::tile::{Tile, VisibleTile, RemoveTile};
    pub use crate::chunk::{Chunk,  ChunkSettings};
    pub use crate::map::{Map, MapBundle, MapSettings, MapTileError};
    pub use crate::TilemapPlugin;
    pub use crate::mesher::{SquareChunkMesher, IsoChunkMesher, HexChunkMesher, HexType, TilemapChunkMesher};
}
