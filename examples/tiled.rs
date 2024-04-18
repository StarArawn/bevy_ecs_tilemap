use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::helpers::tiled::{TiledLayersStorage, TiledMap};

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("map.tmx");

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Tiled Map Editor Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(helpers::tiled::TiledMapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, hide_tiles)
        .run();
}

fn hide_tiles(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    tile_storage_query: Query<&TileStorage>,
    mut map_query: Query<&TiledLayersStorage>,
    mut tile_query: Query<&mut TileVisible>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for layer_storage in map_query.iter_mut() {
            for layer_entity in layer_storage.storage.values() {
                if let Ok(layer_tile_storage) = tile_storage_query.get(*layer_entity) {
                    for tile in layer_tile_storage.iter().flatten() {
                        if let Ok(mut t) = tile_query.get_mut(*tile) {
                            t.0 = false;
                        }
                    }
                }
            }
        }
    }
}
