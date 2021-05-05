use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, thread_rng};

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_xyz(2560.0, 2560.0, 1000.0 - 0.1),
        ..OrthographicCameraBundle::new_2d()
    });

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let mut map = Map::new(MapSettings::new(UVec2::new(5, 5), UVec2::new(128, 128), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0), 0));
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity, true);
    
    for entity in map.get_all_tiles().iter() {
        if let Some(entity) = entity {
            commands.entity(*entity).insert(LastUpdate::default());
        }
    }

    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    });
}

#[derive(Default)]
struct LastUpdate {
    value: f64,
}

// Worst case lookup
fn random(
    mut commands: Commands,
    time: ResMut<Time>,
    mut query: Query<(&UVec2, &mut Tile, &mut LastUpdate)>,
    map_query: Query<&Map>,
) {
    let current_time = time.seconds_since_startup();
    let mut random = thread_rng();
    let mut did_update = false;
    for (_, mut tile, mut last_update) in query.iter_mut() {
        // if (current_time - last_update.value) > 0.1 {
            tile.texture_index = random.gen_range(0..6);
            last_update.value = current_time;
            did_update = true;
        // }
    }

    // Smarter way to update..
    if did_update {
        if let Ok(map) = map_query.single() {
            for x in 0..map.settings.map_size.x {
                for y in 0..map.settings.map_size.y {
                    // Update first tile in each chunk at least until we get an notify_chunk
                    map.notify(&mut commands, UVec2::new(x * map.settings.chunk_size.x, y * map.settings.chunk_size.y));
                }
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Info)
    .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Random Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(random.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
