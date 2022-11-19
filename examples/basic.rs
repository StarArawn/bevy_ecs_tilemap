use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

pub const MAP_SIZE: TilemapSize = TilemapSize { x: 32, y: 32 };
pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(not(feature = "atlas"))] array_texture_loader: Res<ArrayTextureLoader>,
) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    // Eventually, we will insert the `TilemapBundle` bundle on the entity, which
    // will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

    // To begin creating the map we will need a `TileStorage` component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
    // per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(MAP_SIZE);

    // Spawn the elements of the tilemap.
    // Alternatively, you can use helpers::filling::fill_tilemap.
    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let grid_size = TILE_SIZE.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: MAP_SIZE,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TILE_SIZE,
        transform: get_tilemap_center_transform(&MAP_SIZE, &grid_size, &map_type, 0.0),
        ..Default::default()
    });

    // Add atlas to array texture loader so it's preprocessed before we need to use it.
    // Only used when the atlas feature is off and we are using array textures.
    #[cfg(not(feature = "atlas"))]
    {
        array_texture_loader.add(TilemapArrayTexture {
            texture: TilemapTexture::Single(asset_server.load("tiles.png")),
            tile_size: TILE_SIZE,
            ..Default::default()
        });
    }
}

fn swap_texture_or_hide(
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TilemapTexture, &mut Visibility)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let texture_a = TilemapTexture::Single(asset_server.load("tiles.png"));
        let texture_b = TilemapTexture::Single(asset_server.load("tiles2.png"));
        for (mut tilemap_tex, _) in &mut query {
            if *tilemap_tex == texture_a {
                *tilemap_tex = texture_b.clone();
            } else {
                *tilemap_tex = texture_a.clone();
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::H) {
        for (_, mut visibility) in &mut query {
            if visibility.is_visible {
                visibility.is_visible = false;
            } else {
                visibility.is_visible = true;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            window: WindowDescriptor {
                width: (MAP_SIZE.x + 1) as f32 * TILE_SIZE.x,
                height: (MAP_SIZE.y + 1) as f32 * TILE_SIZE.y,
                title: String::from(
                    "Basic Example - Press Space to change Texture and H to show/hide tilemap.",
                ),
                ..Default::default()
            },
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(swap_texture_or_hide)
        .run();
}
