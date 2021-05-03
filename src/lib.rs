//! TODO: DOCS

use bevy::prelude::*;
use chunk::update_chunk_mesh;
use map::{update_chunk_hashmap_for_added_tiles, update_chunk_hashmap_for_removed_tiles, update_tiles};
use prelude::MapVec2;
use render::pipeline::add_tile_map_graph;

mod tile;
mod chunk;
mod map_vec2;
mod map;
mod render;
mod mesher;

/// TODO: DOCS
#[derive(Default)]
pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // TODO: Perhaps use a stage here instead?
            .add_system(update_tiles.system().label("update_tile_data"))
            .add_system(update_chunk_hashmap_for_added_tiles.system().label("hash_update_for_tiles").after("update_tile_data"))
            .add_system(update_chunk_hashmap_for_removed_tiles.system().label("hash_update_for_tiles_removal"))
            .add_system(update_chunk_mesh.system().after("hash_update_for_tiles").after("hash_update_for_tiles_removal"));
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub(crate) fn morton_index(tile_pos: MapVec2) -> usize {
    lindel::morton_encode([tile_pos.x as u64, tile_pos.y as u64]) as usize
}

pub mod prelude {
    pub use crate::tile::Tile;
    pub use crate::chunk::Chunk;
    pub use crate::map_vec2::MapVec2;
    pub use crate::map::{Map, MapBundle, MapTileError, RemoveTile};
    pub use crate::TileMapPlugin;
    pub use crate::mesher::{SquareChunkMesher, IsoChunkMesher, HexChunkMesher, HexType, TilemapChunkMesher};
}
