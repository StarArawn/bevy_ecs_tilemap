use crate::ldtk::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::*;

#[path = "../helpers/mod.rs"]
mod helpers;
mod ldtk;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let handle: Handle<LdtkMap> = asset_server.load("map.ldtk");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(LdtkMapBundle {
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
