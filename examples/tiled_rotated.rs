use bevy::{asset::AssetServerSettings, prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("rotate.tmx");

    commands
        .spawn()
        .insert_bundle(helpers::tiled::TiledMapBundle {
            tiled_map: map_handle,
            ..Default::default()
        });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Tiled Map Editor Rotated Example"),
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(helpers::tiled::TiledMapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .run();
}
