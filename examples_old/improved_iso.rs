use bevy::{math::Vec2, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};
mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.orthographic_projection.scale = 0.5;
    commands.spawn_bundle(camera_bundle);

    let texture_handle = asset_server.load("isometric-sheet.png");

    // Create map entity and component
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Important for ISO rendering! In order to order tiles correctly we need a chunk per Y layer.
    // This is why the map size is 4 on the y but only 1 for chunk size.
    let mut map_settings = LayerSettings::new(
        MapSize(1, 8),
        ChunkSize(8, 1),
        TileSize(64.0, 64.0),
        TextureSize(384.0, 64.0),
    );
    map_settings.grid_size = Vec2::new(64.0, 64.0 / 2.0);
    map_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond3d);

    // Layer 0
    let (mut layer_0, layer_0_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings.clone(), 0u16, 0u16);
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

    map_query.build_layer(&mut commands, layer_0, texture_handle.clone());

    // Make 2 layers on "top" of the base map.
    for z in 0..1 {
        let (mut layer_builder, layer_entity) =
            LayerBuilder::new(&mut commands, map_settings.clone(), 0u16, z + 1);
        map.add_layer(&mut commands, z + 1, layer_entity);

        let mut random = thread_rng();

        for _ in 0..8 {
            let position = TilePos(random.gen_range(0..3), random.gen_range(0..3));
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

        map_query.build_layer(&mut commands, layer_builder, texture_handle.clone());
    }

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 128.0, 0.0))
        .insert(GlobalTransform::default());

    let x_pos = 0.0;
    let y_pos = 0.0;
    // TODO: Replace this with like a "get_z_map_position" or something.
    let center = project_iso(Vec2::new(x_pos, y_pos), 64.0, 32.0);
    let sprite_pos = Transform::from_xyz(center.x, center.y, 1.0 + (1.0 - (center.y / 10000.0)));
    let texture_handle = asset_server.load("player.png");

    commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            transform: sprite_pos,
            ..Default::default()
        })
        .insert(helpers::movement::Player);
}

fn project_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    return Vec2::new(x, -y);
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Iso diamond Map"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::movement::update)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
