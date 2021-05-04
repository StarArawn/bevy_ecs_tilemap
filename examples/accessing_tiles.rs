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

    let mut map = Map::new(UVec2::new(4, 4), UVec2::new(32, 32), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0), 0);
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity, true);

    assert!(map.get_tile(IVec2::new(0, 0)).is_some());
    assert!(map.get_tile_neighbors(UVec2::new(2, 2)).len() == 8);

    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            let neighbors = map.get_tile_neighbors(
                UVec2::new(x, y)
            );
            for (pos, _neighbor) in neighbors.iter() {
                let _ = map.add_tile(&mut commands, UVec2::new(pos.x as u32, pos.y as u32), Tile {
                    texture_index: color,
                    ..Default::default()
                }, true);
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
            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    let neighbors = map.get_tile_neighbors(
                        UVec2::new(x, y)
                    );
                    for (pos, _neighbor) in neighbors.iter() {
                        let _ = map.add_tile(&mut commands, UVec2::new(pos.x as u32, pos.y as u32), Tile {
                            texture_index: color,
                            ..Default::default()
                        }, true);
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
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(update_map.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
