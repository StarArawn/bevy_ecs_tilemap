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

    let mut map = Map::new(Vec2::new(2.0, 2.0).into(), Vec2::new(8.0, 8.0).into(), Vec2::new(16.0, 16.0), Vec2::new(96.0, 256.0));
    let map_entity = commands.spawn()
        .insert(Transform::default())
        .id();
    map.build(&mut commands, &mut meshes, material_handle, map_entity);

    assert!(map.get_tile(Vec2::new(0.0, 0.0).into()).is_some());

    assert!(map.get_tile(Vec2::new(-1.0, 0.0).into()).is_none());

    assert!(map.get_tile(Vec2::new(15.0, 15.0).into()).is_some());

    assert!(map.get_tile(Vec2::new(16.0, 16.0).into()).is_none());

    assert!(map.get_tile_neighbors(Vec2::new(2.0, 2.0).into()).len() == 8);

    commands.entity(map_entity).insert(map);
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
        .run();
}
