use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    // `tile_size` specifies how large in pixels each tile is in the source atlas texture
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    // `in_world_tile_size` specifies how large to render each tile in the world
    let in_world_tile_size = TilemapInWorldTileSize { x: 4.0, y: 4.0 };
    // The `grid_size` is essentially how far about each tile is placed from eachother
    let grid_size = TilemapGridSize { x: 4.0, y: 4.0 };

    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        in_world_tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Option::Some(Window {
                        title: String::from("Benchmark Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            TilemapPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
