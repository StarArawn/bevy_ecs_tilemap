use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::{HexDirection, HexNeighbors};
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use helpers::camera::movement as camera_movement;

// Press SPACE to change map type. Hover over a tile to highlight its label (red) and those of its
// neighbors (blue). Press and hold one of keys 0-5 to mark the neighbor in that direction (green).

// You can increase the MAP_SIDE_LENGTH, in order to test that mouse picking works for larger maps,
// but just make sure that you run in release mode (`cargo run --release --example mouse_to_tile`)
// otherwise things might be too slow.
const MAP_SIDE_LENGTH_X: u32 = 4;
const MAP_SIDE_LENGTH_Y: u32 = 4;

const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };

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
    let tile_handle_hex_row: Handle<Image> = asset_server.load("bw-tile-hex-row.png");
    let tile_handle_hex_col: Handle<Image> = asset_server.load("bw-tile-hex-col.png");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.insert_resource(TileHandleHexCol(tile_handle_hex_col));
    commands.insert_resource(TileHandleHexRow(tile_handle_hex_row));
    commands.insert_resource(FontHandle(font));
}

// Generates the initial tilemap, which is a square grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_hex_row: Res<TileHandleHexRow>) {
    commands.spawn(Camera2dBundle::default());

    let map_size = TilemapSize {
        x: MAP_SIDE_LENGTH_X,
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

    let tile_size = TILE_SIZE_HEX_ROW;
    let grid_size = GRID_SIZE_HEX_ROW;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle_hex_row.clone()),
        tile_size,
        map_type: TilemapType::Hexagon(HexCoordSystem::Row),
        ..Default::default()
    });
}

#[derive(Component)]
struct TileLabel;

// Generates tile position labels of the form: `(tile_pos.x, tile_pos.y)`
fn spawn_tile_labels(
    mut commands: Commands,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    tile_q: Query<&mut TilePos>,
    font_handle: Res<FontHandle>,
) {
    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (map_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.0);
            let transform = *map_transform * Transform::from_translation(tile_center);
            commands.entity(*tile_entity).insert((
                Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..default()
                },
                TileLabel,
            ));
        }
    }
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
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(format!("{map_type:?}"), text_style.clone())
                        .with_alignment(text_alignment),
                    transform,
                    ..default()
                },
                MapTypeLabel,
            ));
        }
    }
}

// Swaps the map type, when user presses SPACE
#[allow(clippy::too_many_arguments)]
fn swap_map_type(
    mut tilemap_query: Query<(
        &Transform,
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
    )>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tile_label_q: Query<
        (&TilePos, &mut Transform),
        (With<TileLabel>, Without<MapTypeLabel>, Without<TilemapType>),
    >,
    mut map_type_label_q: Query<
        (&mut Text, &mut Transform),
        (With<MapTypeLabel>, Without<TileLabel>, Without<TilemapType>),
    >,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
    font_handle: Res<FontHandle>,
    windows: Res<Windows>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (map_transform, mut map_type, mut grid_size, mut map_texture, mut tile_size) in
            tilemap_query.iter_mut()
        {
            match map_type.as_ref() {
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
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Row);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_row).clone());
                    *tile_size = TILE_SIZE_HEX_ROW;
                    *grid_size = GRID_SIZE_HEX_ROW;
                }
                _ => unreachable!(),
            }

            for (tile_pos, mut tile_label_transform) in tile_label_q.iter_mut() {
                let tile_center = tile_pos.center_in_world(&grid_size, &map_type).extend(1.0);
                *tile_label_transform = *map_transform * Transform::from_translation(tile_center);
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

#[derive(Component)]
struct Hovered;

// Converts the cursor position into a world position, taking into account any transforms applied
// the camera.
pub fn cursor_pos_in_world(
    windows: &Windows,
    cursor_pos: Vec2,
    cam_t: &Transform,
    cam: &Camera,
) -> Vec3 {
    let window = windows.primary();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}

#[derive(Default, Resource)]
pub struct CursorPos(Vec3);

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            *cursor_pos = CursorPos(cursor_pos_in_world(
                &windows,
                cursor_moved.position,
                cam_t,
                cam,
            ));
        }
    }
}

// This is where we check which tile the cursor is hovered over.
fn hover_highlight_tile_label(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    highlighted_tiles_q: Query<Entity, With<Hovered>>,
    mut tile_label_q: Query<&mut Text, (With<TileLabel>, Without<MapTypeLabel>)>,
) {
    // Un-highlight any previously highlighted tile labels.
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        if let Ok(mut tile_text) = tile_label_q.get_mut(highlighted_tile_entity) {
            for mut section in tile_text.sections.iter_mut() {
                section.style.color = Color::BLACK;
            }
            commands.entity(highlighted_tile_entity).remove::<Hovered>();
        }
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec3 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut tile_text) = tile_label_q.get_mut(tile_entity) {
                    for mut section in tile_text.sections.iter_mut() {
                        section.style.color = Color::RED;
                    }
                    commands.entity(tile_entity).insert(Hovered);
                }
            }
        }
    }
}

