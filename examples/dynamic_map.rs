use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

#[derive(Default, Component)]
struct LastUpdate {
    value: f64,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("tiles.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(8, 8),
            TileSize(16.0, 16.0),
            TextureSize(96.0, 16.0),
        ),
        0u16,
        0u16,
    );
    layer_builder.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands.entity(layer_entity).insert(LastUpdate::default());

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}

fn build_map(map_query: &mut MapQuery, commands: &mut Commands) {
    let mut random = thread_rng();

    for _ in 0..100 {
        let position = TilePos(random.gen_range(0..16), random.gen_range(0..16));
        // Ignore errors for demo sake.
        let _ = map_query.set_tile(
            commands,
            position,
            Tile {
                texture_index: 0,
                ..Default::default()
            },
            0u16,
            0u16,
        );
        map_query.notify_chunk_for_tile(position, 0u16, 0u16);
    }
}

fn update_map(
    time: ResMut<Time>,
    mut commands: Commands,
    mut query: Query<&mut LastUpdate>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut last_update in query.iter_mut() {
        if (current_time - last_update.value) > 1.0 {
            map_query.despawn_layer_tiles(&mut commands, 0u16, 0u16);
            build_map(&mut map_query, &mut commands);
            last_update.value = current_time;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Dynamic Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .add_system(update_map)
        .run();
}
