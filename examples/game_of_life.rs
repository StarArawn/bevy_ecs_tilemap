use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = UVec2::new(10 * 16, 10 * 16);

    let mut map = Map::new(UVec2::new(10, 10), UVec2::new(16, 16), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0), 0);
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity, false);
    
    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = UVec2::new(
                x,
                y,
            );
            // Ignore errors for demo sake.
            let _ = map.add_tile(&mut commands, position, Tile {
                texture_index: 0,
                ..Default::default()
            }, i % 2 == 0 || i % 7 == 0);
            i += 1;
        }
    }

    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    }).insert(LastUpdate(0.0));
}

pub struct LastUpdate(f64);

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut map_query: Query<(&Map, &mut LastUpdate)>,
    visible: Query<&bevy_ecs_tilemap::prelude::Visible>,
    tile_query: Query<(Entity, &UVec2), With<Tile>>,
) {
    let current_time = time.seconds_since_startup();
    if let Ok((map, mut last_update)) = map_query.single_mut() {
        if current_time - last_update.0 > 0.0 {
            for (entity, pos) in tile_query.iter() {
                // Get neighbor count.
                let neighbor_count = map.get_tile_neighbors(*pos).iter().filter(|x| {
                    if let Some(entity) = x.1 {
                        return visible.get(entity).is_ok();
                    }
                    return false;
                }).count();
                let was_alive = visible.get(entity).is_ok();

                let is_alive = match (was_alive, neighbor_count) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };

                if is_alive && !was_alive {
                    commands.entity(entity).insert(bevy_ecs_tilemap::prelude::Visible);
                    map.notify(&mut commands, *pos);
                } else if !is_alive && was_alive {
                    commands.entity(entity).remove::<bevy_ecs_tilemap::prelude::Visible>();
                    map.notify(&mut commands, *pos);
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

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Game Of Life"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .add_system(update.system())
        .run();
}
