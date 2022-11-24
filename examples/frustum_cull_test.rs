use bevy::log::{Level, LogPlugin};
use bevy::{ecs::system::Resource, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::FrustumCulling;
mod helpers;
use helpers::camera::movement as camera_movement;

// Press SPACE to change map type.
//
// The most important code here is the setting that adds the `FrustumCull` component, and the
// code that enables logging for trace events from bevy_ecs_tilemap.
//
// You can increase the MAP_SIDE_LENGTH, in order to test that mouse picking works for larger maps,
// but just make sure that you run in release mode (`cargo run --release --example mouse_to_tile`)
// otherwise things might be too slow.

// We want to trigger render chunking, which has minimum size 64x64.
const MAP_SIDE_LENGTH_X: u32 = 64;
const MAP_SIDE_LENGTH_Y: u32 = 64;

const TILE_SIZE_SQUARE: TilemapTileSize = TilemapTileSize { x: 50.0, y: 50.0 };
const TILE_SIZE_ISO: TilemapTileSize = TilemapTileSize { x: 100.0, y: 50.0 };
const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_SQUARE: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };
const GRID_SIZE_ISO: TilemapGridSize = TilemapGridSize { x: 100.0, y: 50.0 };

#[derive(Deref, Resource)]
pub struct TileHandleHexRow(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleHexCol(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleSquare(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleIso(Handle<Image>);

#[derive(Deref, Resource)]
pub struct FontHandle(Handle<Font>);

// Spawns different tiles that are used for this example.
fn spawn_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_handle_iso: Handle<Image> = asset_server.load("bw-tile-iso.png");
    let tile_handle_hex_row: Handle<Image> = asset_server.load("bw-tile-hex-row.png");
    let tile_handle_hex_col: Handle<Image> = asset_server.load("bw-tile-hex-col.png");
    let tile_handle_square: Handle<Image> = asset_server.load("bw-tile-square.png");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.insert_resource(TileHandleIso(tile_handle_iso));
    commands.insert_resource(TileHandleHexCol(tile_handle_hex_col));
    commands.insert_resource(TileHandleHexRow(tile_handle_hex_row));
    commands.insert_resource(TileHandleSquare(tile_handle_square));
    commands.insert_resource(FontHandle(font));
}

// Generates the initial tilemap, which is a square grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_square: Res<TileHandleSquare>) {
    commands.spawn(Camera2dBundle::default());

    let map_size = TilemapSize {
        // Render chunks are of size 64, so let's create two render chunks
        x: MAP_SIDE_LENGTH_X * 2,
        y: MAP_SIDE_LENGTH_Y,
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TILE_SIZE_SQUARE;
    let grid_size = GRID_SIZE_SQUARE;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle_square.clone()),
        tile_size,
        map_type: TilemapType::Square,
        // The default behaviour is `FrustumCulling(true)`, but we supply this explicitly here
        // for the purposes of the example.
        frustum_culling: FrustumCulling(true),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct MapTypeLabel;

// Generates the map type label: e.g. `Square { diagonal_neighbors: false }`
fn spawn_map_type_label(
    mut commands: Commands,
    font_handle: Res<FontHandle>,
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
                .spawn(Text2dBundle {
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
    mut tilemap_query: Query<(
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
    )>,
    keyboard_input: Res<Input<KeyCode>>,
    mut map_type_label_q: Query<
        (&mut Text, &mut Transform),
        (With<MapTypeLabel>, Without<TilemapType>),
    >,
    tile_handle_square: Res<TileHandleSquare>,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
    tile_handle_iso: Res<TileHandleIso>,
    font_handle: Res<FontHandle>,
    windows: Res<Windows>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut map_type, mut grid_size, mut map_texture, mut tile_size) in
            tilemap_query.iter_mut()
        {
            match map_type.as_ref() {
                TilemapType::Square { .. } => {
                    *map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);
                    *map_texture = TilemapTexture::Single((*tile_handle_iso).clone());
                    *tile_size = TILE_SIZE_ISO;
                    *grid_size = GRID_SIZE_ISO;
                }
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    *map_type = TilemapType::Isometric(IsoCoordSystem::Staggered);
                    *map_texture = TilemapTexture::Single((*tile_handle_iso).clone());
                    *tile_size = TILE_SIZE_ISO;
                    *grid_size = GRID_SIZE_ISO;
                }
                TilemapType::Isometric(IsoCoordSystem::Staggered) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Row);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_row).clone());
                    *tile_size = TILE_SIZE_HEX_ROW;
                    *grid_size = GRID_SIZE_HEX_ROW;
                }
                TilemapType::Hexagon(HexCoordSystem::Row) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowEven);
                }
                TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Column);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_col).clone());
                    *tile_size = TILE_SIZE_HEX_COL;
                    *grid_size = GRID_SIZE_HEX_COL;
                }
                TilemapType::Hexagon(HexCoordSystem::Column) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::ColumnEven);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::ColumnOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                    *map_type = TilemapType::Square;
                    *map_texture = TilemapTexture::Single((*tile_handle_square).clone());
                    *tile_size = TILE_SIZE_SQUARE;
                    *grid_size = GRID_SIZE_SQUARE;
                }
            }

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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Frustum cull test"),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                // For this example, we want to turn up logging to show trace level events for bevy_ecs_tilemap
                .set(LogPlugin {
                    // Everything else should be set to Level::ERROR
                    level: Level::ERROR,
                    // except for bevy_ecs_tilemap
                    filter: "bevy_ecs_tilemap=trace".into(),
                }),
        )
        .add_plugin(TilemapPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, spawn_assets)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_map_type_label)
        .add_system_to_stage(CoreStage::First, camera_movement)
        .add_system_to_stage(CoreStage::Update, swap_map_type)
        .run();
}
