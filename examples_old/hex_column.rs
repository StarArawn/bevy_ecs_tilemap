use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("flat_hex_tiles.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut map_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(64, 64),
        TileSize(17.0, 15.0),
        TextureSize(17.0, 105.0),
    );
    map_settings.mesh_type = TilemapMeshType::Hexagon(HexType::Column);

    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings.clone(), 0u16, 0u16);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.fill(
        TilePos(0, 0),
        TilePos(64, 64),
        Tile {
            texture_index: 0,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        TilePos(64, 0),
        TilePos(128, 64),
        Tile {
            texture_index: 1,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        TilePos(0, 64),
        TilePos(64, 128),
        Tile {
            texture_index: 2,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        TilePos(64, 64),
        TilePos(128, 128),
        Tile {
            texture_index: 3,
            ..Default::default()
        }
        .into(),
    );

    map_query.build_layer(&mut commands, layer_builder, texture_handle.clone());

    for z in 0..2 {
        let (mut layer_builder, layer_entity) =
            LayerBuilder::<TileBundle>::new(&mut commands, map_settings, 0u16, z + 1);
        map.add_layer(&mut commands, z + 1, layer_entity);

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = TilePos(random.gen_range(0..128), random.gen_range(0..128));
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                Tile {
                    texture_index: z + 1,
                    ..Default::default()
                }
                .into(),
            );
        }

        map_query.build_layer(&mut commands, layer_builder, texture_handle.clone());
    }

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-48.0, -24.0, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Hex Map Column Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
