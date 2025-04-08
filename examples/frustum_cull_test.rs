//! Displays a map large enough for frustum culling to take place and configures
//! `LogPlugin` to display related traces.

use bevy::{input::common_conditions::input_just_pressed, log::LogPlugin, prelude::*};
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};

mod helpers;

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

#[derive(Resource)]
pub struct TextureHandles {
    hex_row: Handle<Image>,
    hex_col: Handle<Image>,
    square: Handle<Image>,
    iso: Handle<Image>,
}
impl FromWorld for TextureHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            hex_row: asset_server.load("bw-tile-hex-row.png"),
            hex_col: asset_server.load("bw-tile-hex-col.png"),
            square: asset_server.load("bw-tile-square.png"),
            iso: asset_server.load("bw-tile-iso.png"),
        }
    }
}

#[derive(Component)]
pub struct MapTypeLabel;

fn spawn_scene(mut commands: Commands, texture_handles: Res<TextureHandles>) {
    commands.spawn(Camera2d);

    let map_size = TilemapSize {
        // Render chunks are of size 64, so let's create two render chunks
        x: MAP_SIDE_LENGTH_X * 2,
        y: MAP_SIDE_LENGTH_Y,
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: GRID_SIZE_SQUARE,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handles.square.clone()),
        tile_size: TILE_SIZE_SQUARE,
        map_type: TilemapType::Square,
        // This is the default value, but we provide it explicitly for demonstration
        // purposes.
        frustum_culling: FrustumCulling(true),
        ..default()
    });
}

fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
        ))
        .with_children(|parent| {
            parent
                .spawn(Text::new(concat!(
                    "wasd:  move camera\n",
                    "zx:    zoom\n",
                    "space: change map type\n\n",
                    "map type: "
                )))
                .with_children(|parent| {
                    parent.spawn((TextSpan::default(), MapTypeLabel));
                });
        });
}

// Switches the map type when the user presses space.
fn switch_map_type(
    mut tilemap_query: Query<(
        &mut TilemapType,
        &mut TilemapGridSize,
        &mut TilemapTexture,
        &mut TilemapTileSize,
    )>,
    texture_handles: Res<TextureHandles>,
) {
    let Ok((mut map_type, mut grid_size, mut map_texture, mut tile_size)) =
        tilemap_query.single_mut()
    else {
        return;
    };

    let next_type = match *map_type {
        TilemapType::Square => TilemapType::Isometric(IsoCoordSystem::Diamond),
        TilemapType::Isometric(IsoCoordSystem::Diamond) => {
            TilemapType::Isometric(IsoCoordSystem::Staggered)
        }
        TilemapType::Isometric(IsoCoordSystem::Staggered) => {
            TilemapType::Hexagon(HexCoordSystem::Row)
        }
        TilemapType::Hexagon(HexCoordSystem::Row) => TilemapType::Hexagon(HexCoordSystem::RowEven),
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            TilemapType::Hexagon(HexCoordSystem::RowOdd)
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            TilemapType::Hexagon(HexCoordSystem::Column)
        }
        TilemapType::Hexagon(HexCoordSystem::Column) => {
            TilemapType::Hexagon(HexCoordSystem::ColumnEven)
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => TilemapType::Square,
    };

    *map_type = next_type;

    match next_type {
        TilemapType::Square => {
            *map_texture = TilemapTexture::Single(texture_handles.square.clone());
            *tile_size = TILE_SIZE_SQUARE;
            *grid_size = GRID_SIZE_SQUARE;
        }
        TilemapType::Isometric(IsoCoordSystem::Diamond)
        | TilemapType::Isometric(IsoCoordSystem::Staggered) => {
            *map_texture = TilemapTexture::Single(texture_handles.iso.clone());
            *tile_size = TILE_SIZE_ISO;
            *grid_size = GRID_SIZE_ISO;
        }
        TilemapType::Hexagon(HexCoordSystem::Row)
        | TilemapType::Hexagon(HexCoordSystem::RowEven)
        | TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            *map_texture = TilemapTexture::Single(texture_handles.hex_row.clone());
            *tile_size = TILE_SIZE_HEX_ROW;
            *grid_size = GRID_SIZE_HEX_ROW;
        }
        TilemapType::Hexagon(HexCoordSystem::Column)
        | TilemapType::Hexagon(HexCoordSystem::ColumnEven)
        | TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            *map_texture = TilemapTexture::Single(texture_handles.hex_col.clone());
            *tile_size = TILE_SIZE_HEX_COL;
            *grid_size = GRID_SIZE_HEX_COL;
        }
    }
}

fn update_map_type_label(
    type_query: Query<&TilemapType, Changed<TilemapType>>,
    mut label_query: Query<&mut TextSpan, With<MapTypeLabel>>,
) {
    let Ok(map_type) = type_query.single() else {
        return;
    };

    let Ok(mut label) = label_query.single_mut() else {
        return;
    };
    label.0 = format!("{:?}", map_type);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Frustum cull test"),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    // For debugging / demonstrating frustum culling, we'll want to see trace-level
                    // logs for `bevy_ecs_tilemap`.
                    filter: "info,bevy_ecs_tilemap=trace".into(),
                    ..default()
                }),
        )
        .add_plugins(TilemapPlugin)
        .init_resource::<TextureHandles>()
        .add_systems(Startup, (spawn_scene, spawn_ui))
        .add_systems(
            Update,
            (
                helpers::camera::movement,
                switch_map_type.run_if(input_just_pressed(KeyCode::Space)),
                update_map_type_label,
            )
                .chain(),
        )
        .run();
}
