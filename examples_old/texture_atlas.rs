use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

#[derive(Default, Component)]
struct LastUpdate {
    value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Finished,
}

#[derive(Default)]
struct TextureHandles {
    handles: Vec<HandleUntyped>,
}

fn load_textures(mut texture_handles: ResMut<TextureHandles>, asset_server: Res<AssetServer>) {
    texture_handles.handles = asset_server.load_folder("individual").unwrap();
}

fn check_textures(
    mut state: ResMut<State<AppState>>,
    texture_handles: ResMut<TextureHandles>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(texture_handles.handles.iter().map(|handle| handle.id))
    {
        state.set(AppState::Finished).unwrap();
    }
}

fn startup(
    mut commands: Commands,
    texture_handles: Res<TextureHandles>,
    mut textures: ResMut<Assets<Image>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let tile_size = Vec2::new(16.0, 16.0);

    let mut atlas_builder = TileAtlasBuilder::new(tile_size);
    for handle in texture_handles.handles.iter() {
        let texture = textures.get(handle).unwrap();
        atlas_builder
            .add_texture(handle.clone_weak().typed::<Image>(), texture)
            .unwrap();
    }

    let texture_atlas = atlas_builder.finish(&mut textures).unwrap();
    let texture_atlas_texture = texture_atlas.texture.clone();
    let texture_atlas_size = texture_atlas.size;

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(8, 8),
            TileSize(tile_size.x, tile_size.y),
            TextureSize(texture_atlas_size.x, texture_atlas_size.y),
        ),
        0u16,
        0u16,
    );
    layer_builder.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_builder, texture_atlas_texture);

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
                texture_index: random.gen_range(0..6),
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
            title: String::from("Texture Atlas Example"),
            ..Default::default()
        })
        .init_resource::<TextureHandles>()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_state(AppState::Setup)
        .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(load_textures.system()))
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(check_textures.system()))
        .add_system_set(SystemSet::on_enter(AppState::Finished).with_system(startup.system()))
        .add_system_set(SystemSet::on_update(AppState::Finished).with_system(update_map.system()))
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
