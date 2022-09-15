use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

// Side length of a colored quadrant (in "number of tiles").
const MAP_SIDE_LENGTH: u32 = 4;

fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("hex-tile-row.png");

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

    let tile_size = TilemapTileSize { x: 49.0, y: 57.0 };
    let grid_size = TilemapGridSize { x: 47.0, y: 47.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: total_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            ..Default::default()
        });
}

#[derive(Component, Deref)]
struct LabeledTile(Entity);

fn spawn_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    tile_q: Query<&mut TilePos>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (tilemap_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos.to_world_pos(grid_size, map_type);
            let text_center = Vec2::new(tile_center.x, tile_center.y + 0.125 * grid_size.y);
            let mut transform = *tilemap_transform;
            transform.translation += text_center.extend(2.0);
            commands
                .spawn_bundle(Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..default()
                })
                .insert(LabeledTile(*tile_entity));
        }
    }
}

fn swap_map_type(
    mut tilemap_query: Query<(
        &Transform,
        &mut TilemapType,
        &TilemapGridSize,
        &mut TilemapTexture,
    )>,
    keyboard_input: Res<Input<KeyCode>>,
    mut label_q: Query<(&LabeledTile, &mut Transform), Without<TilemapType>>,
    tile_q: Query<&TilePos>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (map_transform, mut map_type, grid_size, mut tilemap_texture) in
            tilemap_query.iter_mut()
        {
            match *map_type {
                TilemapType::Hexagon(HexCoordSystem::Row) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowEven);
                }
                TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Column);
                    let texture_handle: Handle<Image> = asset_server.load("hex-tile-col.png");
                    *tilemap_texture = TilemapTexture(texture_handle);
                }
                TilemapType::Hexagon(HexCoordSystem::Column) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowEven);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::RowOdd);
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                    *map_type = TilemapType::Hexagon(HexCoordSystem::Row);
                    let texture_handle: Handle<Image> = asset_server.load("hex-tile-row.png");
                    *tilemap_texture = TilemapTexture(texture_handle);
                }
                _ => {}
            }

            for (labeled_tile, mut tile_transform) in label_q.iter_mut() {
                let tile_pos = tile_q.get(**labeled_tile).unwrap();
                let tile_center = tile_pos.to_world_pos(grid_size, &map_type);
                let text_center = Vec2::new(tile_center.x, tile_center.y + 0.125 * grid_size.y);
                *tile_transform = *map_transform;
                tile_transform.translation += text_center.extend(2.0);
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
        .add_startup_system_to_stage(StartupStage::Startup, spawn_tilemap)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_labels)
        .add_system(helpers::camera::movement)
        .add_system(swap_map_type)
        .run();
}
