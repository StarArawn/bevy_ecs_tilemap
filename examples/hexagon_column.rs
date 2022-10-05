use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

// Side length of a colored quadrant (in "number of tiles").
const QUADRANT_SIDE_LENGTH: u32 = 80;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("flat_hex_tiles.png");

    // In total, there will be `(QUADRANT_SIDE_LENGTH * 2) * (QUADRANT_SIDE_LENGTH * 2)` tiles.
    let total_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };
    let quadrant_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };

    let mut tile_storage = TileStorage::empty(total_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTexture(0),
        TilePos { x: 0, y: 0 },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTexture(1),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: 0,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTexture(2),
        TilePos {
            x: 0,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTexture(3),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
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
            size: total_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type: TilemapType::Hexagon(HexCoordSystem::Column),
            ..Default::default()
        });
}

fn swap_mesh_type(mut query: Query<&mut TilemapType>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut tilemap_mesh_type in query.iter_mut() {
            match *tilemap_mesh_type {
                TilemapType::Hexagon(HexCoordSystem::Column) => {
                    *tilemap_mesh_type = TilemapType::Hexagon(HexCoordSystem::ColumnEven);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                    *tilemap_mesh_type = TilemapType::Hexagon(HexCoordSystem::ColumnOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                    *tilemap_mesh_type = TilemapType::Hexagon(HexCoordSystem::Column);
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
