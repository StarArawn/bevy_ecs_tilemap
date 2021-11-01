use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};
mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("iso_color.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut map_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(32, 32),
        TileSize(64.0, 32.0),
        TextureSize(384.0, 32.0),
    );
    map_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    // Layer 0
    let (mut layer_0, layer_0_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings.clone(), 0u16, 0u16, None);
    map.add_layer(&mut commands, 0u16, layer_0_entity);

    layer_0.fill(
        TilePos(0, 0),
        TilePos(32, 32),
        Tile {
            texture_index: 0,
            ..Default::default()
        }
        .into(),
    );
    layer_0.fill(
        TilePos(32, 0),
        TilePos(64, 32),
        Tile {
            texture_index: 1,
            ..Default::default()
        }
        .into(),
    );
    layer_0.fill(
        TilePos(0, 32),
        TilePos(32, 64),
        Tile {
            texture_index: 2,
            ..Default::default()
        }
        .into(),
    );
    layer_0.fill(
        TilePos(32, 32),
        TilePos(64, 64),
        Tile {
            texture_index: 3,
            ..Default::default()
        }
        .into(),
    );

    map_query.build_layer(&mut commands, layer_0, material_handle.clone());

    // Make 2 layers on "top" of the base map.
    for z in 0..5 {
        let mut new_settings = map_settings.clone();
        new_settings.layer_id = z + 1;
        let (mut layer_builder, layer_entity) = LayerBuilder::new(
            &mut commands,
            new_settings.clone(),
            0u16,
            new_settings.layer_id,
            None,
        );
        map.add_layer(&mut commands, new_settings.layer_id, layer_entity);

        let mut random = thread_rng();

        for _ in 0..1000 {
            let position = TilePos(random.gen_range(0..128), random.gen_range(0..128));
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                TileBundle {
                    tile: Tile {
                        texture_index: 0 + z + 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }

        map_query.build_layer(&mut commands, layer_builder, material_handle.clone());
    }

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 1024.0, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Iso diamond Map"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