#[derive(Component)]
struct NeighborHighlight;

// Swaps the map type, when user presses SPACE
#[allow(clippy::too_many_arguments)]
fn highlight_neighbor_label(
    mut commands: Commands,
    tilemap_query: Query<(&TilemapType, &TilemapSize, &TileStorage)>,
    keyboard_input: Res<Input<KeyCode>>,
    highlighted_tiles_q: Query<Entity, With<NeighborHighlight>>,
    hovered_tiles_q: Query<&TilePos, With<Hovered>>,
    mut tile_label_q: Query<&mut Text, (With<TileLabel>, Without<MapTypeLabel>)>,
) {
    // Un-highlight any previously highlighted tile labels.
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        if let Ok(mut tile_text) = tile_label_q.get_mut(highlighted_tile_entity) {
            for mut section in tile_text.sections.iter_mut() {
                section.style.color = Color::BLACK;
            }
            commands
                .entity(highlighted_tile_entity)
                .remove::<NeighborHighlight>();
        }
    }

    for (map_type, map_size, tile_storage) in tilemap_query.iter() {
        let hex_coord_sys = if let TilemapType::Hexagon(hex_coord_sys) = map_type {
            hex_coord_sys
        } else {
            continue;
        };

        for hovered_tile_pos in hovered_tiles_q.iter() {
            let neighboring_positions =
                HexNeighbors::get_neighboring_positions(hovered_tile_pos, map_size, hex_coord_sys);

            for neighbor_pos in neighboring_positions.iter() {
                // We want to ensure that the tile position lies within the tile map, so we do a
                // `checked_get`.
                if let Some(tile_entity) = tile_storage.checked_get(neighbor_pos) {
                    if let Ok(mut tile_text) = tile_label_q.get_mut(tile_entity) {
                        for mut section in tile_text.sections.iter_mut() {
                            section.style.color = Color::BLUE;
                        }
                        commands.entity(tile_entity).insert(NeighborHighlight);
                    }
                }
            }

            let selected_hex_direction = if keyboard_input.pressed(KeyCode::Key0) {
                Some(HexDirection::Zero)
            } else if keyboard_input.pressed(KeyCode::Key1) {
                Some(HexDirection::One)
            } else if keyboard_input.pressed(KeyCode::Key2) {
                Some(HexDirection::Two)
            } else if keyboard_input.pressed(KeyCode::Key3) {
                Some(HexDirection::Three)
            } else if keyboard_input.pressed(KeyCode::Key4) {
                Some(HexDirection::Four)
            } else if keyboard_input.pressed(KeyCode::Key5) {
                Some(HexDirection::Five)
            } else {
                None
            };

            if let Some(hex_direction) = selected_hex_direction {
                let tile_pos = match map_type {
                    TilemapType::Hexagon(hex_coord_sys) => {
                        // Get the neighbor in a particular direction.
                        // This function does not check to see if the calculated neighbor lies
                        // within the tile map.
                        hex_direction.offset(hovered_tile_pos, *hex_coord_sys)
                    }
                    _ => unreachable!(),
                };

                // We want to ensure that the tile position lies within the tile map, so we do a
                // `checked_get`.
                if let Some(tile_entity) = tile_storage.checked_get(&tile_pos) {
                    if let Ok(mut tile_text) = tile_label_q.get_mut(tile_entity) {
                        for mut section in tile_text.sections.iter_mut() {
                            section.style.color = Color::GREEN;
                        }
                        commands.entity(tile_entity).insert(NeighborHighlight);
                    }
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
                        title: String::from(
                            "Hexagon Neighbors - Hover over a tile, and then press 0-5 to mark neighbors",
                        ),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        .insert_resource(CursorPos(Vec3::new(-100.0, -100.0, 0.0)))
        .add_plugin(TilemapPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, spawn_assets)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tile_labels)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_map_type_label)
        .add_system_to_stage(CoreStage::First, camera_movement)
        .add_system_to_stage(CoreStage::First, update_cursor_pos.after(camera_movement))
        .add_system_to_stage(CoreStage::Update, swap_map_type)
        .add_system_to_stage(
            CoreStage::Update,
            hover_highlight_tile_label.after(swap_map_type),
        )
        .add_system_to_stage(
            CoreStage::Update,
            highlight_neighbor_label.after(hover_highlight_tile_label),
        )
        .run();
}
