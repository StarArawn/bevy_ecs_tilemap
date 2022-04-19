use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::{
        Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapTexture,
    },
    tiles::{Tile2dStorage, TileTexture},
    Tilemap2dPlugin, TilemapBundle,
};

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = Tilemap2dSize { x: 32, y: 32 };

    // Layer 1
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    bevy_ecs_tilemap::helpers::fill_tilemap(
        TileTexture(0),
        tilemap_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle.clone()),
            tile_size: Tilemap2dTileSize { x: 16.0, y: 16.0 },
            ..Default::default()
        });

    // Layer 2
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    bevy_ecs_tilemap::helpers::fill_tilemap(
        TileTexture(2),
        tilemap_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle),
            tile_size: Tilemap2dTileSize { x: 16.0, y: 16.0 },
            transform: Transform::from_xyz(32.0, 32.0, 1.0),
            ..Default::default()
        });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Layers Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
