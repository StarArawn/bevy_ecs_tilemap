use std::io::BufReader;

use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    prelude::{
        AddAsset, Added, AssetEvent, Assets, Bundle, Commands, Component, DespawnRecursiveExt,
        Entity, EventReader, GlobalTransform, Handle, Image, Plugin, Query, Res, Transform,
    },
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_ecs_tilemap::prelude::*;

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .add_system(process_loaded_maps);
    }
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilesets: HashMap<u32, Handle<Image>>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let root_dir = load_context.path().parent().unwrap();
            let map = tiled::parse(BufReader::new(bytes))?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for tileset in &map.tilesets {
                let tile_path = root_dir.join(tileset.images.first().unwrap().source.as_str());
                let asset_path = AssetPath::new(tile_path, None);
                let texture: Handle<Image> = load_context.get_handle(asset_path.clone());

                for i in tileset.first_gid..(tileset.first_gid + tileset.tilecount.unwrap_or(1)) {
                    handles.insert(i, texture.clone());
                }

                dependencies.push(asset_path);
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map,
                tilesets: handles,
            });
            load_context.set_default_asset(loaded_asset.with_dependencies(dependencies));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(&Handle<TiledMap>, &mut TiledLayersStorage)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    for event in map_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                log::info!("Map added!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                log::info!("Map changed!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Removed { handle } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps = changed_maps
                    .into_iter()
                    .filter(|changed_handle| changed_handle == handle)
                    .collect();
            }
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone_weak());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                // TODO: Create a RemoveMap component..
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn_recursive()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                for tileset in tiled_map.map.tilesets.iter() {
                    // Once materials have been created/added we need to then create the layers.
                    for layer in tiled_map.map.layers.iter() {
                        let tile_size = TilemapTileSize {
                            x: tileset.tile_width,
                            y: tileset.tile_height,
                        };
                        let tile_spacing = TilemapSpacing {
                            x: tileset.spacing,
                            y: tileset.spacing,
                        };

                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        let mesh_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => TilemapType::Isometric {
                                diagonal_neighbors: false,
                                coord_system: IsoCoordSystem::Diamond,
                            },
                            tiled::Orientation::Staggered => TilemapType::Isometric {
                                diagonal_neighbors: false,
                                coord_system: IsoCoordSystem::Staggered,
                            },
                            tiled::Orientation::Orthogonal => TilemapType::Square {
                                diagonal_neighbors: false,
                            },
                        };

                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn().id();

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                let mut mapped_y = x;
                                if tiled_map.map.orientation == tiled::Orientation::Orthogonal {
                                    mapped_y = (tiled_map.map.height - 1) as u32 - y;
                                }

                                let mapped_x = x as usize;
                                let mapped_y = mapped_y as usize;

                                let map_tile = match &layer.tiles {
                                    tiled::LayerData::Finite(tiles) => &tiles[mapped_y][mapped_x],
                                    _ => panic!("Infinite maps not supported"),
                                };

                                if map_tile.gid < tileset.first_gid
                                    || map_tile.gid
                                        >= tileset.first_gid + tileset.tilecount.unwrap()
                                {
                                    continue;
                                }

                                let tile_id = map_tile.gid - tileset.first_gid;

                                let tile_pos = TilePos { x, y };
                                let tile_entity = commands
                                    .spawn()
                                    .insert_bundle(TileBundle {
                                        position: tile_pos,
                                        tilemap_id: TilemapId(layer_entity),
                                        texture: TileTexture(tile_id),
                                        flip: TileFlip {
                                            x: map_tile.flip_h,
                                            y: map_tile.flip_v,
                                            d: map_tile.flip_d,
                                        },
                                        ..Default::default()
                                    })
                                    .id();
                                tile_storage.set(&tile_pos, tile_entity);
                            }
                        }

                        commands.entity(layer_entity).insert_bundle(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: TilemapTexture::Single(
                                tiled_map
                                    .tilesets
                                    .get(&tileset.first_gid)
                                    .unwrap()
                                    .clone_weak(),
                            ),
                            tile_size,
                            spacing: tile_spacing,
                            transform: get_tilemap_center_transform(
                                &map_size,
                                &grid_size,
                                layer.layer_index as f32,
                            ) * Transform::from_xyz(offset_x, -offset_y, 0.0),
                            map_type: mesh_type,
                            ..Default::default()
                        });

                        layer_storage
                            .storage
                            .insert(layer.layer_index, layer_entity);
                    }
                }
            }
        }
    }
}
