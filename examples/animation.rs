use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{AnimatedTile, TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapPlugin;
use rand::seq::IteratorRandom;
use rand::thread_rng;

mod helpers;

struct TilemapMetadata {
    size: TilemapSize,
    tile_size: TilemapTileSize,
    grid_size: TilemapGridSize,
}

const BACKGROUND: &str = "tiles.png";
const BACKGROUND_METADATA: TilemapMetadata = TilemapMetadata {
    size: TilemapSize { x: 20, y: 20 },
    tile_size: TilemapTileSize { x: 16.0, y: 16.0 },
    grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
};

const FLOWERS: &str = "flower_sheet.png";
const FLOWERS_METADATA: TilemapMetadata = TilemapMetadata {
    size: TilemapSize { x: 20, y: 20 },
    tile_size: TilemapTileSize { x: 32.0, y: 32.0 },
    grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
};

fn create_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load(BACKGROUND);

    let tilemap_entity = commands.spawn_empty().id();

    let TilemapMetadata {
        size,
        grid_size,
        tile_size,
    } = BACKGROUND_METADATA;

    let mut tile_storage = TileStorage::empty(size);

    fill_tilemap(
        TileTextureIndex(0),
        size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        size,
        grid_size,
        map_type,
        tile_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        transform: get_tilemap_center_transform(&size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

fn create_animated_flowers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load(FLOWERS);

    let TilemapMetadata {
        size: map_size,
        grid_size,
        tile_size,
    } = FLOWERS_METADATA;

    let mut tile_storage = TileStorage::empty(map_size);

    let tilemap_entity = commands.spawn_empty().id();

    // Choose 10 random tiles to contain flowers.
    let mut rng = thread_rng();
    let mut indices: Vec<(u32, u32)> = Vec::with_capacity((map_size.x * map_size.y) as usize);
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            indices.push((x, y));
        }
    }
    for (x, y) in indices.into_iter().choose_multiple(&mut rng, 10) {
        let tile_pos = TilePos { x, y };
        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                },
                // To enable animation, we must insert the `AnimatedTile` component on
                // each tile that is to be animated.
                AnimatedTile {
                    start: 0,
                    end: 13,
                    speed: 0.95,
                },
            ))
            .id();

        tile_storage.set(&tile_pos, tile_entity);
    }
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        size: map_size,
        grid_size,
        map_type,
        tile_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    });
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Animated Map Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Startup, create_background)
        .add_systems(Startup, create_animated_flowers)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, pause_animation)
        .run();
}

fn pause_animation(mut query: Query<&mut AnimatedTile>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyP) {
        for mut anim in &mut query {
            anim.speed = if anim.speed == 0.0 { 1.0 } else { 0.0 }
        }
    }
}
