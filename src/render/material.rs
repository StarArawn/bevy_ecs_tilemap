use crate::prelude::{TilemapId, TilemapRenderSettings};
#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderQueue;
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{StaticSystemParam, SystemParamItem},
    log::error,
    math::FloatOrd,
    prelude::*,
    reflect::TypePath,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        globals::GlobalsBuffer,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, ViewSortedRenderPhases,
        },
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroup, BindGroupEntry, BindGroupLayout,
            BindingResource, OwnedBindingResource, PipelineCache, RenderPipelineDescriptor,
            ShaderRef, SpecializedRenderPipeline, SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        texture::GpuImage,
        view::{ExtractedView, RenderVisibleEntities, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{HashMap, HashSet},
};
use std::{hash::Hash, marker::PhantomData};

use super::{
    chunk::{ChunkId, RenderChunk2dStorage},
    draw::DrawTilemapMaterial,
    pipeline::{TilemapPipeline, TilemapPipelineKey},
    prepare,
    queue::{ImageBindGroups, TilemapViewBindGroup},
    ModifiedImageIds,
};

#[cfg(not(feature = "atlas"))]
pub(crate) use super::TextureArrayCache;

pub trait MaterialTilemap: AsBindGroup + Asset + Clone + Sized {
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
    fn specialize(descriptor: &mut RenderPipelineDescriptor, key: MaterialTilemapKey<Self>) {}
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
            tilemap_pipeline_key: self.tilemap_pipeline_key,
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

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect, PartialEq, Eq, ExtractComponent)]
#[reflect(Component, Default)]
pub struct MaterialTilemapHandle<M: MaterialTilemap>(pub Handle<M>);

impl<M: MaterialTilemap> Default for MaterialTilemapHandle<M> {
    fn default() -> Self {
        Self(Handle::default())
    }
}

impl<M: MaterialTilemap> From<Handle<M>> for MaterialTilemapHandle<M> {
    fn from(handle: Handle<M>) -> Self {
        Self(handle)
    }
}

impl<M: MaterialTilemap> From<MaterialTilemapHandle<M>> for AssetId<M> {
    fn from(tilemap: MaterialTilemapHandle<M>) -> Self {
        tilemap.id()
    }
}

impl<M: MaterialTilemap> From<&MaterialTilemapHandle<M>> for AssetId<M> {
    fn from(tilemap: &MaterialTilemapHandle<M>) -> Self {
        tilemap.id()
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
        app.init_asset::<M>()
            .add_plugins(ExtractComponentPlugin::<MaterialTilemapHandle<M>>::extract_visible());
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawTilemapMaterial<M>>()
                .init_resource::<MaterialTilemapPipeline<M>>()
                .init_resource::<ExtractedMaterialsTilemap<M>>()
                .init_resource::<RenderMaterialsTilemap<M>>()
                .init_resource::<SpecializedRenderPipelines<MaterialTilemapPipeline<M>>>()
                .add_systems(ExtractSchedule, extract_materials_tilemap::<M>)
                .add_systems(
                    Render,
                    prepare_materials_tilemap::<M>.in_set(RenderSet::PrepareAssets),
                )
                .add_systems(
                    Render,
                    (
                        // Ensure `queue_material_tilemap_meshes` runs after `prepare::prepare` because `prepare` calls `commands.spawn` with `ChunkId`
                        // and that data is then consumed by `queue_material_tilemap_mesh`. This is important because `prepare` is part of the `PrepareAssets`
                        // set. Bevy is loose on its expectation of when systems in the `PrepareAssets` set execute (for performance) and only needs them
                        // to run before the `Prepare` set (which is after Queue). This invites the possibility of an intermittent incorrect ordering dependent
                        // on the scheduler.
                        queue_material_tilemap_meshes::<M>
                            .in_set(RenderSet::Queue)
                            .after(prepare::prepare),
                        bind_material_tilemap_meshes::<M>.in_set(RenderSet::PrepareBindGroups),
                    ),
                );
        }
    }
}

