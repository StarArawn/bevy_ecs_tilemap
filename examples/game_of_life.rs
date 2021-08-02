use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = UVec2::new(5 * 16, 5 * 16);

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        LayerSettings::new(
            UVec2::new(5, 5),
            UVec2::new(16, 16),
            Vec2::new(16.0, 16.0),
            Vec2::new(96.0, 256.0),
        ),
        0u16,
        0u16,
        None,
    );

    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = UVec2::new(x, y);
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                Tile {
                    texture_index: 0,
                    visible: i % 2 == 0 || i % 7 == 0,
                    ..Default::default()
                }
                .into(),
            );
            i += 1;
        }
    }

    map_query.build_layer(&mut commands, layer_builder, material_handle.clone());

    commands.entity(layer_entity).insert(LastUpdate(0.0));

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-640.0, -640.0, 0.0))
        .insert(GlobalTransform::default());
}

pub struct LastUpdate(f64);

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    tile_query: Query<(Entity, &Tile, &UVec2)>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    if let Ok(mut last_update) = last_update_query.single_mut() {
        if current_time - last_update.0 > 0.1 {
            for (entity, tile, pos) in tile_query.iter() {
                // Get neighbor count.
                let neighbor_count = map_query
                    .get_tile_neighbors(*pos, 0u16, 0u16)
                    .iter()
                    .filter(|x| {
                        if let Some(entity) = x.1 {
                            if let Ok((_, tile, _)) = tile_query.get(entity) {
                                return tile.visible;
                            }
                        }
                        return false;
                    })
                    .count();
                let was_alive = tile.visible;

                let is_alive = match (was_alive, neighbor_count) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };

                if is_alive && !was_alive {
                    commands.entity(entity).insert(Tile {
                        visible: true,
                        ..*tile
                    });
                    map_query.notify_chunk_for_tile(*pos, 0u16, 0u16);
                } else if !is_alive && was_alive {
                    commands.entity(entity).insert(Tile {
                        visible: false,
                        ..*tile
                    });
                    map_query.notify_chunk_for_tile(*pos, 0u16, 0u16);
                }
            }

            last_update.0 = current_time;
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Game Of Life"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .add_system(update.system())
        .run();
}
