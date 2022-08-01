use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    math::UVec4,
    prelude::{Entity, Handle, Image},
    render::{
        mesh::GpuBufferInfo,
        render_phase::{RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::PipelineCache,
        view::ViewUniformOffset,
    },
};

use crate::map::TilemapId;

use super::{
    chunk::{ChunkId, RenderChunk2dStorage, TilemapUniformData},
    prepare::MeshUniform,
    queue::{
        ImageBindGroups, TilemapUniformDataBindGroup, TilemapViewBindGroup, TransformBindGroup,
    },
    DynamicUniformIndex,
};

pub struct SetMeshViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetMeshViewBindGroup<I> {
    type Param = SQuery<(Read<ViewUniformOffset>, Read<TilemapViewBindGroup>)>;
    #[inline]
    fn render<'w>(
        view: Entity,
        _item: &Transparent2d,
        view_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (view_uniform, pbr_view_bind_group) = view_query.get_inner(view).unwrap();
        pass.set_bind_group(I, &pbr_view_bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetTransformBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTransformBindGroup<I> {
    type Param = (
        SRes<TransformBindGroup>,
        SQuery<Read<DynamicUniformIndex<MeshUniform>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &Transparent2d,
        (transform_bind_group, mesh_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let transform_index = mesh_query.get(item.entity).unwrap();
        pass.set_bind_group(
            I,
            &transform_bind_group.into_inner().value,
            &[transform_index.index()],
        );

        RenderCommandResult::Success
    }
}

pub struct SetTilemapBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapBindGroup<I> {
    type Param = (
        SRes<TilemapUniformDataBindGroup>,
        SQuery<Read<DynamicUniformIndex<TilemapUniformData>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &Transparent2d,
        (tilemap_bind_group, mesh_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let tilemap_uniform_index = mesh_query.get(item.entity).unwrap();
        pass.set_bind_group(
            I,
            &tilemap_bind_group.into_inner().value,
            &[tilemap_uniform_index.index()],
        );

        RenderCommandResult::Success
    }
}

pub struct SetMaterialBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetMaterialBindGroup<I> {
    type Param = (SRes<ImageBindGroups>, SQuery<Read<Handle<Image>>>);
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &Transparent2d,
        (image_bind_groups, entities_with_images): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let image_handle = entities_with_images.get(item.entity).unwrap();
        let bind_group = image_bind_groups
            .into_inner()
            .values
            .get(image_handle)
            .unwrap();
        pass.set_bind_group(I, &bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub struct SetItemPipeline;
impl RenderCommand<Transparent2d> for SetItemPipeline {
    type Param = SRes<PipelineCache>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &Transparent2d,
        pipeline_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(item.pipeline)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub type DrawTilemap = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetTransformBindGroup<1>,
    SetTilemapBindGroup<2>,
    SetMaterialBindGroup<3>,
    DrawMesh,
);

pub struct DrawMesh;
impl RenderCommand<Transparent2d> for DrawMesh {
    type Param = (
        SRes<RenderChunk2dStorage>,
        SQuery<(Read<ChunkId>, Read<TilemapId>)>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &Transparent2d,
        (chunk_storage, chunk_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (chunk_id, tilemap_id) = chunk_query.get(item.entity).unwrap();
        let chunk = chunk_storage.into_inner().get(&UVec4::new(
            chunk_id.0.x,
            chunk_id.0.y,
            chunk_id.0.z,
            tilemap_id.0.id(),
        ));

        if let Some(gpu_mesh) = &chunk.gpu_mesh {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
        }

        RenderCommandResult::Success
    }
}
