use bevy::math::Vec4Swizzles;
use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use helpers::camera::movement as camera_movement;

// Press SPACE to change map type. Hover over mouse tiles to highlight their labels.
//
// The most important function here is the `highlight_tile_labels` systems, which shows how to
// convert a mouse cursor position into a tile position.

const MAP_SIDE_LENGTH: u32 = 4;
const TILE_SIZE_SQUARE: TilemapTileSize = TilemapTileSize { x: 50.0, y: 50.0 };
const TILE_SIZE_ISO: TilemapTileSize = TilemapTileSize { x: 100.0, y: 50.0 };
const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_SQUARE: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };
const GRID_SIZE_ISO: TilemapGridSize = TilemapGridSize { x: 100.0, y: 50.0 };
const LABEL_OFFSET_SQUARE: Vec2 = Vec2::new(0.0, 0.0);
const LABEL_OFFSET_HEX_ROW: Vec2 = Vec2::new(0.0, 0.0);
const LABEL_OFFSET_HEX_COL: Vec2 = Vec2::new(0.0, 0.0);
const LABEL_OFFSET_ISO: Vec2 = Vec2::new(0.0, 0.0);

fn get_label_offset(map_type: &TilemapType) -> Vec2 {
    match map_type {
        TilemapType::Square { .. } => LABEL_OFFSET_SQUARE,
        TilemapType::Hexagon(hex_coord_sys) => match hex_coord_sys {
            HexCoordSystem::Column | HexCoordSystem::ColumnEven | HexCoordSystem::ColumnOdd => {
                LABEL_OFFSET_HEX_COL
            }
            _ => LABEL_OFFSET_HEX_ROW,
        },
        TilemapType::Isometric { .. } => LABEL_OFFSET_ISO,
    }
}

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
    let tile_handle_iso: Handle<Image> = asset_server.load("bw-tile-iso.png");
    let tile_handle_hex_row: Handle<Image> = asset_server.load("bw-tile-hex-row.png");
    let tile_handle_hex_col: Handle<Image> = asset_server.load("bw-tile-hex-col.png");
    let tile_handle_square: Handle<Image> = asset_server.load("bw-tile-square.png");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.insert_resource(TileHandleIso(tile_handle_iso));
    commands.insert_resource(TileHandleHexCol(tile_handle_hex_col));
    commands.insert_resource(TileHandleHexRow(tile_handle_hex_row));
    commands.insert_resource(TileHandleSquare(tile_handle_square));
    commands.insert_resource(font);
}

