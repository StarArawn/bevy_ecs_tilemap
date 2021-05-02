use bevy::prelude::*;
use chunk::update_chunk_mesh;
use map::{update_chunk_hashmap_for_added_tiles, update_chunk_hashmap_for_removed_tiles};
use render::pipeline::add_tile_map_graph;

mod chunk;
mod map;
mod map_vec2;
mod mesher;
mod render;
mod tile;

#[derive(Default)]
pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // TODO: Perhaps use a stage here instead?
            .add_system(
                update_chunk_hashmap_for_added_tiles
                    .system()
                    .label("hash_update_for_tiles"),
            )
            .add_system(
                update_chunk_hashmap_for_removed_tiles
                    .system()
                    .label("hash_update_for_tiles_removal"),
            )
            .add_system(
                update_chunk_mesh
                    .system()
                    .after("hash_update_for_tiles")
                    .after("hash_update_for_tiles_removal"),
            );
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub mod prelude {
    pub use crate::chunk::Chunk;
    pub use crate::map::{Map, MapBundle, MapSettings, MapTileError, RemoveTile};
    pub use crate::map_vec2::MapVec2;
    pub use crate::mesher::{
        HexChunkMesher, HexType, IsoChunkMesher, SquareChunkMesher, TilemapChunkMesher,
    };
    pub use crate::tile::Tile;
    pub use crate::TileMapPlugin;
}
