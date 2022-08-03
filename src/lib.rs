#![allow(dead_code)]

use bevy::prelude::{Bundle, GlobalTransform, Plugin, Transform};
use map::{
    TilemapGridSize, TilemapMeshType, TilemapSize, TilemapSpacing, TilemapTexture,
    TilemapTextureSize, TilemapTileSize,
};
use tiles::TileStorage;

/// A module which provides helper functions.
pub mod helpers;
/// A module which contains tilemap components.
pub mod map;
pub(crate) mod render;
/// A module which contains tile components.
pub mod tiles;

/// A bevy tilemap plugin. This must be included in order for everything to be rendered.
/// But is not necessary if you are running without a renderer.
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(render::TilemapRenderingPlugin);
    }
}

/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Default, Clone)]
pub struct TilemapBundle {
    pub grid_size: TilemapGridSize,
    pub mesh_type: TilemapMeshType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture_size: TilemapTextureSize,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// A module which exports commonly used dependencies.
pub mod prelude {
    pub use crate::helpers::*;
    pub use crate::map::*;
    pub use crate::tiles::*;
    pub use crate::TilemapBundle;
    pub use crate::TilemapPlugin;
}
