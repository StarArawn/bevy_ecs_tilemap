use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

/// This is like `fill_tilemap_rect`, a function in `helpers.rs`, except that it
/// also sets a color.
///
/// Fills a rectangular region with the given tile.
///
/// The rectangular region is defined by an `origin` in `TilePos`, and a size
/// in tiles (`TilemapSize`).
pub fn fill_tilemap_rect_color(
    tile_texture: TileTexture,
    origin: TilePos,
    size: TilemapSize,
    color: Color,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos {
                x: origin.x + x,
                y: origin.y + y,
            };

            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: tile_texture,
                    color: TileColor(color),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

// Side length of a colored quadrant (in "number of tiles").
const QUADRANT_SIDE_LENGTH: u32 = 64;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

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

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos { x: 0, y: 0 },
        quadrant_size,
        Color::rgba(1.0, 0.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: 0,
        },
        quadrant_size,
        Color::rgba(0.0, 1.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos {
            x: 0,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        Color::rgba(0.0, 0.0, 1.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        Color::rgba(1.0, 1.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
            size: total_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            map_type: TilemapType::Square {
                neighbors_include_diagonals: false,
            },
            ..Default::default()
        });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Color Example"),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .run();
}
