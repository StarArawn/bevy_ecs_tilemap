use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Component)]
struct CurrentColor(u16);

#[derive(Component)]
struct LastUpdate(f64);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    // Size of the tile map in tiles.
    let tilemap_size = Tilemap2dSize { x: 128, y: 128 };

    // To create a map we use the Tile2dStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);

    // Create a tilemap entity a little early
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn().id();

    // Spawn a 32 by 32 tilemap.
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }

    // We can grab a list of neighbors.
    let neighbors = tile_storage.get_tile_neighbors(&TilePos2d { x: 0, y: 0 });

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos2d { x: 0, y: 0 }).is_some());
    assert!(neighbors.len() == 8);
    let neighbor_count = neighbors.iter().filter(|n| n.is_some()).count();
    assert!(neighbor_count == 3); // Only 3 neighbors since negative is outside of map.

    // This changes some of our tiles by looking at neighbors.
    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.
            let neighbors = tile_storage.get_neighboring_pos(&TilePos2d { x, y });
            for pos in neighbors.iter().filter_map(|pos| pos.as_ref()) {
                // We can replace the tile texture component like so:
                commands
                    .entity(tile_storage.get(pos).unwrap())
                    .insert(TileTexture(color));
            }
        }
    }

    // This is the size of each individual tiles in pixels.
    let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };

    // Spawns a tilemap.
    // Once the tile storage is inserted onto the tilemap entity it can no longer be accessed.
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..Default::default()
        })
        .insert(LastUpdate(0.0))
        .insert(CurrentColor(1));
}

// A system that manipulates tile colors.
fn update_map(
    time: Res<Time>,
    mut tilemap_query: Query<(&mut CurrentColor, &mut LastUpdate, &Tile2dStorage)>,
    mut tile_query: Query<&mut TileTexture>,
) {
    let current_time = time.seconds_since_startup();
    for (mut current_color, mut last_update, tile_storage) in tilemap_query.iter_mut() {
        if current_time - last_update.0 > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }

            let mut color = current_color.0;

            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    // Grab the neighboring tiles
                    let neighboring_entities = tile_storage.get_tile_neighbors(&TilePos2d { x, y });

                    // Iterate over neighbors
                    for i in 0..8 {
                        if let Some(neighbor) = neighboring_entities[i] {
                            // Query the tile entities to change the colors
                            if let Ok(mut tile_texture) = tile_query.get_mut(neighbor) {
                                tile_texture.0 = color as u32;
                            }
                        }
                    }
                }
                color += 1;
                if color > 5 {
                    color = 1;
                }
            }
            last_update.0 = current_time;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Basic Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .add_system(update_map)
        .run();
}
