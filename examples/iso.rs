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
    // Create the camera setup
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Load textures
    let texture_handle = asset_server.load("iso.png");
    
    // Apply textures to materials
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Set a ratio for texture size. Higher means chunkier textures


    let mut map_settings = MapSettings::new(
        UVec2::new(4, 4), // Map Size
        UVec2::new(32, 32), // Chunk Size
        Vec2::new(64.0, 64.0), //Tile Size
        Vec2::new(640.0, 1024.0), // Texture Size
        0, // Layer ID
    );

    // We want an isometric map, so We'll set the mesh_type to the relevant type
    map_settings.mesh_type = TilemapMeshType::Isometric;

    // Get a map for Layer 0
    let mut map = Map::new(map_settings.clone());
    
    // Get an entity ID to be able to refer to the map later on
    let map_entity_id = commands.spawn().id();
    
    // Iterate through all tiles and populate all of these with the same texture
    // The fifth argument is a function which is iterated though. 
    map.build_iter(
        &mut commands,
        &mut meshes,
        material_handle.clone(),
        map_entity_id,
        |_| Tile {
            texture_index: 10,
            ..Default::default()
        },
    );

    // Save the map and add the Bevy bundle
    // containing Map, Transform and GlobalTransform components
    commands.entity(map_entity_id).insert_bundle(MapBundle {
        map,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Make 2 layers on "top" of the base map.
    for z in 0..5 {
        let mut new_settings = map_settings.clone();
        new_settings.layer_id = z + 1;
        let mut map = Map::new(new_settings);
        let map_entity = commands.spawn().id();
        map.build(
            &mut commands,
            &mut meshes,
            material_handle.clone(),
            map_entity,
            false,
        );

        let mut random = thread_rng();

        for _ in 0..1000 {
            let position = UVec2::new(random.gen_range(0..128), random.gen_range(0..128));
            // Ignore errors for demo sake.
            let _ = map.add_tile(
                &mut commands,
                position,
                Tile {
                    texture_index: 70 + z + 1,
                    ..Default::default()
                },
                true,
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
            title: String::from("Iso Map"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
