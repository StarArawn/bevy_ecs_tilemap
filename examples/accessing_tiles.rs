use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

struct CurrentColor(u32);
struct LastUpdate(f64);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_xyz(1024.0, 1024.0, 1000.0 - 0.1),
        ..OrthographicCameraBundle::new_2d()
    });

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let mut map = Map::new(Vec2::new(4.0, 4.0).into(), Vec2::new(32.0, 32.0).into(), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0), 0);
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity, true);

    assert!(map.get_tile(Vec2::new(0.0, 0.0).into()).is_some());
    assert!(map.get_tile_neighbors(Vec2::new(2.0, 2.0).into()).len() == 8);

    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            let neighbors = map.get_tile_neighbors(
                Vec2::new(x as f32, y as f32).into()
            );
            for (pos, _neighbor) in neighbors.iter() {
                let _ = map.add_tile(&mut commands, *pos, Tile {
                    texture_index: color,
                    ..Default::default()
                });
            }
        }
    }

    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    })
    .insert(CurrentColor(1))
    .insert(LastUpdate(0.0));
}


// Should run after the commands from startup have been processed.
// An example of a slow way of editing tiles..
fn update_map(
    mut commands: Commands,
    time: Res<Time>,
    mut map_query: Query<(&mut Map, &mut CurrentColor, &mut LastUpdate)>,
) {
    let current_time = time.seconds_since_startup();
    for (mut map, mut current_color, mut last_update) in map_query.iter_mut() {
        if (current_time - last_update.0) > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }
            let mut color = current_color.0;
            dbg!(color);
            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    let neighbors = map.get_tile_neighbors(
                        Vec2::new(x as f32, y as f32).into()
                    );
                    for (pos, _neighbor) in neighbors.iter() {
                        let _ = map.add_tile(&mut commands, *pos, Tile {
                            texture_index: color,
                            ..Default::default()
                        });
                    }
                }
                color +=1;
                if color > 5 {
                    color = 1;
                }
            }

            last_update.0 = current_time;
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
            title: String::from("Accessing tiles"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(update_map.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
