use std::marker::PhantomData;

use bevy::{
    asset::embedded_asset,
    core_pipeline::core_2d::Transparent2d,
    image::ImageSamplerDescriptor,
    mesh::MeshVertexAttribute,
    platform::collections::HashSet,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, extract_resource},
        render_phase::AddRenderCommand,
        render_resource::{FilterMode, SpecializedRenderPipelines, VertexFormat},
        sync_world::RenderEntity,
    },
    shader::load_shader_library,
};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderDevice;
#[cfg(not(feature = "atlas"))]
use bevy::render::texture::GpuImage;
use extract::remove_changed;

use crate::{
    TilemapFirstSet,
    tiles::{TilePos, TileStorage},
};
use crate::{
    prelude::TilemapTexture,
    render::{
        material::{MaterialTilemapPlugin, StandardTilemapMaterial},
        prepare::{MeshUniformResource, TilemapUniformResource},
    },
};

use self::{
    chunk::RenderChunk2dStorage, draw::DrawTilemap, pipeline::TilemapPipeline,
    queue::ImageBindGroups,
};

mod chunk;
mod draw;
mod extract;
pub mod material;
mod pipeline;
pub(crate) mod prepare;
mod queue;

#[cfg(not(feature = "atlas"))]
mod texture_array_cache;

#[cfg(not(feature = "atlas"))]
use self::extract::ExtractedTilemapTexture;
#[cfg(not(feature = "atlas"))]
pub(crate) use self::texture_array_cache::TextureArrayCache;

#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct ExtractedFilterMode(FilterMode);

#[derive(Resource, Deref)]
pub struct DefaultSampler(ImageSamplerDescriptor);

/// Size of the chunks used to render the tilemap.
///
/// Initialized from [`TilemapRenderSettings`](crate::map::TilemapRenderSettings) resource, if
/// provided. Otherwise, defaults to `64 x 64`.
#[derive(Debug, Copy, Clone, Deref)]
pub(crate) struct RenderChunkSize(UVec2);

impl RenderChunkSize {
    pub const fn new(chunk_size: UVec2) -> RenderChunkSize {
        RenderChunkSize(chunk_size)
    }

    /// Calculates the index of the chunk this tile is in.
    #[inline]
    pub fn map_tile_to_chunk(&self, tile_position: &TilePos) -> UVec2 {
        let tile_pos: UVec2 = tile_position.into();
        tile_pos / self.0
    }

    /// Calculates the index of this tile within the chunk.
    #[inline]
    pub fn map_tile_to_chunk_tile(&self, tile_position: &TilePos, chunk_position: &UVec2) -> UVec2 {
        let tile_pos: UVec2 = tile_position.into();
        tile_pos - (*chunk_position * self.0)
    }
}

pub struct TilemapRenderingPlugin;

impl Plugin for TilemapRenderingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "atlas"))]
        app.add_systems(Update, set_texture_to_copy_src);

        app.add_systems(First, clear_removed.in_set(TilemapFirstSet));

        app.add_observer(on_remove_tile);
        app.add_observer(on_remove_tilemap);

        app.add_plugins(ExtractComponentPlugin::<RemovedTileEntity>::default());
        app.add_plugins(ExtractComponentPlugin::<RemovedMapEntity>::default());

        app.add_plugins(MaterialTilemapPlugin::<StandardTilemapMaterial>::default());

        app.world_mut()
            .resource_mut::<Assets<StandardTilemapMaterial>>()
            .insert(
                Handle::<StandardTilemapMaterial>::default().id(),
                StandardTilemapMaterial::default(),
            )
            .unwrap();

        app.init_resource::<ModifiedImageIds>()
            .add_systems(Update, collect_modified_image_asset_messages);
    }

    fn finish(&self, app: &mut App) {
        let sampler = app.get_added_plugins::<ImagePlugin>().first().map_or_else(
            || ImagePlugin::default_nearest().default_sampler,
            |plugin| plugin.default_sampler.clone(),
        );

        load_shader_library!(app, "shaders/common.wesl");
        load_shader_library!(app, "shaders/mesh_output.wesl");
        load_shader_library!(app, "shaders/tilemap_vertex_output.wesl");
        load_shader_library!(app, "shaders/square.wesl");
        load_shader_library!(app, "shaders/diamond_iso.wesl");
        load_shader_library!(app, "shaders/staggered_iso.wesl");
        load_shader_library!(app, "shaders/column_hex.wesl");
        load_shader_library!(app, "shaders/column_even_hex.wesl");
        load_shader_library!(app, "shaders/column_odd_hex.wesl");
        load_shader_library!(app, "shaders/row_hex.wesl");
        load_shader_library!(app, "shaders/row_even_hex.wesl");
        load_shader_library!(app, "shaders/row_odd_hex.wesl");
        embedded_asset!(app, "shaders/tilemap_vertex.wesl");
        embedded_asset!(app, "shaders/tilemap_fragment.wesl");

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        render_app.init_resource::<TilemapPipeline>();

        #[cfg(not(feature = "atlas"))]
        render_app
            .init_resource::<TextureArrayCache>()
            .add_systems(
                Render,
                prepare_textures.in_set(RenderSystems::PrepareAssets),
            )
            .add_systems(Render, texture_array_cache::remove_modified_textures);

        render_app
            .insert_resource(DefaultSampler(sampler))
            .insert_resource(RenderChunk2dStorage::default())
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract,
                    extract_resource::<ModifiedImageIds, RenderApp, ()>,
                ),
            )
            .add_systems(
                Render,
                (prepare::prepare_removal, prepare::prepare)
                    .chain()
                    .in_set(RenderSystems::PrepareAssets),
            )
            .add_systems(
                Render,
                queue::queue_transform_bind_group.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(Render, remove_changed.in_set(RenderSystems::Cleanup))
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedRenderPipelines<TilemapPipeline>>()
            .init_resource::<MeshUniformResource>()
            .init_resource::<TilemapUniformResource>()
            .init_resource::<ModifiedImageIds>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();
    }
}

