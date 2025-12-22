//! # Demonstrate `TilemapAnchor` component
//!
//! This example demonstrates the various `TilemapAnchor` variants with all of
//! the map types and a handful of grid scale factors.
//!
//! ## Controls
//! Press SPACE to change map anchor.
//! Press ENTER to change map type.
//! Press TAB to change the grid size.

use bevy::{image::Image, sprite::Anchor};
use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use helpers::anchor::rotate_right;
use helpers::camera::movement as camera_movement;
use rand::rng;
use rand::seq::IndexedRandom;

const MAP_SIDE_LENGTH_X: u32 = 4;
const MAP_SIDE_LENGTH_Y: u32 = 4;
const TILE_SIZE_SQUARE: TilemapTileSize = TilemapTileSize { x: 50.0, y: 50.0 };
const TILE_SIZE_ISO: TilemapTileSize = TilemapTileSize { x: 100.0, y: 50.0 };
const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const TILE_SIZE_HEX_COL: TilemapTileSize = TilemapTileSize { x: 58.0, y: 50.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_COL: TilemapGridSize = TilemapGridSize { x: 58.0, y: 50.0 };
const GRID_SIZE_SQUARE: TilemapGridSize = TilemapGridSize { x: 50.0, y: 50.0 };
const GRID_SIZE_ISO: TilemapGridSize = TilemapGridSize { x: 100.0, y: 50.0 };

#[derive(Deref, Resource)]
pub struct TileHandleHexRow(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleHexCol(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleSquare(Handle<Image>);

#[derive(Deref, Resource)]
pub struct TileHandleIso(Handle<Image>);

#[derive(Resource)]
pub struct GridScale {
    scales: Vec<f32>,
    index: usize,
}

impl GridScale {
    fn new(scales: Vec<f32>) -> Self {
        GridScale { index: 0, scales }
    }
    fn get(&self) -> f32 {
        self.scales[self.index]
    }
    fn next(&mut self) {
        self.index = (self.index + 1) % self.scales.len();
    }
    fn apply(&self, grid_size: TilemapGridSize) -> TilemapGridSize {
        let scale = self.get();
        TilemapGridSize::new(grid_size.x * scale, grid_size.y * scale)
    }
}

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
impl FromWorld for TileHandleSquare {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("bw-tile-square.png"))
    }
}
impl FromWorld for TileHandleIso {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("bw-tile-iso.png"))
    }
}

fn color(s: &str) -> Color {
    Srgba::hex(s).expect("hex color").into()
}

// Generates the initial tilemap with random colors from a hardcoded palette.
fn spawn_tilemap(mut commands: Commands, tile_handle_square: Res<TileHandleSquare>) {
    commands.spawn(Camera2d);

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

    let mut rng = rng();
    let colors: Vec<Color> = vec![
        color("FFBE0B"),
        color("FB5607"),
        color("FF006E"),
        color("8338EC"),
        color("3A86FF"),
    ];
    for tile_id in tile_storage.iter().flatten() {
        commands
            .entity(*tile_id)
            .insert(TileColor(*colors.choose(&mut rng).unwrap()));
    }

    let tile_size = TILE_SIZE_SQUARE;
    let grid_size = GRID_SIZE_SQUARE;
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle_square.clone()),
        tile_size,
        map_type,
        anchor: TilemapAnchor::TopLeft,
        ..Default::default()
    });
}

#[derive(Component)]
struct TileLabel(Entity);

