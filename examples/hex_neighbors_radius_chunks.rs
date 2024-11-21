use bevy::{color::palettes, math::Vec4Swizzles, prelude::*};
use bevy_ecs_tilemap::{helpers::hex_grid::offset::*, prelude::*};
mod helpers;
use helpers::camera::movement as camera_movement;

const CHUNK_MAP_SIDE_LENGTH_X: u32 = 4;
const CHUNK_MAP_SIDE_LENGTH_Y: u32 = 4;

const CHUNKS_X: i32 = 2;
const CHUNKS_Y: i32 = 2;

const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };

const MAX_RADIUS: u32 = 6;

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

#[derive(Deref, Component, Clone, Copy)]
pub struct ChunkPos(IVec2);

fn chunk_in_world_position(pos: IVec2, map_type: TilemapType) -> Vec3 {
    let tile_size = match map_type {
        TilemapType::Hexagon(HexCoordSystem::RowEven)
        | TilemapType::Hexagon(HexCoordSystem::RowOdd) => TILE_SIZE_HEX_ROW,
        TilemapType::Hexagon(HexCoordSystem::ColumnEven)
        | TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => TILE_SIZE_HEX_COL,
        _ => unreachable!(),
    };
    let grid_size = match map_type {
        TilemapType::Hexagon(HexCoordSystem::RowEven)
        | TilemapType::Hexagon(HexCoordSystem::RowOdd) => GRID_SIZE_HEX_ROW,
        TilemapType::Hexagon(HexCoordSystem::ColumnEven)
        | TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => GRID_SIZE_HEX_COL,
        _ => unreachable!(),
    };
    if matches!(
        map_type,
        TilemapType::Hexagon(HexCoordSystem::RowEven)
            | TilemapType::Hexagon(HexCoordSystem::RowOdd)
    ) {
        Vec3::new(
            tile_size.x * CHUNK_MAP_SIDE_LENGTH_X as f32 * pos.x as f32,
            TilePos {
                x: 0,
                y: CHUNK_MAP_SIDE_LENGTH_Y,
            }
            .center_in_world(&grid_size, &map_type)
            .y * pos.y as f32,
            0.0,
        )
    } else if matches!(
        map_type,
        TilemapType::Hexagon(HexCoordSystem::ColumnEven)
            | TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
    ) {
        Vec3::new(
            TilePos {
                x: CHUNK_MAP_SIDE_LENGTH_X,
                y: 0,
            }
            .center_in_world(&grid_size, &map_type)
            .x * pos.x as f32,
            tile_size.y * CHUNK_MAP_SIDE_LENGTH_Y as f32 * pos.y as f32,
            0.0,
        )
    } else {
        unreachable!()
    }
}

fn hex_pos_from_tile_pos(
    tile_pos: &TilePos,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    map_transform: &Transform,
) -> IVec2 {
    let tile_translation =
        *map_transform * tile_pos.center_in_world(grid_size, map_type).extend(0.0);
    match map_type {
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            let pos = RowEvenPos::from_world_pos(&tile_translation.truncate(), grid_size);
            IVec2 { x: pos.q, y: pos.r }
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            let pos = RowOddPos::from_world_pos(&tile_translation.truncate(), grid_size);
            IVec2 { x: pos.q, y: pos.r }
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            let pos = ColEvenPos::from_world_pos(&tile_translation.truncate(), grid_size);
            IVec2 { x: pos.q, y: pos.r }
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            let pos = ColOddPos::from_world_pos(&tile_translation.truncate(), grid_size);
            IVec2 { x: pos.q, y: pos.r }
        }
        _ => unreachable!(),
    }
}

fn hex_neighbors_radius(hex_pos: IVec2, radius: u32, map_type: &TilemapType) -> Vec<IVec2> {
    let neighbors = generate_hexagon(
        match map_type {
            TilemapType::Hexagon(HexCoordSystem::RowEven) => RowEvenPos {
                q: hex_pos.x,
                r: hex_pos.y,
            }
            .into(),
            TilemapType::Hexagon(HexCoordSystem::RowOdd) => RowOddPos {
                q: hex_pos.x,
                r: hex_pos.y,
            }
            .into(),
            TilemapType::Hexagon(HexCoordSystem::ColumnEven) => ColEvenPos {
                q: hex_pos.x,
                r: hex_pos.y,
            }
            .into(),
            TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => ColOddPos {
                q: hex_pos.x,
                r: hex_pos.y,
            }
            .into(),
            _ => unreachable!(),
        },
        radius,
    );
    neighbors
        .into_iter()
        .map(|axial_pos| match map_type {
            TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                let pos = RowEvenPos::from(axial_pos);
                IVec2 { x: pos.q, y: pos.r }
            }
            TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
                let pos = RowOddPos::from(axial_pos);
                IVec2 { x: pos.q, y: pos.r }
            }
            TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                let pos = ColEvenPos::from(axial_pos);
                IVec2 { x: pos.q, y: pos.r }
            }
            TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                let pos = ColOddPos::from(axial_pos);
                IVec2 { x: pos.q, y: pos.r }
            }
            _ => unreachable!(),
        })
        .collect()
}

