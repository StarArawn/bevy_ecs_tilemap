#![allow(dead_code)]
#![expect(deprecated)]

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

use bevy::{
    prelude::{
        Bundle, Changed, Component, Deref, First, GlobalTransform, InheritedVisibility,
        IntoSystemConfigs, IntoSystemSetConfigs, Plugin, Query, Reflect, ReflectComponent,
        SystemSet, Transform, ViewVisibility, Visibility,
    },
    time::TimeSystem,
};

#[cfg(feature = "render")]
use map::{
    TilemapGridSize, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize,
    TilemapTileSize, TilemapType,
};
use prelude::{TilemapId, TilemapRenderSettings};
use render::material::{StandardTilemapMaterial, TilemapMaterial, TilemapMaterialHandle};
use std::marker::PhantomData;
use tiles::{
    AnimatedTile, TileColor, TileFlip, TilePos, TilePosOld, TileStorage, TileTextureIndex,
    TileVisible,
};

#[cfg(all(not(feature = "atlas"), feature = "render"))]
use bevy::render::{ExtractSchedule, RenderApp};

/// A module that allows pre-loading of atlases into array textures.
#[cfg(all(not(feature = "atlas"), feature = "render"))]
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
        app.add_plugins(render::TilemapRenderingPlugin);

        app.add_systems(First, update_changed_tile_positions.in_set(TilemapFirstSet));

        #[cfg(all(not(feature = "atlas"), feature = "render"))]
        {
            app.insert_resource(array_texture_preload::ArrayTextureLoader::default());
            let render_app = app.sub_app_mut(RenderApp);
            render_app.add_systems(ExtractSchedule, array_texture_preload::extract);
        }

        app.register_type::<FrustumCulling>()
            .register_type::<TilemapId>()
            .register_type::<TilemapSize>()
            .register_type::<TilemapTexture>()
            .register_type::<TilemapTileSize>()
            .register_type::<TilemapGridSize>()
            .register_type::<TilemapSpacing>()
            .register_type::<TilemapTextureSize>()
            .register_type::<TilemapType>()
            .register_type::<TilePos>()
            .register_type::<TileTextureIndex>()
            .register_type::<TileColor>()
            .register_type::<TileVisible>()
            .register_type::<TileFlip>()
            .register_type::<TileStorage>()
            .register_type::<TilePosOld>()
            .register_type::<AnimatedTile>()
            .configure_sets(First, TilemapFirstSet.after(TimeSystem));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TilemapFirstSet;

#[derive(Component, Reflect, Debug, Clone, Copy, Deref)]
#[reflect(Component)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    /// By default, `FrustumCulling` is `true`.
    fn default() -> Self {
        FrustumCulling(true)
    }
}

#[cfg(feature = "render")]
#[deprecated(
    since = "0.15.0",
    note = "Use the `Tilemap` type alias instead. Inserting it will now automatically insert the other components it requires."
)]
pub type TilemapBundle = MaterialTilemapBundle<StandardTilemapMaterial>;

#[cfg(feature = "render")]
pub type Tilemap = MaterialTilemap<StandardTilemapMaterial>;
pub const Tilemap: Tilemap = MaterialTilemap::<StandardTilemapMaterial>(PhantomData);

#[cfg(feature = "render")]
#[derive(Component, Clone, Debug)]
#[require(
    FrustumCulling,
    GlobalTransform,
    InheritedVisibility,
    TilemapMaterialHandle<M>,
    TileStorage,
    TilemapGridSize,
    TilemapRenderSettings,
    TilemapSize,
    TilemapSpacing,
    TilemapTexture,
    TilemapTileSize,
    TilemapType,
    Transform,
    ViewVisibility,
    Visibility,
)]
/// The default tilemap, with custom Material rendering support.
pub struct MaterialTilemap<M: TilemapMaterial>(pub PhantomData<M>);

impl<M: TilemapMaterial> Default for MaterialTilemap<M> {
    fn default() -> Self {
        MaterialTilemap(PhantomData)
    }
}

#[cfg(feature = "render")]
#[deprecated(
    since = "0.15.0",
    note = "Use the `MaterialTilemap` component instead. Inserting it will now automatically insert the other components it requires."
)]
#[derive(Bundle, Debug, Default, Clone)]
/// The default tilemap, with custom Material rendering support.
pub struct MaterialTilemapBundle<M: TilemapMaterial> {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling,
    pub material: TilemapMaterialHandle<M>,
}

#[cfg(not(feature = "render"))]
#[derive(Component, Clone, Debug)]
#[require(
    FrustumCulling,
    GlobalTransform,
    InheritedVisibility,
    TileStorage,
    TilemapGridSize,
    TilemapRenderSettings,
    TilemapSize,
    TilemapSpacing,
    TilemapTexture,
    TilemapTileSize,
    TilemapType,
    Transform,
    ViewVisibility,
    Visibility
)]
/// The default tilemap.
pub struct StandardTilemap;

#[cfg(not(feature = "render"))]
#[deprecated(
    since = "0.15.0",
    note = "Use the `StandardTilemap` component instead. Inserting it will now automatically insert the other components it requires."
)]
#[derive(Bundle, Debug, Default, Clone)]
/// The default tilemap.
pub struct StandardTilemapBundle {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling,
}

/// A module which exports commonly used dependencies.
pub mod prelude {
    #[cfg(all(not(feature = "atlas"), feature = "render"))]
    pub use crate::array_texture_preload::*;
    pub use crate::helpers;
    pub use crate::helpers::filling::*;
    pub use crate::helpers::geometry::*;
    pub use crate::helpers::transform::*;
    pub use crate::map::*;
    #[cfg(feature = "render")]
    pub use crate::render::material::StandardTilemapMaterial;
    #[cfg(feature = "render")]
    pub use crate::render::material::TilemapMaterial;
    #[cfg(feature = "render")]
    pub use crate::render::material::TilemapMaterialHandle;
    #[cfg(feature = "render")]
    pub use crate::render::material::TilemapMaterialKey;
    #[cfg(feature = "render")]
    pub use crate::render::material::TilemapMaterialPlugin;
    pub use crate::tiles::*;
    #[cfg(feature = "render")]
    pub use crate::MaterialTilemapBundle;
    #[cfg(feature = "render")]
    pub use crate::Tilemap;
    pub use crate::TilemapBundle;
    pub use crate::TilemapPlugin;
}

/// Updates old tile positions with the new values from the last frame.
fn update_changed_tile_positions(mut query: Query<(&TilePos, &mut TilePosOld), Changed<TilePos>>) {
    for (tile_pos, mut tile_pos_old) in query.iter_mut() {
        tile_pos_old.0 = *tile_pos;
    }
}
