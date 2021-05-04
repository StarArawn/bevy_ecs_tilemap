//! TODO: DOCS
use bevy::prelude::*;
use chunk::{update_chunk_mesh, update_chunk_visibility};
use map::{update_chunk_hashmap_for_added_tiles, update_chunk_hashmap_for_removed_tiles, update_tiles};
use render::pipeline::add_tile_map_graph;

mod tile;
mod chunk;
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
            .add_system(update_chunk_visibility.system().label("update_chunk_visibility"))
            .add_system(update_chunk_mesh.system().after("hash_update_for_tiles").after("hash_update_for_tiles_removal"));
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub(crate) fn morton_index(tile_pos: UVec2) -> usize {
    lindel::morton_encode([tile_pos.x as u64, tile_pos.y as u64]) as usize
}

pub(crate) fn morton_pos(index: usize) -> UVec2 {
    let [x, y]: [u32; 2] = lindel::morton_decode(index as u64);
    UVec2::new(x, y)
}

pub mod prelude {
    pub use crate::tile::{Tile, Visible};
    pub use crate::chunk::Chunk;
    pub use crate::map::{Map, MapBundle, MapSettings, MapTileError, RemoveTile};
    pub use crate::TileMapPlugin;
    pub use crate::mesher::{SquareChunkMesher, IsoChunkMesher, HexChunkMesher, HexType, TilemapChunkMesher};
}
