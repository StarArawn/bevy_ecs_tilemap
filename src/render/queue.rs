use bevy::{
    core_pipeline::core_2d::Transparent2d,
    math::UVec4,
    prelude::{Commands, Component, Entity, Handle, Image, Msaa, Query, Res, ResMut, Transform},
    render::{
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, DynamicUniformBuffer,
            PipelineCache, SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        view::{ExtractedView, ViewUniforms},
    },
    utils::FloatOrd,
    utils::HashMap,
};

use crate::map::TilemapId;

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderQueue;

#[cfg(not(feature = "atlas"))]
use super::texture_array_cache::TextureArrayCache;

use super::{
    chunk::{ChunkId, RenderChunk2dStorage, TilemapUniformData},
    draw::DrawTilemap,
    pipeline::{TilemapPipeline, TilemapPipelineKey},
    prepare::MeshUniform,
};

pub struct TransformBindGroup {
    pub value: BindGroup,
}

pub fn queue_transform_bind_group(
    mut commands: Commands,
    tilemap_pipeline: Res<TilemapPipeline>,
    render_device: Res<RenderDevice>,
    transform_uniforms: Res<DynamicUniformBuffer<MeshUniform>>,
) {
    if let Some(binding) = transform_uniforms.binding() {
        commands.insert_resource(TransformBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("transform_bind_group"),
                layout: &tilemap_pipeline.mesh_layout,
            }),
        });
    }
}

pub struct TilemapUniformDataBindGroup {
    pub value: BindGroup,
}

pub fn queue_tilemap_bind_group(
    mut commands: Commands,
    tilemap_pipeline: Res<TilemapPipeline>,
    render_device: Res<RenderDevice>,
    tilemap_uniforms: Res<DynamicUniformBuffer<TilemapUniformData>>,
) {
    if let Some(binding) = tilemap_uniforms.binding() {
        commands.insert_resource(TilemapUniformDataBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("tilemap_bind_group"),
                layout: &tilemap_pipeline.uniform_layout,
            }),
        });
    }
}

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Default)]
pub struct ImageBindGroups {
    pub values: HashMap<Handle<Image>, BindGroup>,
}

#[allow(clippy::too_many_arguments)]
pub fn queue_meshes(
    mut commands: Commands,
    chunk_storage: Res<RenderChunk2dStorage>,
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    tilemap_pipeline: Res<TilemapPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TilemapPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    gpu_images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    standard_tilemap_meshes: Query<(Entity, &ChunkId, &Transform, &TilemapId)>,
    mut views: Query<(Entity, &ExtractedView, &mut RenderPhase<Transparent2d>)>,
    #[cfg(not(feature = "atlas"))] mut texture_array_cache: ResMut<TextureArrayCache>,
    #[cfg(not(feature = "atlas"))] render_queue: Res<RenderQueue>,
) {
    #[cfg(not(feature = "atlas"))]
    texture_array_cache.queue(&render_device, &render_queue, &gpu_images);

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for (entity, _view, mut transparent_phase) in views.iter_mut() {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("tilemap_view_bind_group"),
                layout: &tilemap_pipeline.view_layout,
            });

            commands.entity(entity).insert(TilemapViewBindGroup {
                value: view_bind_group,
            });

            let draw_tilemap = transparent_2d_draw_functions
                .read()
                .get_id::<DrawTilemap>()
                .unwrap();

            for (entity, chunk_id, transform, tilemap_id) in standard_tilemap_meshes.iter() {
                let chunk = chunk_storage.get(&UVec4::new(
                    chunk_id.0.x,
                    chunk_id.0.y,
                    chunk_id.0.z,
                    tilemap_id.0.id(),
                ));

                #[cfg(not(feature = "atlas"))]
                if !texture_array_cache.contains(&chunk.texture.0) {
                    continue;
                }

                #[cfg(feature = "atlas")]
                if gpu_images.get(&chunk.texture.0).is_none() {
                    continue;
                }

                image_bind_groups
                    .values
                    .entry(chunk.texture.0.clone_weak())
                    .or_insert_with(|| {
                        #[cfg(not(feature = "atlas"))]
                        let gpu_image = texture_array_cache.get(&chunk.texture.0);
                        #[cfg(feature = "atlas")]
                        let gpu_image = gpu_images.get(&chunk.texture.0).unwrap();
                        render_device.create_bind_group(&BindGroupDescriptor {
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&gpu_image.sampler),
                                },
                            ],
                            label: Some("sprite_material_bind_group"),
                            layout: &tilemap_pipeline.material_layout,
                        })
                    });

                let key = TilemapPipelineKey {
                    msaa: msaa.samples,
                    mesh_type: chunk.mesh_type,
                };

                let pipeline_id = pipelines.specialize(&mut pipeline_cache, &tilemap_pipeline, key);
                transparent_phase.add(Transparent2d {
                    entity,
                    draw_function: draw_tilemap,
                    pipeline: pipeline_id,
                    sort_key: FloatOrd(transform.translation.z as f32),
                    batch_range: None,
                });
            }
        }
    }
}
