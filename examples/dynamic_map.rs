use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, thread_rng};

mod helpers;

#[derive(Default)]
struct LastUpdate {
    value: f64,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let mut map = Map::new(Vec2::new(2.0, 2.0).into(), Vec2::new(8.0, 8.0).into(), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0), 0);
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity, false);

    build_map(&mut map, &mut commands);

    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    }).insert(LastUpdate::default());
}

fn build_map(map: &mut Map, commands: &mut Commands) {
    let mut random = thread_rng();

    for _ in 0..100 {
        let position = Vec2::new(
            random.gen_range(0.0..16.0),
            random.gen_range(0.0..16.0),
        );
        // Ignore errors for demo sake.
        let _ = map.add_tile(commands, position.into(), Tile {
            texture_index: 0,
            ..Default::default()
        });
    }
}

fn remove_map(map: &Map, commands: &mut Commands) {
    for (tile_pos, _) in map.get_all_tiles() {
        map.remove_tile(commands, *tile_pos);
    }
}

fn update_map(
    time: ResMut<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Map, &mut LastUpdate)>
) {
    let current_time = time.seconds_since_startup();
    for (mut map, mut last_update) in query.iter_mut() {
        if (current_time - last_update.value) > 1.0 {
            remove_map(&map, &mut commands);
            build_map(&mut map, &mut commands);
            last_update.value = current_time;
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
            title: String::from("Dynamic Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .add_system(update_map.system())
        .run();
}
