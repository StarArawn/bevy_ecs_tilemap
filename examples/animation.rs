use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::map::{
    Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
    TilemapTexture,
};
use bevy_ecs_tilemap::tiles::{AnimatedTile, Tile2dStorage, TileBundle, TilePos2d, TileTexture};
use bevy_ecs_tilemap::{Tilemap2dPlugin, TilemapBundle};
use rand::{thread_rng, Rng};

mod helpers;

fn create_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = Tilemap2dSize { x: 20, y: 20 };
    let texture_size = Tilemap2dTextureSize { x: 96.0, y: 16.0 };
    let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };
    let grid_size = Tilemap2dGridSize { x: 16.0, y: 16.0 };

    // To create a map we use the Tile2dStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn().id();

    // Spawn the elements of the tilemap.
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture_size,
            texture: TilemapTexture(texture_handle.clone()),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..Default::default()
        });
}

fn create_flowers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("flower_sheet.png");

    let tilemap_size = Tilemap2dSize { x: 10, y: 10 };
    let texture_size = Tilemap2dTextureSize { x: 32.0, y: 448.0 };
    let tile_size = Tilemap2dTileSize { x: 32.0, y: 32.0 };
    let grid_size = Tilemap2dGridSize { x: 16.0, y: 16.0 };

    let mut tile_storage = Tile2dStorage::empty(tilemap_size);

    let tilemap_entity = commands.spawn().id();

    let mut random = thread_rng();

    for _ in 0..10 {
        let tile_pos = TilePos2d {
            x: random.gen_range(0..tilemap_size.x),
            y: random.gen_range(0..tilemap_size.y),
        };
        let tile_entity = commands
            .spawn()
            .insert_bundle(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture: TileTexture(0),
                ..Default::default()
            })
            .id();
        tile_storage.set(&tile_pos, Some(tile_entity));
        commands.entity(tile_entity).insert(AnimatedTile {
            start: 0,
            end: 13,
            speed: 0.95,
        });
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture_size,
            texture: TilemapTexture(texture_handle.clone()),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                1.0,
            ),
            ..Default::default()
        });
}

fn startup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Animated Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(create_background)
        .add_system(create_flowers)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
