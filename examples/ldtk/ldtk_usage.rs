//! This example is capable of spawning tilemaps from [LDtk](https://ldtk.io) files.
//!
//! It can load the AutoTile and Tile layers of simple LDtk levels.
//! However, it does have limitations.
//! Some edge cases around tileset definitions and layer definitions haven't been considered here.
//! Furthermore, since this example is primarily concerned with the tilemap functionality,
//! there's no solution built in for Entity or Intgrid layers.
//!
//! For a more comprehensive LDtk solution, consider [bevy_ecs_ldtk](https://github.com/Trouv/bevy_ecs_ldtk), which uses bevy_ecs_tilemap internally.

use crate::ldtk::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::*;

#[path = "../helpers/mod.rs"]
mod helpers;
mod ldtk;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let handle: Handle<LdtkMap> = asset_server.load("map.ldtk");

    commands.spawn().insert_bundle(LdtkMapBundle {
        ldtk_map: handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("LDTK Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_plugin(LdtkPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