pub struct PreparedMaterialTilemap<T: MaterialTilemap> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: T::Data,
}

#[derive(Resource)]
struct ExtractedMaterialsTilemap<M: MaterialTilemap> {
    extracted: Vec<(AssetId<M>, M)>,
    removed: Vec<AssetId<M>>,
}

impl<M: MaterialTilemap> Default for ExtractedMaterialsTilemap<M> {
    fn default() -> Self {
        Self {
            extracted: Default::default(),
            removed: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct MaterialTilemapPipeline<M: MaterialTilemap> {
    pub tilemap_pipeline: TilemapPipeline,
    pub material_tilemap_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<M: MaterialTilemap> Clone for MaterialTilemapPipeline<M> {
    fn clone(&self) -> Self {
        Self {
            tilemap_pipeline: self.tilemap_pipeline.clone(),
            material_tilemap_layout: self.material_tilemap_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: PhantomData,
        }
    }
}

impl<M: MaterialTilemap> SpecializedRenderPipeline for MaterialTilemapPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialTilemapKey<M>;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut descriptor = self.tilemap_pipeline.specialize(key.tilemap_pipeline_key);
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }
        descriptor.layout = vec![
            self.tilemap_pipeline.view_layout.clone(),
            self.tilemap_pipeline.mesh_layout.clone(),
            self.tilemap_pipeline.material_layout.clone(),
            self.material_tilemap_layout.clone(),
        ];

        M::specialize(&mut descriptor, key);
        descriptor
    }
}

impl<M: MaterialTilemap> FromWorld for MaterialTilemapPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();
        let material_tilemap_layout = M::bind_group_layout(render_device);

        MaterialTilemapPipeline {
            tilemap_pipeline: world.resource::<TilemapPipeline>().clone(),
            material_tilemap_layout,
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

/// Stores all prepared representations of [`Material2d`] assets for as long as they exist.
#[derive(Resource, Deref, DerefMut)]
pub struct RenderMaterialsTilemap<T: MaterialTilemap>(
    HashMap<AssetId<T>, PreparedMaterialTilemap<T>>,
);

impl<T: MaterialTilemap> Default for RenderMaterialsTilemap<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// This system extracts all created or modified assets of the corresponding [`Material2d`] type
/// into the "render world".
fn extract_materials_tilemap<M: MaterialTilemap>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    assets: Extract<Res<Assets<M>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed_assets.insert(id);
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(id);
                removed.push(*id);
            }
            _ => continue,
        }
    }

    let mut extracted_assets = Vec::new();
    for id in changed_assets.drain() {
        if let Some(asset) = assets.get(*id) {
            extracted_assets.push((*id, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedMaterialsTilemap {
        extracted: extracted_assets,
        removed,
    });
}

/// All [`Material2d`] values of a given type that should be prepared next frame.
pub struct PrepareNextFrameMaterials<M: MaterialTilemap> {
    assets: Vec<(AssetId<M>, M)>,
}

impl<M: MaterialTilemap> Default for PrepareNextFrameMaterials<M> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

/// This system prepares all assets of the corresponding [`Material2d`] type
/// which where extracted this frame for the GPU.
fn prepare_materials_tilemap<M: MaterialTilemap>(
    mut prepare_next_frame: Local<PrepareNextFrameMaterials<M>>,
    mut extracted_assets: ResMut<ExtractedMaterialsTilemap<M>>,
    mut render_materials: ResMut<RenderMaterialsTilemap<M>>,
    render_device: Res<RenderDevice>,
    pipeline: Res<MaterialTilemapPipeline<M>>,
    mut param: StaticSystemParam<M::Param>,
) {
    let queued_assets = std::mem::take(&mut prepare_next_frame.assets);
    for (handle, material) in queued_assets {
        match prepare_material_tilemap(&material, &render_device, &pipeline, &mut param) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
            Err(AsBindGroupError::InvalidSamplerType(_, _, _)) => {
                error!("Encountered AsBindGroupError::InvalidSamplerType while preparing material");
            }
        }
    }

    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_materials.remove(&removed);
    }

    for (handle, material) in std::mem::take(&mut extracted_assets.extracted) {
        match prepare_material_tilemap(&material, &render_device, &pipeline, &mut param) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
            Err(AsBindGroupError::InvalidSamplerType(_, _, _)) => {
                error!("Encountered AsBindGroupError::InvalidSamplerType while preparing material");
            }
        }
    }
}

