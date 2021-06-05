use std::collections::{HashMap};
use crate::prelude::*;

use bevy::{asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset}, prelude::*};
use bevy::reflect::TypeUuid;

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct LdtkMap {
    pub project: ldtk_rust::Project,
    pub tilesets: HashMap<i64, Handle<Texture>>,
}


#[derive(Default, Bundle)]
pub struct LdtkMapBundle {
    pub ldtk_map: Handle<LdtkMap>,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct LdtkLoader;

impl AssetLoader for LdtkLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let project: ldtk_rust::Project = serde_json::from_slice(bytes)?;
            let dependencies: Vec<(i64, AssetPath)> = project.defs.tilesets.iter().map(|tileset| {
                (tileset.uid, load_context.path().parent().unwrap().join(tileset.rel_path.clone()).into())
            }).collect();

            let loaded_asset = LoadedAsset::new(LdtkMap {
                project,
                tilesets: dependencies.iter().map(|dep| (dep.0, load_context.get_handle(dep.1.clone()))).collect()
            });
            load_context.set_default_asset(loaded_asset.with_dependencies(dependencies.iter().map(|x| x.1.clone()).collect()));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ldtk"];
        EXTENSIONS
    }
}

pub fn process_loaded_tile_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<LdtkMap>>,
    maps: Res<Assets<LdtkMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(
        Entity,
        &Handle<LdtkMap>,
        &mut Map,
    )>,
    new_maps: Query<&Handle<LdtkMap>, Added<Handle<LdtkMap>>>,
    layer_query: Query<&Layer>,
    chunk_query: Query<&Chunk>,
) {
    let mut changed_maps = Vec::<Handle<LdtkMap>>::default();
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
                changed_maps = changed_maps.into_iter().filter(|changed_handle| changed_handle == handle).collect();
            }
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone());
    }

    for changed_map in changed_maps.iter() {
        for (_, map_handle, mut map) in query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            if let Some(ldtk_map) = maps.get(map_handle) {
                // Despawn all tiles/chunks/layers.
                for (layer_id, layer_entity) in map.get_layers() {
                    if let Ok(layer) = layer_query.get(layer_entity) {
                        for x in 0..layer.get_layer_size_in_tiles().x {
                            for y in 0..layer.get_layer_size_in_tiles().y {
                                let tile_pos = UVec2::new(x, y);
                                let chunk_pos = UVec2::new(
                                    tile_pos.x / layer.settings.chunk_size.x,
                                    tile_pos.y / layer.settings.chunk_size.y,
                                );
                                if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                                    if let Ok(chunk) = chunk_query.get(chunk_entity) {
                                        let chunk_tile_pos = chunk.to_chunk_pos(tile_pos);
                                        if let Some(tile) = chunk.get_tile_entity(chunk_tile_pos) {
                                            commands.entity(tile).despawn_recursive();
                                        }
                                    }

                                    commands.entity(chunk_entity).despawn_recursive();
                                }
                            }
                        }
                    }
                    map.remove_layer(&mut commands, layer_id);
                }

                // Pull out tilesets.
                let mut tilesets = HashMap::new();
                ldtk_map.project.defs.tilesets.iter().for_each(|tileset| {
                    tilesets.insert(tileset.uid, (ldtk_map.tilesets.get(&tileset.uid).unwrap().clone(), tileset.clone()));
                });

                let default_grid_size = ldtk_map.project.default_grid_size;

                let map_tile_count_x = (ldtk_map.project.levels[0].px_wid / default_grid_size) as u32;
                let map_tile_count_y = (ldtk_map.project.levels[0].px_hei / default_grid_size) as u32;

                let map_size = UVec2::new(
                    (map_tile_count_x as f32 / 32.0).ceil() as u32,
                    (map_tile_count_y as f32 / 32.0).ceil() as u32,
                );

                for (layer_id, layer) in ldtk_map.project.levels[0].layer_instances.as_ref().unwrap().iter().rev().enumerate() {
                    let (texture, tileset) = if let Some(uid) = layer.tileset_def_uid {
                        tilesets.get(&uid).unwrap().clone()
                    } else {
                        continue;
                    };
                    
                    let mut settings = LayerSettings::new(
                        map_size,
                        UVec2::new(32, 32),
                        Vec2::new(tileset.tile_grid_size as f32, tileset.tile_grid_size as f32),
                        Vec2::new(tileset.px_wid as f32, tileset.px_hei as f32)
                    );
                    settings.set_layer_id(layer_id as u16);

                    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
                        &mut commands,
                        settings,
                        map.id,
                        layer_id as u16,
                        None,
                    );

                    let tileset_width_in_tiles = (tileset.px_wid / default_grid_size) as u32;

                    for tile in layer.grid_tiles.iter() {
                        let tileset_x = (tile.src[0] / default_grid_size) as u32;
                        let tileset_y = (tile.src[1] / default_grid_size) as u32;

                        let mut pos = UVec2::new(
                            (tile.px[0] / default_grid_size) as u32,
                            (tile.px[1] / default_grid_size) as u32
                        );

                        pos.y = map_tile_count_y - pos.y;

                        layer_builder.set_tile(
                            pos,
                            Tile {
                                texture_index: (tileset_y * tileset_width_in_tiles + tileset_x) as u16,
                                ..Default::default()
                            }.into()
                        ).unwrap();
                    }

                    let material_handle = materials.add(ColorMaterial::texture(texture));
                    let layer_bundle = layer_builder.build(&mut commands, &mut meshes, material_handle);
                    let mut layer = layer_bundle.layer;
                    let mut transform = Transform::from_xyz(0.0, -ldtk_map.project.levels[0].px_hei as f32, layer_bundle.transform.translation.z);
                    layer.settings.layer_id = layer.settings.layer_id;
                    transform.translation.z = layer.settings.layer_id as f32;
                    map.add_layer(&mut commands, layer.settings.layer_id, layer_entity);
                    commands
                        .entity(layer_entity)
                        .insert_bundle(LayerBundle {
                            layer,
                            transform,
                            ..layer_bundle
                        });
                }
            }
        }
    }
}

/// Adds the default systems and pipelines used by bevy_ecs_tilemap::ldtk.
#[derive(Default)]
pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_asset::<LdtkMap>()
            .add_asset_loader(LdtkLoader)
            .add_system(process_loaded_tile_maps.system());
    }
}