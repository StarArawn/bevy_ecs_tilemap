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

    let mut map = Map::new(MapSettings::new(
        UVec2::new(4, 4),
        UVec2::new(8, 8),
        Vec2::new(16.0, 16.0),
        Vec2::new(96.0, 256.0),
        0,
    ));
    let map_entity = commands.spawn().id();
    map.build_iter(
        &mut commands,
        &mut meshes,
        material_handle,
        map_entity,
        |_| Tile {
            texture_index: 1,
            ..Default::default()
        },
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
    visibility_query: Query<&bevy_ecs_tilemap::prelude::VisibleTile>,
) {
    let current_time = time.seconds_since_startup();
    for (map, mut last_update) in map_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.1 {
            let mut random = thread_rng();
            let position = IVec2::new(random.gen_range(0..32), random.gen_range(0..32));

            // Instead of removing the tile entity we want to hide the tile by removing the Visible component.
            if let Some(tile_entity) = map.get_tile(position) {
                if visibility_query.get(tile_entity).is_ok() {
                    commands
                        .entity(tile_entity)
                        .remove::<bevy_ecs_tilemap::prelude::VisibleTile>();
                } else {
                    commands
                        .entity(tile_entity)
                        .insert(bevy_ecs_tilemap::prelude::VisibleTile);
                }
            }

            map.notify(
                &mut commands,
                UVec2::new(position.x as u32, position.y as u32),
            );

            last_update.value = current_time;
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Remove Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(remove_tiles.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
