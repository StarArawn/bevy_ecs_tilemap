use bevy::{ecs::system::Resource, prelude::*};
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

#[derive(Deref, Resource)]
pub struct TileHandleHexRow(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleHexCol(Handle<Image>);

#[derive(Deref, Resource)]
pub struct FontHandle(Handle<Font>);

impl FromWorld for TileHandleHexCol {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("bw-tile-hex-col.png"))
    }
}
impl FromWorld for TileHandleHexRow {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("bw-tile-hex-row.png"))
    }
}
impl FromWorld for FontHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("fonts/FiraSans-Bold.ttf"))
    }
}

// Generates the initial tilemap, which is a square grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_hex_row: Res<TileHandleHexRow>) {
    commands.spawn(Camera2d);

    let map_size = TilemapSize {
        x: MAP_SIDE_LENGTH,
        y: MAP_SIDE_LENGTH,
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    let hex_coord_system = HexCoordSystem::Row;

    fill_tilemap_hexagon(
        TileTextureIndex(0),
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
    let map_type = TilemapType::Hexagon(hex_coord_system);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle_hex_row.clone()),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct MapTypeLabel;

// Generates the map type label: e.g. `Square { diagonal_neighbors: false }`
fn spawn_map_type_label(
    mut commands: Commands,
    font_handle: Res<FontHandle>,
    windows: Query<&Window>,
    map_type_q: Query<&TilemapType>,
) {
    for window in windows.iter() {
        for map_type in map_type_q.iter() {
            // Place the map type label somewhere in the top left side of the screen
            let transform = Transform {
                translation: Vec2::new(-0.5 * window.width() / 2.0, 0.8 * window.height() / 2.0)
                    .extend(1.0),
                ..Default::default()
            };
            commands.spawn((
                Text2d::new(format!("{map_type:?}")),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(JustifyText::Center),
                transform,
                MapTypeLabel,
            ));
        }
    }
}

// Swaps the map type, when user presses SPACE
#[allow(clippy::too_many_arguments)]
fn swap_map_type(
    mut commands: Commands,
    mut tilemap_query: Query<(
        Entity,
        &mut Transform,
        &TilemapSize,
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
        &mut TileStorage,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut map_type_label_q: Query<&mut Text2d, With<MapTypeLabel>>,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (
            map_id,
            mut map_transform,
            map_size,
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

            *map_transform = get_tilemap_center_transform(map_size, &grid_size, &map_type, 0.0);

            // Re-generate tiles in a hexagonal pattern.
            fill_tilemap_hexagon(
                TileTextureIndex(0),
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

            for mut label_text in map_type_label_q.iter_mut() {
                label_text.0 = format!("{:?}", map_type.as_ref());
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Generating a hexagonal hex map"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .init_resource::<TileHandleHexCol>()
        .init_resource::<TileHandleHexRow>()
        .init_resource::<FontHandle>()
        .add_systems(
            Startup,
            (spawn_tilemap, apply_deferred, spawn_map_type_label).chain(),
        )
        .add_systems(Update, camera_movement)
        .add_systems(Update, swap_map_type)
        .run();
}
