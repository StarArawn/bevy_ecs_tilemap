use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = Tilemap2dSize { x: 512, y: 512 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    let mut i = 0;
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    visible: TileVisible(i % 2 == 0 || i % 7 == 0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
            i += 1;
        }
    }

    let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..Default::default()
        })
        .insert(LastUpdate(0.0));
}

#[derive(Component)]
pub struct LastUpdate(f64);

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut tile_storage_query: Query<(&Tile2dStorage, &mut LastUpdate)>,
    tile_query: Query<(Entity, &TilePos2d, &TileVisible)>,
) {
    let current_time = time.seconds_since_startup();
    let (tile_storage, mut last_update) = tile_storage_query.single_mut();
    if current_time - last_update.0 > 0.1 {
        for (entity, position, visibility) in tile_query.iter() {
            let neighbor_count = tile_storage
                .get_tile_neighbors(position)
                .iter()
                .filter(|neighboring_result| {
                    if let Some(neighboring_result) = neighboring_result {
                        let tile_component = tile_query
                            .get_component::<TileVisible>(*neighboring_result)
                            .unwrap();
                        tile_component.0
                    } else {
                        false
                    }
                })
                .count();

            let was_alive = visibility.0;

            let is_alive = match (was_alive, neighbor_count) {
                (true, x) if x < 2 => false,
                (true, 2) | (true, 3) => true,
                (true, x) if x > 3 => false,
                (false, 3) => true,
                (otherwise, _) => otherwise,
            };

            if is_alive && !was_alive {
                commands.entity(entity).insert(TileVisible(true));
            } else if !is_alive && was_alive {
                commands.entity(entity).insert(TileVisible(false));
            }
        }
        last_update.0 = current_time;
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Game of Life Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .add_system(update)
        .run();
}
