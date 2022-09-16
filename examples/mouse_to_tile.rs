use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

const MAP_SIDE_LENGTH: u32 = 4;
const TILE_SIZE_SQUARE: TilemapTileSize = TilemapTileSize { x: 50.0, y: 50.0 };
const TILE_SIZE_ISO: TilemapTileSize = TilemapTileSize { x: 100.0, y: 50.0 };
const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_SQUARE: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_ISO: TilemapGridSize = TilemapGridSize { x: 100.0, y: 50.0 };
const LABEL_OFFSET_SQUARE: Vec2 = Vec2::new(0.5, 0.5);
const LABEL_OFFSET_HEX_ROW: Vec2 = Vec2::new(0.0, 0.0);
const LABEL_OFFSET_HEX_COL: Vec2 = Vec2::new(0.0, 0.0);
const LABEL_OFFSET_ISO: Vec2 = Vec2::new(0.5, 0.5);

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

fn spawn_tilemap(mut commands: Commands, tile_handle_square: Res<TileHandleSquare>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // In total, there will be `(QUADRANT_SIDE_LENGTH * 2) * (QUADRANT_SIDE_LENGTH * 2)` tiles.
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

fn spawn_tile_labels(
    mut commands: Commands,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    tile_q: Query<&mut TilePos>,
    font_handle: Res<Handle<Font>>,
) {
    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (tilemap_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        let grid_size_vec: Vec2 = grid_size.into();
        let label_offset = get_label_offset(map_type) * grid_size_vec;

        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos.to_world_pos(grid_size, map_type);
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

fn spawn_instructions() {}

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
                let tile_center = tile_pos.to_world_pos(&grid_size, &map_type);
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

fn highlight_tile_labels(
    mut commands: Commands,
    windows: Res<Windows>,
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

    for window in windows.iter() {
        if let Some(position) = window.cursor_position() {
            let centered_position =
                position - Vec2::new(window.width() / 2.0, window.height() / 2.0);
            for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
                let map_translation = map_transform.translation.truncate();
                let world_pos = centered_position - map_translation;
                info!("map_translation: {map_translation}");
                info!("World position: {}", &world_pos);
                if let Some(tile_pos) =
                    TilePos::from_world_pos(&world_pos, map_size, grid_size, map_type)
                {
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
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Hexagon Row Example"),
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, spawn_assets)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tile_labels)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_map_type_label)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_instructions)
        .add_system(helpers::camera::movement)
        .add_system(swap_map_type)
        .add_system(highlight_tile_labels)
        .run();
}
