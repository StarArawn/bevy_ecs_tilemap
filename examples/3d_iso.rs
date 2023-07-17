use bevy::utils::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("iso_map.tmx");

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(TilemapRenderSettings {
            // Map size is 12x12 so we'll have render chunks that are:
            // 12 tiles wide and 1 tile tall.
            render_chunk_size: UVec2::new(3, 1),
            y_sort: true,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("3D Isometric Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                }),
        )
        .add_plugins((TilemapPlugin, helpers::tiled::TiledMapPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