// Generates tile position labels of the form: `(tile_pos.x, tile_pos.y)`
fn spawn_tile_labels(
    mut commands: Commands,
    tilemap_q: Query<(
        &Transform,
        &TilemapType,
        &TilemapGridSize,
        &TilemapTileSize,
        &TileStorage,
        &TilemapSize,
        &TilemapAnchor,
    )>,
    tile_q: Query<&mut TilePos>,
) {
    for (map_transform, map_type, grid_size, tile_size, tilemap_storage, map_size, anchor) in
        tilemap_q.iter()
    {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos
                .center_in_world(map_size, grid_size, tile_size, map_type, anchor)
                .extend(1.0);
            let transform = *map_transform * Transform::from_translation(tile_center);

            let label_entity = commands
                .spawn((
                    Text2d::new(format!("{},{}", tile_pos.x, tile_pos.y)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(Justify::Center),
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
pub struct MapLabel;

// Generates the labels.
fn spawn_map_label(
    mut commands: Commands,
    windows: Query<&Window>,
    map_type_q: Query<(&TilemapType, &TilemapAnchor)>,
) {
    for window in windows.iter() {
        for (map_type, anchor) in map_type_q.iter() {
            // Place the map type label somewhere in the top left side of the screen
            let transform = Transform {
                translation: Vec2::new(-0.9 * window.width() / 2.0, 0.9 * window.height() / 2.0)
                    .extend(1.0),
                ..Default::default()
            };
            let font_size = 20.0;
            commands
                .spawn((
                    Text2d::new("Anchor (SPACE): "),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Left),
                    Anchor::TOP_LEFT,
                    transform,
                    MapLabel,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextSpan::new(format!("{:?}", anchor)),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        TextSpan::new("\nType (ENTER): "),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        TextSpan::new(format!("{:?}", map_type)),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ));

                    parent.spawn((
                        TextSpan::new("\nGrid Scale (TAB): "),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        TextSpan::new("1.0"),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ));
                });
        }
    }
}

/// Changes the anchor when user presses SPACE.
/// Changes the map type when user presses ENTER.
/// Changes the grid scale when user presses TAB.
#[allow(clippy::too_many_arguments)]
fn swap_map_type(
    mut tilemap_query: Query<(
        &mut Transform,
        &TilemapSize,
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
        &mut TilemapAnchor,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    tile_label_q: Query<
        (&TileLabel, &TilePos),
        (With<TileLabel>, Without<MapLabel>, Without<TilemapType>),
    >,
    map_type_label_q: Query<Entity, With<MapLabel>>,
    mut transform_q: Query<&mut Transform, Without<TilemapType>>,
    tile_handle_hex_row: Res<TileHandleHexRow>,
    tile_handle_hex_col: Res<TileHandleHexCol>,
    tile_handle_square: Res<TileHandleSquare>,
    tile_handle_iso: Res<TileHandleIso>,
    mut grid_scale: ResMut<GridScale>,
    mut writer: TextUiWriter,
) {
    if !keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::Tab]) {
        return;
    }
    for (
        map_transform,
        map_size,
        mut map_type,
        mut grid_size,
        mut map_texture,
        mut tile_size,
        mut anchor,
    ) in tilemap_query.iter_mut()
    {
        if keyboard_input.just_pressed(KeyCode::Space) {
            *anchor = rotate_right(&anchor);
        }
        if keyboard_input.just_pressed(KeyCode::Enter) {
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
                    *grid_size = grid_scale.apply(GRID_SIZE_HEX_COL);
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
                    *grid_size = grid_scale.apply(GRID_SIZE_SQUARE);
                }
                TilemapType::Square => {
                    *map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);
                    *map_texture = TilemapTexture::Single((*tile_handle_iso).clone());
                    *tile_size = TILE_SIZE_ISO;
                    *grid_size = grid_scale.apply(GRID_SIZE_ISO);
                }
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    *map_type = TilemapType::Isometric(IsoCoordSystem::Staggered);
                }
                TilemapType::Isometric(IsoCoordSystem::Staggered) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Row);
                    *map_texture = TilemapTexture::Single((*tile_handle_hex_row).clone());
                    *tile_size = TILE_SIZE_HEX_ROW;
                    *grid_size = grid_scale.apply(GRID_SIZE_HEX_ROW);
                }
            }
        }
        if keyboard_input.just_pressed(KeyCode::Tab) {
            grid_scale.next();
            match map_type.as_ref() {
                TilemapType::Hexagon(
                    HexCoordSystem::Row | HexCoordSystem::RowEven | HexCoordSystem::RowOdd,
                ) => {
                    *grid_size = grid_scale.apply(GRID_SIZE_HEX_ROW);
                }
                TilemapType::Hexagon(
                    HexCoordSystem::Column | HexCoordSystem::ColumnEven | HexCoordSystem::ColumnOdd,
                ) => {
                    *grid_size = grid_scale.apply(GRID_SIZE_HEX_COL);
                }
                TilemapType::Square => {
                    *grid_size = grid_scale.apply(GRID_SIZE_SQUARE);
                }
                TilemapType::Isometric(_) => {
                    *grid_size = grid_scale.apply(GRID_SIZE_ISO);
                }
            }
        }
        for (label, tile_pos) in tile_label_q.iter() {
            if let Ok(mut tile_label_transform) = transform_q.get_mut(label.0) {
                let tile_center = tile_pos
                    .center_in_world(map_size, &grid_size, &tile_size, &map_type, &anchor)
                    .extend(1.0);
                *tile_label_transform = *map_transform * Transform::from_translation(tile_center);
            }
        }
        for label_text in &map_type_label_q {
            *writer.text(label_text, 1) = format!("{:?}", anchor.as_ref());
            *writer.text(label_text, 3) = format!("{:?}", map_type.as_ref());
            *writer.text(label_text, 5) = format!("{:.2}", grid_scale.get());
        }
    }
}

fn origin_axes(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct SpawnTilemapSet;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Anchor - Press SPACE to change anchor"),
                        resolution: WindowResolution::new(450, 450),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .init_resource::<TileHandleHexCol>()
        .init_resource::<TileHandleHexRow>()
        .init_resource::<TileHandleSquare>()
        .init_resource::<TileHandleIso>()
        .insert_resource(GridScale::new(vec![1.0, 0.75, 1.5]))
        .add_systems(
            Startup,
            (spawn_tilemap, ApplyDeferred)
                .chain()
                .in_set(SpawnTilemapSet),
        )
        .add_systems(
            Startup,
            (spawn_tile_labels, spawn_map_label).after(SpawnTilemapSet),
        )
        .add_systems(First, camera_movement)
        .add_systems(Update, swap_map_type)
        .add_systems(Update, origin_axes)
        .run();
}
