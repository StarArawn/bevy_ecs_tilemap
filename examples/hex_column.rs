use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("flat_hex_tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));
    let map_settings = MapSettings {
        map_size: Vec2::new(1.0, 1.0).into(),
        chunk_size: Vec2::new(64.0, 64.0).into(),
        tile_size: Vec2::new(17.0, 15.0),
        texture_size: Vec2::new(102.0, 15.0),
    };
    let mut map = Map::new(map_settings, 0);
    // New mesher needs to be applied before chunks are built with map.
    map.mesher = Box::new(HexChunkMesher::new(HexType::ColumnEven));
    let map_entity = commands.spawn().id();
    map.build(
        &mut commands,
        &mut meshes,
        material_handle.clone(),
        map_entity,
        true,
    );
    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    });

    for z in 0..2 {
        let mut map = Map::new(map_settings, z + 1);
        // New mesher needs to be applied before chunks are built with map.
        map.mesher = Box::new(HexChunkMesher::new(HexType::ColumnEven));
        let map_entity = commands.spawn().id();
        map.build(
            &mut commands,
            &mut meshes,
            material_handle.clone(),
            map_entity,
            false,
        );

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = Vec2::new(random.gen_range(0.0..64.0), random.gen_range(0.0..64.0));
            // Ignore errors for demo sake.
            let _ = map.add_tile(
                &mut commands,
                position.into(),
                Tile {
                    texture_index: z + 1,
                    ..Default::default()
                },
            );
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
            title: String::from("Hex Map Column Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TileMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .run();
}
