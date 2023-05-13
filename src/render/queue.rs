use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupDescriptor, BindGroupEntry},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use super::{
    pipeline::TilemapPipeline,
    prepare::{MeshUniformResource, TilemapUniformResource},
};
use crate::TilemapTexture;

#[derive(Resource)]
pub struct TransformBindGroup {
    pub value: BindGroup,
}

pub fn queue_transform_bind_group(
    mut commands: Commands,
    tilemap_pipeline: Res<TilemapPipeline>,
    render_device: Res<RenderDevice>,
    transform_uniforms: Res<MeshUniformResource>,
    tilemap_uniforms: Res<TilemapUniformResource>,
) {
    if let (Some(binding1), Some(binding2)) =
        (transform_uniforms.0.binding(), tilemap_uniforms.0.binding())
    {
        commands.insert_resource(TransformBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: binding1,
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: binding2,
                    },
                ],
                label: Some("transform_bind_group"),
                layout: &tilemap_pipeline.mesh_layout,
            }),
        });
    }
}

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Default, Resource)]
pub struct ImageBindGroups {
    pub values: HashMap<TilemapTexture, BindGroup>,
}
