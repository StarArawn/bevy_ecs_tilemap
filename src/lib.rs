use bevy::prelude::*;
use chunk::update_chunk_mesh;
use render::pipeline::add_tile_map_graph;

mod tile;
mod chunk;
mod map_vec2;
mod map;
mod render;

#[derive(Default)]
pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_chunk_mesh.system());
        let world = app.world_mut();
        add_tile_map_graph(world);
    }
}

pub mod prelude {
    pub use crate::tile::Tile;
    pub use crate::chunk::Chunk;
    pub use crate::map_vec2::MapVec2;
    pub use crate::map::Map;
    pub use crate::TileMapPlugin;
}
