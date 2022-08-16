use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("flat_hex_tiles.png");

    let tilemap_size = TilemapSize { x: 128, y: 128 };
    let mut tile_storage =
        TilemapStorage::empty(TilemapMeshType::Hexagon(HexType::Column), tilemap_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(0),
        TilePos { x: 0, y: 0 },
        TilemapSize { x: 128, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(1),
        TilePos { x: 64, y: 0 },
        TilemapSize { x: 128, y: 64 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(2),
        TilePos { x: 0, y: 64 },
        TilemapSize { x: 64, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(3),
        TilePos { x: 64, y: 64 },
        TilemapSize { x: 128, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 17.0, y: 15.0 };
    let grid_size = TilemapGridSize { x: 17.0, y: 15.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            ..Default::default()
        });
}

fn swap_mesh_type(mut query: Query<&mut TilemapStorage>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut tilemap_storage in query.iter_mut() {
            match tilemap_storage.get_mesh_type() {
                TilemapMeshType::Hexagon(HexType::Column) => {
                    tilemap_storage.set_mesh_type(TilemapMeshType::Hexagon(HexType::ColumnEven));
                }
                TilemapMeshType::Hexagon(HexType::ColumnEven) => {
                    tilemap_storage.set_mesh_type(TilemapMeshType::Hexagon(HexType::ColumnOdd));
                }
                TilemapMeshType::Hexagon(HexType::ColumnOdd) => {
                    tilemap_storage.set_mesh_type(TilemapMeshType::Hexagon(HexType::Column));
                }
                _ => {}
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Hexagon Column Example"),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(swap_mesh_type)
        .run();
}