pub fn set_texture_to_copy_src(
    mut images: ResMut<Assets<Image>>,
    texture_query: Query<&TilemapTexture>,
) {
    // quick and dirty, run this for all textures anytime a texture component is created.
    for texture in texture_query.iter() {
        texture.set_images_to_copy_src(&mut images)
    }
}

/// Stores the index of a uniform inside of [`ComponentUniforms`].
#[derive(Component)]
pub struct DynamicUniformIndex<C: Component> {
    index: u32,
    marker: PhantomData<C>,
}

impl<C: Component> DynamicUniformIndex<C> {
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
}

pub const ATTRIBUTE_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Position", 229221259, VertexFormat::Float32x4);
pub const ATTRIBUTE_TEXTURE: MeshVertexAttribute =
    MeshVertexAttribute::new("Texture", 222922753, VertexFormat::Float32x4);
pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 231497124, VertexFormat::Float32x4);

#[derive(Component, ExtractComponent, Clone)]
#[extract_app(RenderApp)]
pub struct RemovedTileEntity(pub RenderEntity);

#[derive(Component, ExtractComponent, Clone)]
#[extract_app(RenderApp)]
pub struct RemovedMapEntity(pub RenderEntity);

fn on_remove_tile(
    removed: On<Remove<TilePos>>,
    mut commands: Commands,
    query: Query<&RenderEntity>,
) {
    if let Ok(render_entity) = query.get(removed.entity) {
        commands.spawn(RemovedTileEntity(*render_entity));
    }
}

fn on_remove_tilemap(
    removed: On<Remove<TileStorage>>,
    mut commands: Commands,
    query: Query<&RenderEntity>,
) {
    if let Ok(render_entity) = query.get(removed.entity) {
        commands.spawn(RemovedMapEntity(*render_entity));
    }
}

fn clear_removed(
    mut commands: Commands,
    removed_query: Query<Entity, With<RemovedTileEntity>>,
    removed_map_query: Query<Entity, With<RemovedMapEntity>>,
) {
    for entity in removed_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in removed_map_query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "atlas"))]
fn prepare_textures(
    render_device: Res<RenderDevice>,
    mut texture_array_cache: ResMut<TextureArrayCache>,
    extracted_tilemap_textures: Query<&ExtractedTilemapTexture>,
    render_images: Res<bevy::render::render_asset::RenderAssets<GpuImage>>,
) {
    for extracted_texture in extracted_tilemap_textures.iter() {
        texture_array_cache.add_extracted_texture(extracted_texture);
    }

    texture_array_cache.prepare(&render_device, &render_images);
}

/// Resource to hold the ids of modified Image assets of a single frame.
#[derive(Resource, ExtractResource, Clone, Default)]
#[extract_app(RenderApp)]
pub struct ModifiedImageIds(HashSet<AssetId<Image>>);

impl ModifiedImageIds {
    // Determines whether `texture` contains any handles of modified images.
    pub fn is_texture_modified(&self, texture: &TilemapTexture) -> bool {
        texture
            .image_handles()
            .iter()
            .any(|&image| self.0.contains(&image.id()))
    }
}

/// A system to collect the asset events of modified images for one frame.
/// AssetEvents cannot be read from the render sub-app, so this system packs
/// them up into a convenient resource which can be extracted for rendering.
pub fn collect_modified_image_asset_messages(
    mut asset_messages: MessageReader<AssetEvent<Image>>,
    mut modified_image_ids: ResMut<ModifiedImageIds>,
) {
    modified_image_ids.0.clear();

    for asset_event in asset_messages.read() {
        let id = match asset_event {
            AssetEvent::Modified { id } => id,
            _ => continue,
        };
        modified_image_ids.0.insert(*id);
    }
}
