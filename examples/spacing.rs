use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("tiles-spaced.png");

    let map_size = TilemapSize { x: 32, y: 32 };

    // Layer 1
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        Tilemap,
        grid_size,
        map_type,
        map_size,
        tile_storage,
        TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        TilemapSpacing { x: 8.0, y: 8.0 },
        get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
    ));

    // Layer 2
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(2),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands.entity(tilemap_entity).insert((
        Tilemap,
        grid_size,
        map_type,
        map_size,
        tile_storage,
        TilemapTexture::Single(texture_handle),
        TilemapTileSize { x: 16.0, y: 16.0 },
        get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0)
            * Transform::from_xyz(32.0, 32.0, 0.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Spacing Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
