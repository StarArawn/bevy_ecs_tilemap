//! This example is capable of spawning tilemaps from [LDtk](https://ldtk.io) files.
//!
//! It can load the AutoTile and Tile layers of simple LDtk levels.
//! However, it does have limitations.
//! Some edge cases around tileset definitions and layer definitions haven't been considered here.
//! Furthermore, since this example is primarily concerned with the tilemap functionality,
//! there's no solution built in for Entity or Intgrid layers.
//!
//! For a more comprehensive LDtk solution, consider [bevy_ecs_ldtk](https://github.com/Trouv/bevy_ecs_ldtk), which uses bevy_ecs_tilemap internally.

use bevy::prelude::*;
use bevy_ecs_tilemap::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let handle = helpers::ldtk::LdtkMapHandle(asset_server.load("map.ldtk"));

    commands.spawn(helpers::ldtk::LdtkMapBundle {
        ldtk_map: handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("LDTK Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(helpers::ldtk::LdtkPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