fn hex_neighbors_radius_from_tile_pos(
    tile_pos: &TilePos,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    map_transform: &Transform,
    radius: u32,
) -> Vec<IVec2> {
    let hex_pos = hex_pos_from_tile_pos(tile_pos, grid_size, map_type, map_transform);
    hex_neighbors_radius(hex_pos, radius, map_type)
}

fn spawn_chunks(mut commands: Commands, tile_handle_hex_row: Res<TileHandleHexRow>) {
    commands.spawn(Camera2d);

    let map_size = TilemapSize {
        x: CHUNK_MAP_SIDE_LENGTH_X,
        y: CHUNK_MAP_SIDE_LENGTH_Y,
    };

    let tile_size = TILE_SIZE_HEX_ROW;
    let grid_size = GRID_SIZE_HEX_ROW;
    let map_type = TilemapType::Hexagon(HexCoordSystem::RowEven);

    // Makes it so chunks spawn around the world center
    let lower_bound_x = -(CHUNKS_X / 2);
    let lower_bound_y = -(CHUNKS_Y / 2);
    for chunk_x in lower_bound_x..(CHUNKS_X + lower_bound_x) {
        for chunk_y in lower_bound_y..(CHUNKS_Y + lower_bound_y) {
            let chunk_pos = ChunkPos(IVec2 {
                x: chunk_x,
                y: chunk_y,
            });

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

            commands
                .entity(tilemap_entity)
                .insert(TilemapBundle {
                    grid_size,
                    size: map_size,
                    storage: tile_storage,
                    texture: TilemapTexture::Single(tile_handle_hex_row.clone()),
                    tile_size,
                    map_type,
                    transform: Transform::from_translation(chunk_in_world_position(
                        *chunk_pos, map_type,
                    )),
                    ..Default::default()
                })
                .insert(chunk_pos);
        }
    }
}

fn swap_map_type(
    mut tilemap_query: Query<(
        &mut Transform,
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
        &TileStorage,
        &ChunkPos,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    tile_label_q: Query<(Entity, &TileLabel, &TilePos), Without<TilemapType>>,
    mut transform_q: Query<(&mut Transform, &mut Text2d), Without<TilemapType>>,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (
            mut map_transform,
            mut map_type,
            mut grid_size,
            mut map_texture,
            mut tile_size,
            tile_storage,
            chunk_pos,
        ) in tilemap_query.iter_mut()
        {
            match map_type.as_ref() {
                TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowOdd)
                }
                TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::ColumnEven);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_col).clone());
                    *tile_size = TILE_SIZE_HEX_COL;
                    *grid_size = GRID_SIZE_HEX_COL;
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::ColumnOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowEven);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_row).clone());
                    *tile_size = TILE_SIZE_HEX_ROW;
                    *grid_size = GRID_SIZE_HEX_ROW;
                }
                _ => unreachable!(),
            }

            *map_transform =
                Transform::from_translation(chunk_in_world_position(**chunk_pos, *map_type));

            for (tile_entity, label, tile_pos) in tile_label_q.iter() {
                if let Ok((mut tile_label_transform, mut tile_label_text)) =
                    transform_q.get_mut(label.0)
                {
                    if let Some(ent) = tile_storage.checked_get(tile_pos) {
                        if ent == tile_entity {
                            let tile_center =
                                tile_pos.center_in_world(&grid_size, &map_type).extend(1.0);
                            *tile_label_transform =
                                *map_transform * Transform::from_translation(tile_center);
                            let hex_pos = hex_pos_from_tile_pos(
                                tile_pos,
                                &grid_size,
                                &map_type,
                                &map_transform,
                            );
                            tile_label_text.0 = format!("{}, {}", hex_pos.x, hex_pos.y);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct TileLabel(Entity);

fn spawn_tile_labels(
    mut commands: Commands,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    tile_q: Query<&TilePos>,
    font_handle: Res<FontHandle>,
) {
    for (map_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.0);
            let transform = *map_transform * Transform::from_translation(tile_center);

            let hex_pos = hex_pos_from_tile_pos(tile_pos, grid_size, map_type, map_transform);

            let label_entity = commands
                .spawn((
                    Text2d(format!("{}, {}", hex_pos.x, hex_pos.y)),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(JustifyText::Center),
                    transform,
                ))
                .id();
            commands
                .entity(*tile_entity)
                .insert(TileLabel(label_entity));
        }
    }
}

#[derive(Component)]
struct Hovered;

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
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
    tile_label_q: Query<&TileLabel>,
    mut text_q: Query<&mut TextColor>,
) {
    // Un-highlight any previously highlighted tile labels.
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        if let Ok(label) = tile_label_q.get(highlighted_tile_entity) {
            if let Ok(mut text_color) = text_q.get_mut(label.0) {
                text_color.0 = Color::BLACK;
                commands.entity(highlighted_tile_entity).remove::<Hovered>();
            }
        }
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        let cursor_pos = cursor_pos.0;
        let cursor_pos_in_map_pos = {
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_pos_in_map_pos, map_size, grid_size, map_type)
        {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(label) = tile_label_q.get(tile_entity) {
                    if let Ok(mut text_color) = text_q.get_mut(label.0) {
                        text_color.0 = palettes::tailwind::RED_600.into();
                        commands.entity(tile_entity).insert(Hovered);
                    }
                }
            }
        }
    }
}

