use bevy::{
    camera::RenderTarget,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits},
        pointer::{PointerId, PointerLocation},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::prelude::*;

pub struct TilemapPickingPlugin;

impl Plugin for TilemapPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, tile_picking.in_set(PickingSystems::Backend));
    }
}

fn tile_picking(
    pointers_q: Query<(&PointerId, &PointerLocation)>,
    cameras_q: Query<(
        Entity,
        &Camera,
        &RenderTarget,
        &GlobalTransform,
        &Projection,
    )>,
    primary_window_q: Query<Entity, With<PrimaryWindow>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapAnchor,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
        &ViewVisibility,
    )>,
    tiles_q: Query<&TileVisible>,
    mut output: MessageWriter<PointerHits>,
) {
    for (p_id, p_loc) in pointers_q
        .iter()
        .filter_map(|(p_id, p_loc)| p_loc.location().map(|l| (p_id, l)))
    {
        let Some((
            cam_entity,
            camera,
            _cam_target,
            cam_transform,
            Projection::Orthographic(cam_ortho),
        )) = cameras_q
            .iter()
            .filter(|(_, camera, ..)| camera.is_active)
            .find(|(_, _, cam_target, ..)| {
                cam_target
                    .normalize(match primary_window_q.single() {
                        Ok(w) => Some(w),
                        Err(_) => return false,
                    })
                    .as_ref()
                    == Some(&p_loc.target)
            })
        else {
            continue;
        };

        let Ok(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, p_loc.position)
        else {
            continue;
        };

        let picks = tilemap_q
            .iter()
            .filter(|(.., vis)| vis.get())
            .filter_map(
                |(
                    map_s,
                    map_grid_size,
                    map_tile_size,
                    map_anchor,
                    map_type,
                    map_store,
                    map_transform,
                    ..,
                )| {
                    let in_map_pos = {
                        let pos = Vec4::from((cursor_pos_world, 0., 1.0));
                        let in_map_pos = map_transform.to_matrix().inverse() * pos;
                        in_map_pos.xy()
                    };

                    let tile_entity = TilePos::from_world_pos(
                        &in_map_pos,
                        map_s,
                        map_grid_size,
                        map_tile_size,
                        map_type,
                        map_anchor,
                    )
                    .and_then(|tile_pos| map_store.get(&tile_pos))
                    .filter(|&tile_entity| {
                        tiles_q.get(tile_entity).is_ok_and(|visible| visible.0)
                    })?;

                    let hit_pos_world = map_transform.transform_point(in_map_pos.extend(0.0));

                    let hit_pos_cam = cam_transform
                        .affine()
                        .inverse()
                        .transform_point3(hit_pos_world);

                    let depth = -cam_ortho.near - hit_pos_cam.z;

                    Some((tile_entity, HitData::new(cam_entity, depth, None, None)))
                },
            )
            .collect::<Vec<_>>();

        output.write(PointerHits::new(*p_id, picks, camera.order as f32));
    }
}
