use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = TilemapSize { x: 2, y: 1 };

    // Create a tilemap entity a little early.
    let tilemap_entity = commands.spawn().id();

    let mut tile_storage = TileStorage::empty(tilemap_size);

    // Spawn the elements of the tilemap.

    let tile_pos = TilePos { x: 0, y: 0 };
    let tile_entity = commands
        .spawn()
        .insert_bundle(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            ..Default::default()
        })
        .id();
    tile_storage.set(&tile_pos, Some(tile_entity));
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 16.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&tilemap_size, &tile_size, 0.0),
            ..Default::default()
        });
}

fn swap_pos(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut TilePos>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut pos in query.iter_mut() {
            if pos.x == 0 {
                pos.x = 1;
            } else {
                pos.x = 0;
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Update tile positions without despawning."),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(swap_pos)
        .run();
}
