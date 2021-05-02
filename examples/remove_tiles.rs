use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

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
    let map_settings = MapSettings {
        map_size: Vec2::new(2.0, 2.0).into(),
        chunk_size: Vec2::new(8.0, 8.0).into(),
        tile_size: Vec2::new(16.0, 16.0),
        texture_size: Vec2::new(96.0, 256.0),
    };
    let mut map = Map::new(map_settings, 0);
    let map_entity = commands.spawn().id();
    map.build(
        &mut commands,
        &mut meshes,
        material_handle,
        map_entity,
        true,
    );

    commands
        .entity(map_entity)
        .insert_bundle(MapBundle {
            map,
            ..Default::default()
        })
        .insert(LastUpdate::default());
}

fn remove_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut map_query: Query<(&Map, &mut LastUpdate)>,
) {
    let current_time = time.seconds_since_startup();
    for (map, mut last_update) in map_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.5 {
            let mut random = thread_rng();
            let tile_entity = map.get_tile(
                Vec2::new(random.gen_range(0.0..16.0), random.gen_range(0.0..16.0)).into(),
            );

            // Note you can also call map.remove_tile() instead.
            if tile_entity.is_some() {
                commands.entity(*tile_entity.unwrap()).insert(RemoveTile);
            }

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
            title: String::from("Remove Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(remove_tiles.system())
        .run();
}
