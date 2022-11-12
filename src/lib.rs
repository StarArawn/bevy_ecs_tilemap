#![allow(dead_code)]

//! Bevy ECS Tilemap plugin is a ECS driven tilemap rendering library. It's designed to be fast and highly customizable. Each tile is considered a unique entity and all tiles are stored in the game world.
//!
//!
//! ## Features
//! - A tile per entity.
//! - Fast rendering using a chunked approach.
//! - Layers and sparse tile maps.
//! - GPU powered animations.
//! - Isometric and Hexagonal tile maps.
//! - Initial support for Tiled file exports.
//! - Support for isometric and hexagon rendering.
//! - Built in animation support  – see [`animation` example](https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/animation.rs).
//! - Texture array support.

use bevy::prelude::{
    Bundle, Changed, Component, ComputedVisibility, CoreStage, Deref, GlobalTransform, Plugin,
    Query, Transform, Visibility,
};
#[cfg(all(not(feature = "atlas"), feature = "render"))]
use map::TilemapTextureSize;
use map::{
    TilemapGridSize, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize, TilemapType,
};
use tiles::{TilePos, TilePosOld, TileStorage};

#[cfg(all(not(feature = "atlas"), feature = "render"))]
use bevy::render::{RenderApp, RenderStage};

/// A module that allows pre-loading of atlases into array textures.
#[cfg(not(feature = "atlas"))]
mod array_texture_preload;
/// A module which provides helper functions.
pub mod helpers;
/// A module which contains tilemap components.
pub mod map;
#[cfg(feature = "render")]
pub(crate) mod render;
/// A module which contains tile components.
pub mod tiles;

/// A bevy tilemap plugin. This must be included in order for everything to be rendered.
/// But is not necessary if you are running without a renderer.
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "render")]
        app.add_plugin(render::TilemapRenderingPlugin);

        app.add_system_to_stage(CoreStage::First, update_changed_tile_positions);

        #[cfg(all(not(feature = "atlas"), feature = "render"))]
        {
            app.insert_resource(array_texture_preload::ArrayTextureLoader::default());
            let render_app = app.sub_app_mut(RenderApp);
            render_app.add_system_to_stage(RenderStage::Extract, array_texture_preload::extract);
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    /// By default, `FrustumCulling` is `true`.
    fn default() -> Self {
        FrustumCulling(true)
    }
}

/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Default, Clone)]
pub struct TilemapBundle {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub computed_visibility: ComputedVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling,
}

/// A module which exports commonly used dependencies.
pub mod prelude {
    #[cfg(not(feature = "atlas"))]
    pub use crate::array_texture_preload::*;
    pub use crate::helpers::filling::*;
    pub use crate::helpers::geometry::*;
    pub use crate::helpers::hex_grid::*;
    pub use crate::helpers::projection::*;
    pub use crate::helpers::selection::*;
    pub use crate::helpers::square_grid::*;
    pub use crate::helpers::transform::*;
    pub use crate::map::*;
    pub use crate::tiles::*;
    pub use crate::TilemapBundle;
    pub use crate::TilemapPlugin;
}

/// Updates old tile positions with the new values from the last frame.
fn update_changed_tile_positions(mut query: Query<(&TilePos, &mut TilePosOld), Changed<TilePos>>) {
    for (tile_pos, mut tile_pos_old) in query.iter_mut() {
        tile_pos_old.0 = *tile_pos;
    }
}
