#![allow(dead_code)]

use bevy::prelude::{Bundle, Plugin, Transform, GlobalTransform};
use map::{
    Tilemap2dGridSize, Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize,
    TilemapMeshType, TilemapTexture,
};
use tiles::Tile2dStorage;

pub mod helpers;
pub mod map;
pub(crate) mod render;
pub mod tiles;

pub struct Tilemap2dPlugin;

impl Plugin for Tilemap2dPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(render::Tilemap2dRenderingPlugin);
    }
}

#[derive(Bundle, Debug, Default, Clone)]
pub struct TilemapBundle {
    pub grid_size: Tilemap2dGridSize,
    pub mesh_type: TilemapMeshType,
    pub size: Tilemap2dSize,
    pub spacing: Tilemap2dSpacing,
    pub storage: Tile2dStorage,
    pub texture_size: Tilemap2dTextureSize,
    pub texture: TilemapTexture,
    pub tile_size: Tilemap2dTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub mod prelude {
    pub use crate::Tilemap2dPlugin;
    pub use crate::TilemapBundle;
    pub use crate::helpers::*;
    pub use crate::map::*;
    pub use crate::tiles::*;
}