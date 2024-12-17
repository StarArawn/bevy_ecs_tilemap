//! Example showing tilemap despawning.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new("space: spawn tilemap\nesc: despawn tilemap"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    maps: Query<(), With<TileStorage>>,
) {
    let num_maps = maps.iter().len();

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    // Spawn the elements of the tilemap.
    // Alternatively, you can use helpers::filling::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    color: TileColor(
                        Hsla::hsl(0., 0.9, 0.8)
                            .rotate_hue(num_maps as f32 * 19.)
                            .into(),
                    ),
                    texture_index: TileTextureIndex(5),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let offset = Vec3::splat(num_maps as f32 * tile_size.x / 2.);
    let mut transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
    transform.translation += offset;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform,
        ..default()
    });
}

fn despawn_map(mut commands: Commands, mut maps: Query<(Entity, &mut TileStorage, &Transform)>) {
    let Some((tilemap_entity, mut tile_storage, _)) = maps
        .iter_mut()
        .sort_by::<&Transform>(|a, b| b.translation.z.partial_cmp(&a.translation.z).unwrap())
        .next()
    else {
        return;
    };

    commands.entity(tilemap_entity).despawn_recursive();

    for maybe_entity in tile_storage.iter_mut() {
        if let Some(entity) = maybe_entity.take() {
            commands.entity(entity).despawn();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Despawn Tilemap Example"),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, spawn_map.run_if(input_just_pressed(KeyCode::Space)))
        .add_systems(
            Update,
            despawn_map.run_if(input_just_pressed(KeyCode::Escape)),
        )
        .run();
}
