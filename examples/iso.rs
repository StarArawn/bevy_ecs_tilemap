use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, thread_rng};

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("iso.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Layer 0
    let mut map = Map::new(Vec2::new(2.0, 2.0).into(), Vec2::new(8.0, 8.0).into(), Vec2::new(64.0, 64.0), Vec2::new(640.0, 1024.0), 0);
    map.mesher = Box::new(IsoChunkMesher);
    let map_entity = commands.spawn().id();
    map.build(&mut commands, &mut meshes, material_handle.clone(), map_entity, true);
    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Make 2 layers on "top" of the base map.
    for z in 0..2 {
        let mut map = Map::new(Vec2::new(2.0, 2.0).into(), Vec2::new(8.0, 8.0).into(), Vec2::new(64.0, 64.0), Vec2::new(640.0, 1024.0), z + 1);
        map.mesher = Box::new(IsoChunkMesher);
        let map_entity = commands.spawn().id();
        map.build(&mut commands, &mut meshes, material_handle.clone(), map_entity, false);

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = Vec2::new(
                random.gen_range(0.0..16.0),
                random.gen_range(0.0..16.0),
            );
            // Ignore errors for demo sake.
            let _ = map.add_tile(&mut commands, position.into(), Tile {
                texture_index: z + 1,
                ..Default::default()
            });
        }
        commands.entity(map_entity).insert_bundle(MapBundle {
            map,
            transform: Transform::from_xyz(0.0, 0.0, z as f32 + 1.0),
            ..Default::default()
        });
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
            title: String::from("Iso Map"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .run();
}
