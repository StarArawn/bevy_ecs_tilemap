use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        }, extract_component::ExtractComponentPlugin, render_phase::AddRenderCommand, RenderApp,
    },
};
use std::{hash::Hash, marker::PhantomData};

use super::pipeline::TilemapPipelineKey;

pub trait MaterialTilemap: AsBindGroup + Send + Sync + Clone + TypeUuid + Sized + 'static {
    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        key: MaterialTilemapKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

pub struct MaterialTilemapKey<M: MaterialTilemap> {
    pub tilemap_pipeline_key: TilemapPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: MaterialTilemap> Eq for MaterialTilemapKey<M> where M::Data: PartialEq {}

impl<M: MaterialTilemap> PartialEq for MaterialTilemapKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tilemap_pipeline_key == other.tilemap_pipeline_key
            && self.bind_group_data == other.bind_group_data
    }
}

impl<M: MaterialTilemap> Clone for MaterialTilemapKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            tilemap_pipeline_key: self.tilemap_pipeline_key.clone(),
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: MaterialTilemap> Hash for MaterialTilemapKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tilemap_pipeline_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

pub struct MaterialTilemapPlugin<M: MaterialTilemap>(PhantomData<M>);

impl<M: MaterialTilemap> Default for MaterialTilemapPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: MaterialTilemap> Plugin for MaterialTilemapPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_asset::<M>()
            .add_plugin(ExtractComponentPlugin::<Handle<M>>::extract_visible());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawMaterial2d<M>>()
                .init_resource::<Material2dPipeline<M>>()
                .init_resource::<ExtractedMaterials2d<M>>()
                .init_resource::<RenderMaterials2d<M>>()
                .init_resource::<SpecializedMeshPipelines<Material2dPipeline<M>>>()
                .add_system(extract_materials_2d::<M>.in_schedule(ExtractSchedule))
                .add_system(
                    prepare_materials_2d::<M>
                        .in_set(RenderSet::Prepare)
                        .after(PrepareAssetSet::PreAssetPrepare),
                )
                .add_system(queue_material2d_meshes::<M>.in_set(RenderSet::Queue));
        }
    }
}