fn prepare_material_tilemap<M: MaterialTilemap>(
    material: &M,
    render_device: &RenderDevice,
    pipeline: &MaterialTilemapPipeline<M>,
    param: &mut SystemParamItem<M::Param>,
) -> Result<PreparedMaterialTilemap<M>, AsBindGroupError> {
    let prepared =
        material.as_bind_group(&pipeline.material_tilemap_layout, render_device, param)?;
    Ok(PreparedMaterialTilemap {
        bindings: prepared.bindings,
        bind_group: prepared.bind_group,
        key: prepared.data,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn queue_material_tilemap_meshes<M: MaterialTilemap>(
    chunk_storage: Res<RenderChunk2dStorage>,
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    _render_device: Res<RenderDevice>,
    (material_tilemap_pipeline, mut material_pipelines): (
        Res<MaterialTilemapPipeline<M>>,
        ResMut<SpecializedRenderPipelines<MaterialTilemapPipeline<M>>>,
    ),
    pipeline_cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    globals_buffer: Res<GlobalsBuffer>,
    (standard_tilemap_meshes, materials): (
        Query<(Entity, &ChunkId, &Transform, &TilemapId)>,
        Query<&MaterialTilemapHandle<M>>,
    ),
    mut views: Query<(Entity, &ExtractedView, &Msaa, &RenderVisibleEntities)>,
    render_materials: Res<RenderMaterialsTilemap<M>>,
    #[cfg(not(feature = "atlas"))] (mut texture_array_cache, render_queue): (
        ResMut<TextureArrayCache>,
        Res<RenderQueue>,
    ),
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    #[cfg(not(feature = "atlas"))]
    texture_array_cache.queue(&_render_device, &render_queue, &gpu_images);

    if standard_tilemap_meshes.is_empty() {
        return;
    }

    if view_uniforms.uniforms.binding().is_none() && globals_buffer.buffer.binding().is_none() {
        return;
    }

    for (view_entity, view, msaa, visible_entities) in views.iter_mut() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let draw_tilemap = transparent_2d_draw_functions
            .read()
            .get_id::<DrawTilemapMaterial<M>>()
            .unwrap();

        for (entity, chunk_id, transform, tilemap_id) in standard_tilemap_meshes.iter() {
            if !visible_entities
                .iter::<With<TilemapRenderSettings>>()
                .any(|(entity, _main_entity)| entity.index() == tilemap_id.0.index())
            {
                continue;
            }

            let Ok(material_handle) = materials.get(tilemap_id.0) else {
                continue;
            };
            let Some(material) = render_materials.get(&material_handle.id()) else {
                continue;
            };

            if let Some(chunk) = chunk_storage.get(&UVec4::new(
                chunk_id.0.x,
                chunk_id.0.y,
                chunk_id.0.z,
                tilemap_id.0.index(),
            )) {
                #[cfg(not(feature = "atlas"))]
                if !texture_array_cache.contains(&chunk.texture) {
                    continue;
                }

                #[cfg(feature = "atlas")]
                if gpu_images.get(chunk.texture.image_handle()).is_none() {
                    continue;
                }

                let key = TilemapPipelineKey {
                    msaa: msaa.samples(),
                    map_type: chunk.get_map_type(),
                    hdr: view.hdr,
                };

                let pipeline_id = material_pipelines.specialize(
                    &pipeline_cache,
                    &material_tilemap_pipeline,
                    MaterialTilemapKey {
                        tilemap_pipeline_key: key,
                        bind_group_data: material.key.clone(),
                    },
                );
                let z = if chunk.y_sort {
                    transform.translation.z
                        + (1.0
                            - (transform.translation.y
                                / (chunk.map_size.y as f32 * chunk.tile_size.y)))
                } else {
                    transform.translation.z
                };
                transparent_phase.add(Transparent2d {
                    entity: (entity, tilemap_id.0.into()),
                    draw_function: draw_tilemap,
                    pipeline: pipeline_id,
                    sort_key: FloatOrd(z),
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::NONE,
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn bind_material_tilemap_meshes<M: MaterialTilemap>(
    mut commands: Commands,
    chunk_storage: Res<RenderChunk2dStorage>,
    render_device: Res<RenderDevice>,
    tilemap_pipeline: Res<TilemapPipeline>,
    view_uniforms: Res<ViewUniforms>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    globals_buffer: Res<GlobalsBuffer>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    (standard_tilemap_meshes, materials): (
        Query<(&ChunkId, &TilemapId)>,
        Query<&MaterialTilemapHandle<M>>,
    ),
    mut views: Query<(Entity, &RenderVisibleEntities)>,
    render_materials: Res<RenderMaterialsTilemap<M>>,
    modified_image_ids: Res<ModifiedImageIds>,
    #[cfg(not(feature = "atlas"))] (mut texture_array_cache, render_queue): (
        ResMut<TextureArrayCache>,
        Res<RenderQueue>,
    ),
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    #[cfg(not(feature = "atlas"))]
    texture_array_cache.queue(&render_device, &render_queue, &gpu_images);

    if standard_tilemap_meshes.is_empty() {
        return;
    }

    if let (Some(view_binding), Some(globals)) = (
        view_uniforms.uniforms.binding(),
        globals_buffer.buffer.binding(),
    ) {
        for (entity, visible_entities) in views.iter_mut() {
            let view_bind_group = render_device.create_bind_group(
                Some("tilemap_view_bind_group"),
                &tilemap_pipeline.view_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: view_binding.clone(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: globals.clone(),
                    },
                ],
            );

            commands.entity(entity).insert(TilemapViewBindGroup {
                value: view_bind_group,
            });

            for (chunk_id, tilemap_id) in standard_tilemap_meshes.iter() {
                if !visible_entities
                    .iter::<With<TilemapRenderSettings>>()
                    .any(|(entity, _main_entity)| entity.index() == tilemap_id.0.index())
                {
                    continue;
                }

                let Ok(material_handle) = materials.get(tilemap_id.0) else {
                    continue;
                };
                if render_materials.get(&material_handle.id()).is_none() {
                    continue;
                };

                if let Some(chunk) = chunk_storage.get(&UVec4::new(
                    chunk_id.0.x,
                    chunk_id.0.y,
                    chunk_id.0.z,
                    tilemap_id.0.index(),
                )) {
                    #[cfg(not(feature = "atlas"))]
                    if !texture_array_cache.contains(&chunk.texture) {
                        continue;
                    }

                    #[cfg(feature = "atlas")]
                    if gpu_images.get(chunk.texture.image_handle()).is_none() {
                        continue;
                    }

                    let create_bind_group = || {
                        #[cfg(not(feature = "atlas"))]
                        let gpu_image = texture_array_cache.get(&chunk.texture);
                        #[cfg(feature = "atlas")]
                        let gpu_image = gpu_images.get(chunk.texture.image_handle()).unwrap();
                        render_device.create_bind_group(
                            Some("sprite_material_bind_group"),
                            &tilemap_pipeline.material_layout,
                            &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&gpu_image.sampler),
                                },
                            ],
                        )
                    };
                    if modified_image_ids.is_texture_modified(&chunk.texture) {
                        image_bind_groups
                            .values
                            .insert(chunk.texture.clone_weak(), create_bind_group());
                    } else {
                        image_bind_groups
                            .values
                            .entry(chunk.texture.clone_weak())
                            .or_insert_with(create_bind_group);
                    }
                }
            }
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Default, TypePath, Asset)]
pub struct StandardTilemapMaterial {}

impl MaterialTilemap for StandardTilemapMaterial {}
