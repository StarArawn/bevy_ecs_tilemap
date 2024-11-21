use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Component)]
struct CurrentColor(u16);

#[derive(Component)]
struct LastUpdate(f64);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    // Size of the tile map in tiles.
    let map_size = TilemapSize { x: 128, y: 128 };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = TileStorage::empty(map_size);

    // For the purposes of this example, we consider a tilemap with rectangular tiles.
    let map_type = TilemapType::Square;

    // Create a tilemap entity a little early
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn_empty().id();

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // We can grab a list of neighbors.
    let neighbor_positions =
        Neighbors::get_square_neighboring_positions(&TilePos { x: 0, y: 0 }, &map_size, true);
    let neighbor_entities = neighbor_positions.entities(&tile_storage);

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos { x: 0, y: 0 }).is_some());
    assert_eq!(neighbor_entities.iter().count(), 3); // Only 3 neighbors since negative is outside of map.

    // This changes some of our tiles by looking at neighbors.
    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.

            let neighbors =
                Neighbors::get_square_neighboring_positions(&TilePos { x, y }, &map_size, true);
            for pos in neighbors.iter() {
                // We can replace the tile texture component like so:
                commands
                    .entity(tile_storage.get(pos).unwrap())
                    .insert(TileTextureIndex(color));
            }
        }
    }

    // This is the size of each individual tiles in pixels.
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();

    // Spawns a tilemap.
    // Once the tile storage is inserted onto the tilemap entity it can no longer be accessed.
    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            map_type,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        LastUpdate(0.0),
        CurrentColor(1),
    ));
}

// A system that manipulates tile colors.
fn update_map(
    time: Res<Time>,
    mut tilemap_query: Query<(
        &mut CurrentColor,
        &mut LastUpdate,
        &TileStorage,
        &TilemapSize,
    )>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    let current_time = time.elapsed_secs_f64();
    for (mut current_color, mut last_update, tile_storage, map_size) in tilemap_query.iter_mut() {
        if current_time - last_update.0 > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }

            let mut color = current_color.0;

            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    // Grab the neighboring tiles
                    let neighboring_entities = Neighbors::get_square_neighboring_positions(
                        &TilePos { x, y },
                        map_size,
                        true,
                    )
                    .entities(tile_storage);

                    // Iterate over neighbors
                    for neighbor_entity in neighboring_entities.iter() {
                        // Query the tile entities to change the colors
                        if let Ok(mut tile_texture) = tile_query.get_mut(*neighbor_entity) {
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Accessing Tiles Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, update_map)
        .run();
}