// Generates the initial tilemap, which is a square grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_square: Res<TileHandleSquare>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let total_size = TilemapSize {
        x: MAP_SIDE_LENGTH,
        y: MAP_SIDE_LENGTH,
    };

    let mut tile_storage = TileStorage::empty(total_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap(
        TileTexture(0),
        total_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TILE_SIZE_SQUARE;
    let grid_size = GRID_SIZE_SQUARE;

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: total_size,
            storage: tile_storage,
            texture: TilemapTexture(tile_handle_square.clone()),
            tile_size,
            map_type: TilemapType::Square {
                diagonal_neighbors: false,
            },
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
    font_handle: Res<Handle<Font>>,
) {
    info!("Generating tile labels.");
    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (tilemap_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        info!("Found a tilemap!");
        let grid_size_vec: Vec2 = grid_size.into();
        let label_offset = get_label_offset(map_type) * grid_size_vec;

        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            info!("Found a tile: {tile_pos:?}");
            let tile_center = tile_pos.center_in_world(grid_size, map_type);
            let text_center = tile_center + label_offset;
            let mut transform = *tilemap_transform;
            transform.translation += text_center.extend(2.0);
            commands
                .entity(*tile_entity)
                .insert_bundle(Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..default()
                })
                .insert(TileLabel);
        }
    }
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
            let transform = Transform {
                translation: Vec2::new(-0.5 * window.width() / 2.0, 0.8 * window.height() / 2.0)
                    .extend(0.0),
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
    tile_handle_square: Res<TileHandleSquare>,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
    tile_handle_iso: Res<TileHandleIso>,
    font_handle: Res<Handle<Font>>,
    windows: Res<Windows>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (map_transform, mut map_type, mut grid_size, mut map_texture, mut tile_size) in
            tilemap_query.iter_mut()
        {
            match map_type.as_ref() {
                TilemapType::Square { .. } => {
                    *map_type = TilemapType::Isometric {
                        diagonal_neighbors: false,
                        coord_system: IsoCoordSystem::Diamond,
                    };
                    *map_texture = TilemapTexture((*tile_handle_iso).clone());
                    *tile_size = TILE_SIZE_ISO;
                    *grid_size = GRID_SIZE_ISO;
                }
                TilemapType::Isometric {
                    coord_system: IsoCoordSystem::Diamond,
                    ..
                } => {
                    *map_type = TilemapType::Isometric {
                        diagonal_neighbors: false,
                        coord_system: IsoCoordSystem::Staggered,
                    };
                    *map_texture = TilemapTexture((*tile_handle_iso).clone());
                    *tile_size = TILE_SIZE_ISO;
                    *grid_size = GRID_SIZE_ISO;
                }
                TilemapType::Isometric {
                    coord_system: IsoCoordSystem::Staggered,
                    ..
                } => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Row);
                    *map_texture = TilemapTexture((*tile_handle_hex_row).clone());
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
                    *map_texture = TilemapTexture((*tile_handle_hex_col).clone());
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
                    *map_type = TilemapType::Square {
                        diagonal_neighbors: false,
                    };
                    *map_texture = TilemapTexture((*tile_handle_square).clone());
                    *tile_size = TILE_SIZE_SQUARE;
                    *grid_size = GRID_SIZE_SQUARE;
                }
            }

            let grid_size_vec: Vec2 = (*grid_size).into();
            let label_offset = get_label_offset(&map_type) * grid_size_vec;

            for (tile_pos, mut tile_label_transform) in tile_label_q.iter_mut() {
                let tile_center = tile_pos.center_in_world(&grid_size, &map_type);
                let text_center = tile_center + label_offset;
                *tile_label_transform = *map_transform;
                tile_label_transform.translation += text_center.extend(2.0);
            }

            for window in windows.iter() {
                for (mut label_text, mut label_transform) in map_type_label_q.iter_mut() {
                    *label_transform = Transform {
                        translation: Vec2::new(
                            -0.5 * window.width() / 2.0,
                            0.8 * window.height() / 2.0,
                        )
                        .extend(0.0),
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
struct HighlightedLabel;

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

#[derive(Default)]
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
fn highlight_tile_labels(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    highlighted_tiles_q: Query<Entity, With<HighlightedLabel>>,
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
                .remove::<HighlightedLabel>();
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
                    commands.entity(tile_entity).insert(HighlightedLabel);
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
            title: String::from("Mouse Position to Tile Position"),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(CursorPos(Vec3::new(-100.0, -100.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_stage_after(
            StartupStage::Startup,
            "label_stage",
            SystemStage::parallel(),
        )
        .add_startup_system_to_stage(StartupStage::PreStartup, spawn_assets)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        // Must add a custom stage, rather than use `StartupStage::PostStartup`, because
        // `StartupStage::PostStartup` just doesn't seem to work.
        .add_startup_system_to_stage("label_stage", spawn_tile_labels)
        .add_startup_system_to_stage("label_stage", spawn_map_type_label)
        .add_system_to_stage(CoreStage::First, camera_movement)
        .add_system_to_stage(CoreStage::First, update_cursor_pos.after(camera_movement))
        .add_system_to_stage(CoreStage::Update, swap_map_type)
        .add_system_to_stage(
            CoreStage::Update,
            highlight_tile_labels.after(swap_map_type),
        )
        .run();
}
