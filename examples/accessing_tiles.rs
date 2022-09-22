use bevy::{prelude::*, render::texture::ImageSettings};
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
    let tilemap_size = TilemapSize { x: 128, y: 128 };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = TileStorage::empty(tilemap_size);

    // For the purposes of this example, we consider a square tile map,
    // where diagonals are also considered to be neighbors.
    let tilemap_type = TilemapType::Square {
        diagonal_neighbors: true,
    };

    // Create a tilemap entity a little early
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn().id();

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };
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
    let neighbors = get_tile_neighbors(&TilePos { x: 0, y: 0 }, &tile_storage, &tilemap_type);

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos { x: 0, y: 0 }).is_some());
    assert_eq!(neighbors.count(), 3); // Only 3 neighbors since negative is outside of map.

    // This changes some of our tiles by looking at neighbors.
    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.
            let neighbors = get_neighboring_pos(&TilePos { x, y }, &tilemap_size, &tilemap_type);
            for pos in neighbors.into_iter() {
                // We can replace the tile texture component like so:
                commands
                    .entity(tile_storage.get(&pos).unwrap())
                    .insert(TileTexture(color));
            }
        }
    }

    // This is the size of each individual tiles in pixels.
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    // Spawns a tilemap.
    // Once the tile storage is inserted onto the tilemap entity it can no longer be accessed.
    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            map_type: tilemap_type,
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&tilemap_size, &tile_size, 0.0),
            ..Default::default()
        })
        .insert(LastUpdate(0.0))
        .insert(CurrentColor(1));
}

// A system that manipulates tile colors.
fn update_map(
    time: Res<Time>,
    mut tilemap_query: Query<(
        &mut CurrentColor,
        &mut LastUpdate,
        &TileStorage,
        &TilemapType,
    )>,
    mut tile_query: Query<&mut TileTexture>,
) {
    let current_time = time.seconds_since_startup();
    for (mut current_color, mut last_update, tile_storage, tilemap_type) in tilemap_query.iter_mut()
    {
        if current_time - last_update.0 > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }

            let mut color = current_color.0;

            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    // Grab the neighboring tiles
                    let neighboring_entities =
                        get_tile_neighbors(&TilePos { x, y }, tile_storage, tilemap_type);

                    // Iterate over neighbors
                    for neighbor_entity in neighboring_entities.into_iter() {
                        // Query the tile entities to change the colors
                        if let Ok(mut tile_texture) = tile_query.get_mut(neighbor_entity) {
                            tile_texture.0 = color as u32;
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
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(update_map)
        .run();
}
