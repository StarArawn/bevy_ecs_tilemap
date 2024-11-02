use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use helpers::tiled::TiledMapAssetHandle;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let map_handle: TiledMapAssetHandle = asset_server.load("rotate.tmx").into();

    commands.spawn((helpers::tiled::TiledMap, map_handle));
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Tiled Map Editor Rotated Example"),
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
        .run();
}
