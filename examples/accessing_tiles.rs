use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let mut map = Map::new(Vec2::new(2.0, 2.0).into(), Vec2::new(9.0, 9.0).into(), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0));
    let map_entity = commands.spawn()
        .insert(Transform::default())
        .id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity);

    assert!(map.get_tile(Vec2::new(0.0, 0.0).into()).is_some());

    assert!(map.get_tile(Vec2::new(-1.0, 0.0).into()).is_none());

    assert!(map.get_tile(Vec2::new(15.0, 15.0).into()).is_some());

    assert!(map.get_tile(Vec2::new(20.0, 20.0).into()).is_none());

    assert!(map.get_tile_neighbors(Vec2::new(2.0, 2.0).into()).len() == 8);

    commands.entity(map_entity).insert(map);
}


// Should run after the commands from startup have been processed.
fn build_map(
    map_query: Query<&Map, Added<Map>>,
    mut tile_query: Query<&mut Tile>,
) {
    for map in map_query.iter() {
        let mut color = 0;
        for x in (2..20).step_by(4) {
            color += 1;
            for y in (2..20).step_by(4) {
                let neighbors = map.get_tile_neighbors(Vec2::new(x as f32, y as f32).into());
            
                for neighbor in neighbors.iter() {
                    if neighbor.is_some() {
                        let neighbor = neighbor.unwrap();
                        if let Ok(mut tile) = tile_query.get_mut(*neighbor) {
                            tile.texture_index = color;
                        }
                    }
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
            title: String::from("Accessing tiles"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(build_map.system())
        .run();
}