#[derive(Deref, Resource)]
struct HighlightRadius(u32);
impl Default for HighlightRadius {
    fn default() -> Self {
        Self(2)
    }
}

fn update_radius(mut radius: ResMut<HighlightRadius>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        radius.0 = std::cmp::min(MAX_RADIUS, radius.0 + 1);
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        radius.0 = std::cmp::max(1, radius.0 - 1);
    }
}

#[derive(Component)]
struct NeighborHighlight;

#[allow(clippy::too_many_arguments)]
// Highlight neighbors of a tile in a radius
fn highlight_neighbor_labels(
    mut commands: Commands,
    tilemap_query: Query<(&TilemapType, &TilemapGridSize, &TileStorage, &Transform)>,
    highlighted_tiles_q: Query<Entity, With<NeighborHighlight>>,
    hovered_tiles_q: Query<(Entity, &TilePos), With<Hovered>>,
    tiles_q: Query<&TilePos, Without<Hovered>>,
    tile_label_q: Query<&TileLabel>,
    mut text_q: Query<&mut TextColor>,
    radius: Res<HighlightRadius>,
) {
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        if let Ok(label) = tile_label_q.get(highlighted_tile_entity) {
            if let Ok(mut text_color) = text_q.get_mut(label.0) {
                text_color.0 = Color::BLACK;
                commands
                    .entity(highlighted_tile_entity)
                    .remove::<NeighborHighlight>();
            }
        }
    }

    let mut neighbors: Option<Vec<IVec2>> = None;

    for (map_type, grid_size, tile_storage, map_t) in tilemap_query.iter() {
        for (hovered_tile_entity, hovered_tile_pos) in hovered_tiles_q.iter() {
            if let Some(ent) = tile_storage.checked_get(hovered_tile_pos) {
                if ent == hovered_tile_entity {
                    neighbors = Some(hex_neighbors_radius_from_tile_pos(
                        hovered_tile_pos,
                        grid_size,
                        map_type,
                        map_t,
                        **radius,
                    ));
                }
            }
        }
    }

    if let Some(neighbors) = neighbors {
        for (map_type, grid_size, tile_storage, map_t) in tilemap_query.iter() {
            for tile_entity in tile_storage.iter().flatten() {
                if let Ok(tile_pos) = tiles_q.get(*tile_entity) {
                    let tile_hex_pos = hex_pos_from_tile_pos(tile_pos, grid_size, map_type, map_t);
                    if neighbors.contains(&tile_hex_pos) {
                        if let Ok(label) = tile_label_q.get(*tile_entity) {
                            if let Ok(mut text_color) = text_q.get_mut(label.0) {
                                text_color.0 = palettes::tailwind::BLUE_600.into();
                                commands.entity(*tile_entity).insert(NeighborHighlight);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct SpawnChunksSet;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from(
                            "Hexagon radius - Hover over a tile to highlight tiles in a radius, use up and down arrow to change radius",
                        ),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .init_resource::<CursorPos>()
        .init_resource::<HighlightRadius>()
        .init_resource::<TileHandleHexCol>()
        .init_resource::<TileHandleHexRow>()
        .init_resource::<FontHandle>()
        .add_systems(Startup, (spawn_chunks, apply_deferred).chain().in_set(SpawnChunksSet))
        .add_systems(Startup, spawn_tile_labels.after(SpawnChunksSet))
        .add_systems(First, (camera_movement, update_cursor_pos).chain())
        .add_systems(Update, swap_map_type)
        .add_systems(Update, hover_highlight_tile_label.after(swap_map_type))
        .add_systems(Update, update_radius.after(hover_highlight_tile_label))
        .add_systems(Update, highlight_neighbor_labels.after(update_radius))
        .run();
}
