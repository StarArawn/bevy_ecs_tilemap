mod helpers;

#[cfg(not(feature = "atlas"))]
mod no_atlas {
    use super::helpers;
    use bevy::prelude::*;
    use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
    use bevy_ecs_tilemap::prelude::*;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    const MAP_RADIUS: u32 = 10;
    const MAP_DIAMETER: u32 = 2 * MAP_RADIUS + 1;
    const MAP_CENTER: TilePos = TilePos {
        x: MAP_RADIUS,
        y: MAP_RADIUS,
    };
    // const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 54.0 };
    const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 56.0 };
    const COORD_SYS: HexCoordSystem = HexCoordSystem::Row;

    #[cfg(not(feature = "atlas"))]
    fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn(Camera2d);

        // Most of the work is happening bevy side. In this case, using the `ktx2` feature. If this
        // feature is not turned on, that the image won't properly be interpreted as a texture
        // container. The other alternative is `dds`.
        // let texture_vec = TilemapTexture::TextureContainer(asset_server.load("hex-tiles.ktx2"));

        // Optionally you can also load in KTX2's as regular single textures:
        let texture_vec = TilemapTexture::Vector(vec![
            asset_server.load("hex-tile-0.ktx2"),
            asset_server.load("hex-tile-1.ktx2"),
            asset_server.load("hex-tile-2.ktx2"),
        ]);

        let map_size = TilemapSize {
            x: MAP_DIAMETER,
            y: MAP_DIAMETER,
        };

        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_entity = commands.spawn_empty().id();
        let tilemap_id = TilemapId(tilemap_entity);

        let tile_positions = generate_hexagon(
            AxialPos::from_tile_pos_given_coord_system(&MAP_CENTER, COORD_SYS),
            MAP_RADIUS,
        )
        .into_iter()
        .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(COORD_SYS));

        let mut rng = thread_rng();
        let weighted_tile_choices = [
            (TileTextureIndex(0), 0.8),
            (TileTextureIndex(1), 0.1),
            (TileTextureIndex(2), 0.1),
        ];
        for position in tile_positions {
            let texture = weighted_tile_choices
                .choose_weighted(&mut rng, |choice| choice.1)
                .unwrap()
                .0;
            let tile_entity = commands
                .spawn(TileBundle {
                    position,
                    tilemap_id,
                    texture_index: texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&position, tile_entity);
        }

        let tile_size = TILE_SIZE;
        let grid_size = TILE_SIZE.into();
        let map_type = TilemapType::Hexagon(COORD_SYS);

        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size,
            map_type,
            tile_size,
            size: map_size,
            storage: tile_storage,
            texture: texture_vec,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        });
    }

    pub fn main() {
        App::new()
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: String::from("Using TilemapTexture::TextureContainer"),
                            ..Default::default()
                        }),
                        ..default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .add_plugins(TilemapPlugin)
            .add_systems(Startup, startup)
            .add_systems(Update, helpers::camera::movement)
            .run();
    }
}

fn main() {
    #[cfg(feature = "atlas")]
    panic!("Atlas feature does not support texture containers!");

    #[cfg(not(feature = "atlas"))]
    no_atlas::main();
}
