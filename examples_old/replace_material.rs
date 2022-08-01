use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Default, Component)]
struct DidUpdate {
    value: bool,
}

struct Materials {
    a: Handle<Image>,
    b: Handle<Image>,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle_a = asset_server.load("tiles.png");
    let texture_handle_b = asset_server.load("tiles2.png");

    let materials = Materials {
        a: texture_handle_a,
        b: texture_handle_b,
    };

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let layer_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(8, 8),
        TileSize(16.0, 16.0),
        TextureSize(96.0, 16.0),
    );

    let center = layer_settings.get_pixel_center();

    let (mut layer_builder, layer_entity) =
        LayerBuilder::new(&mut commands, layer_settings, 0u16, 0u16);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_builder, materials.a.clone());

    commands.entity(layer_entity).insert(DidUpdate::default());

    commands.insert_resource(materials);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

fn replace_material(
    time: Res<Time>,
    materials: Res<Materials>,
    mut did_update_query: Query<&mut DidUpdate>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut did_update in did_update_query.iter_mut() {
        if did_update.value {
            return;
        }
        // Replace the material after two seconds.
        if current_time > 2.0 {
            let layer = map_query.get_layer(0u16, 0u16);
            let chunk_entity = if let Some((_, layer)) = layer {
                // Replace the material in the first chunk.
                let chunk = layer.get_chunk(ChunkPos::default());
                if let Some(e) = chunk {
                    // Get the chunk's ColorMaterial component.
                    Some(e)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(chunk_entity) = chunk_entity {
                map_query.update_chunk(chunk_entity, |mut chunk| {
                    chunk.material = materials.b.clone();
                });
            }

            did_update.value = true;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Replace Material Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(replace_material)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
