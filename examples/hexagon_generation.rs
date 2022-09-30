use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use helpers::camera::movement as camera_movement;

// Press SPACE to change map type. Hover over a tile to highlight its label (red) and those of its
// neighbors (blue). Press and hold one of keys 0-5 to mark the neighbor in that direction (green).

// You can increase the MAP_SIDE_LENGTH, in order to test larger maps but just make sure that you run
// in release mode (`cargo run --release --example hexagon_generation`) otherwise things might be too
// slow.
const MAP_SIDE_LENGTH: u32 = 8;

const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };

#[derive(Component, Deref)]
pub struct TileHandleHexRow(Handle<Image>);

#[derive(Component, Deref)]
pub struct TileHandleHexCol(Handle<Image>);

#[derive(Component, Deref)]
pub struct TileHandleSquare(Handle<Image>);

#[derive(Component, Deref)]
pub struct TileHandleIso(Handle<Image>);

// Spawns different tiles that are used for this example.
fn spawn_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_handle_hex_row: Handle<Image> = asset_server.load("bw-tile-hex-row.png");
    let tile_handle_hex_col: Handle<Image> = asset_server.load("bw-tile-hex-col.png");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.insert_resource(TileHandleHexCol(tile_handle_hex_col));
    commands.insert_resource(TileHandleHexRow(tile_handle_hex_row));
    commands.insert_resource(font);
}

// Generates the initial tilemap, which is a square grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_hex_row: Res<TileHandleHexRow>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let total_size = TilemapSize {
        x: MAP_SIDE_LENGTH,
        y: MAP_SIDE_LENGTH,
    };

    let mut tile_storage = TileStorage::empty(total_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    let hex_coord_system = HexCoordSystem::Row;

    fill_tilemap_hexagon(
        TileTexture(0),
        TilePos {
            x: MAP_SIDE_LENGTH / 2,
            y: MAP_SIDE_LENGTH / 2,
        },
        MAP_SIDE_LENGTH / 2,
        hex_coord_system,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TILE_SIZE_HEX_ROW;
    let grid_size = GRID_SIZE_HEX_ROW;

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: total_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_handle_hex_row.clone()),
            tile_size,
            map_type: TilemapType::Hexagon(hex_coord_system),
            ..Default::default()
        });
}

#[derive(Component)]
pub struct MapTypeLabel;

// Generates the map type label: e.g. `Square { diagonal_neighbors: false }`
fn spawn_map_type_label(
    mut commands: Commands,
    font_handle: Res<Handle<Font>>,
    windows: Res<Windows>,
    map_type_q: Query<&TilemapType>,
) {
    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;

    for window in windows.iter() {
        for map_type in map_type_q.iter() {
            // Place the map type label somewhere in the top left side of the screen
            let transform = Transform {
                translation: Vec2::new(-0.5 * window.width() / 2.0, 0.8 * window.height() / 2.0)
                    .extend(1.0),
                ..Default::default()
            };
            commands
                .spawn_bundle(Text2dBundle {
                    text: Text::from_section(format!("{map_type:?}"), text_style.clone())
                        .with_alignment(text_alignment),
                    transform,
                    ..default()
                })
                .insert(MapTypeLabel);
        }
    }
}

// Swaps the map type, when user presses SPACE
#[allow(clippy::too_many_arguments)]
fn swap_map_type(
    mut commands: Commands,
    mut tilemap_query: Query<(
        Entity,
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
        &mut TileStorage,
    )>,
    keyboard_input: Res<Input<KeyCode>>,
    mut map_type_label_q: Query<
        (&mut Text, &mut Transform),
        (With<MapTypeLabel>, Without<TilemapType>),
    >,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
    font_handle: Res<Handle<Font>>,
    windows: Res<Windows>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (
            map_id,
            mut map_type,
            mut grid_size,
            mut map_texture,
            mut tile_size,
            mut tile_storage,
        ) in tilemap_query.iter_mut()
        {
            // Remove all previously spawned tiles.
            for possible_entity in tile_storage.iter_mut() {
                // see documentation for take to understand how it works:
                // https://doc.rust-lang.org/std/option/enum.Option.html#method.take
                if let Some(entity) = possible_entity.take() {
                    commands.entity(entity).despawn_recursive();
                }
            }

            let new_coord_sys = match map_type.as_ref() {
                TilemapType::Hexagon(HexCoordSystem::Row) => HexCoordSystem::Column,
                TilemapType::Hexagon(HexCoordSystem::Column) => HexCoordSystem::Row,
                _ => unreachable!(),
            };

            *map_type = TilemapType::Hexagon(new_coord_sys);

            if new_coord_sys == HexCoordSystem::Column {
                *map_texture = TilemapTexture::Single((*tile_handle_hex_col).clone());
                *tile_size = TILE_SIZE_HEX_COL;
                *grid_size = GRID_SIZE_HEX_COL;
            } else if new_coord_sys == HexCoordSystem::Row {
                *map_texture = TilemapTexture::Single((*tile_handle_hex_row).clone());
                *tile_size = TILE_SIZE_HEX_ROW;
                *grid_size = GRID_SIZE_HEX_ROW;
            }

            // Re-generate tiles in a hexagonal pattern.
            fill_tilemap_hexagon(
                TileTexture(0),
                TilePos {
                    x: MAP_SIDE_LENGTH / 2,
                    y: MAP_SIDE_LENGTH / 2,
                },
                MAP_SIDE_LENGTH / 2,
                new_coord_sys,
                TilemapId(map_id),
                &mut commands,
                &mut tile_storage,
            );

            for window in windows.iter() {
                for (mut label_text, mut label_transform) in map_type_label_q.iter_mut() {
                    *label_transform = Transform {
                        translation: Vec2::new(
                            -0.5 * window.width() / 2.0,
                            0.8 * window.height() / 2.0,
                        )
                        .extend(1.0),
                        ..Default::default()
                    };
                    *label_text = Text::from_section(
                        format!("{:?}", map_type.as_ref()),
                        TextStyle {
                            font: font_handle.clone(),
                            font_size: 20.0,
                            color: Color::BLACK,
                        },
                    )
                    .with_alignment(TextAlignment::CENTER);
                }
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from(
                "Hexagon Neighbors - Hover over a tile, and then press 0-5 to mark neighbors",
            ),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, spawn_assets)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_map_type_label)
        .add_system(camera_movement)
        .add_system(swap_map_type)
        .run();
}
