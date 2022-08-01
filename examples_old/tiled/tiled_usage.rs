use crate::tiled::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[path = "../helpers/mod.rs"]
mod helpers;
mod tiled;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let handle: Handle<TiledMap> = asset_server.load("map.tmx");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Tiled map editor example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
