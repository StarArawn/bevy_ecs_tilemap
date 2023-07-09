use std::marker::PhantomData;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexAttribute,
        render_phase::AddRenderCommand,
        render_resource::{
            FilterMode, SamplerDescriptor, SpecializedRenderPipelines, VertexFormat,
        },
        Render, RenderApp, RenderSet,
    },
};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderDevice;

use crate::render::prepare::{MeshUniformResource, TilemapUniformResource};
use crate::{
    prelude::{TilemapRenderSettings, TilemapTexture},
    tiles::{TilePos, TileStorage},
};

use self::{
    chunk::RenderChunk2dStorage,
    draw::DrawTilemap,
    pipeline::{TilemapPipeline, TILEMAP_SHADER_FRAGMENT, TILEMAP_SHADER_VERTEX},
    queue::ImageBindGroups,
};

mod chunk;
mod draw;
mod extract;
mod include_shader;
mod pipeline;
pub(crate) mod prepare;
mod queue;

#[cfg(not(feature = "atlas"))]
mod texture_array_cache;

#[cfg(not(feature = "atlas"))]
use self::extract::ExtractedTilemapTexture;
#[cfg(not(feature = "atlas"))]
pub(crate) use self::texture_array_cache::TextureArrayCache;

/// The default chunk_size (in tiles) used per mesh.
const CHUNK_SIZE_2D: UVec2 = UVec2::from_array([64, 64]);

#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct ExtractedFilterMode(FilterMode);

#[derive(Resource, Deref)]
pub struct DefaultSampler(SamplerDescriptor<'static>);

/// Size of the chunks used to render the tilemap.
///
/// Initialized from [`TilemapRenderSettings`](crate::map::TilemapRenderSettings) resource, if
/// provided. Otherwise, defaults to `64 x 64`.
#[derive(Resource, Debug, Copy, Clone, Deref)]
pub(crate) struct RenderChunkSize(UVec2);

impl RenderChunkSize {
    pub fn new(chunk_size: UVec2) -> RenderChunkSize {
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

/// Sorts chunks using Y sort during render.
///
/// Initialized from [`TilemapRenderSettings`](crate::map::TilemapRenderSettings) resource, if
/// provided. Otherwise, defaults to false.
#[derive(Resource, Debug, Copy, Clone, Deref)]
pub struct RenderYSort(bool);

pub struct TilemapRenderingPlugin;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SecondsSinceStartup(pub f32);

pub const COLUMN_EVEN_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7704924705970804993);
pub const COLUMN_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11710877199891728627);
pub const COLUMN_ODD_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6706359414982022142);
pub const COMMON: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15420881977837458322);
pub const DIAMOND_ISO: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6710251300621614118);
pub const MESH_OUTPUT: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2707251459590872179);
pub const ROW_EVEN_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7149718726759672633);
pub const ROW_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5506589682629967569);
pub const ROW_ODD_HEX: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13608302855194400936);
pub const STAGGERED_ISO: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9802843761568314416);
pub const SQUARE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7333720254399106799);
pub const TILEMAP_VERTEX_OUTPUT: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6104533649830094529);

impl Plugin for TilemapRenderingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "atlas"))]
        app.add_systems(Update, set_texture_to_copy_src);

        app.add_systems(First, clear_removed);
        app.add_systems(PostUpdate, (removal_helper_tilemap, removal_helper));

        // Extract the chunk size from the TilemapRenderSettings used to initialize the
        // ChunkCoordinate resource to insert into the render pipeline
        let (chunk_size, y_sort) = {
            match app.world.get_resource::<TilemapRenderSettings>() {
                Some(settings) => (settings.render_chunk_size, settings.y_sort),
                None => (CHUNK_SIZE_2D, false),
            }
        };

        load_internal_asset!(
            app,
            COLUMN_EVEN_HEX,
            "shaders/column_even_hex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            COLUMN_HEX,
            "shaders/column_hex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            COLUMN_ODD_HEX,
            "shaders/column_odd_hex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(app, COMMON, "shaders/common.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            DIAMOND_ISO,
            "shaders/diamond_iso.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ROW_EVEN_HEX,
            "shaders/row_even_hex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(app, ROW_HEX, "shaders/row_hex.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            ROW_ODD_HEX,
            "shaders/row_odd_hex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(app, ROW_HEX, "shaders/row_hex.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            MESH_OUTPUT,
            "shaders/mesh_output.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(app, SQUARE, "shaders/square.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            STAGGERED_ISO,
            "shaders/staggered_iso.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TILEMAP_VERTEX_OUTPUT,
            "shaders/tilemap_vertex_output.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TILEMAP_SHADER_VERTEX,
            "shaders/tilemap_vertex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TILEMAP_SHADER_FRAGMENT,
            "shaders/tilemap_fragment.wgsl",
            Shader::from_wgsl
        );

        let sampler = app.get_added_plugins::<ImagePlugin>().first().map_or_else(
            || ImagePlugin::default_nearest().default_sampler,
            |plugin| plugin.default_sampler.clone(),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(DefaultSampler(sampler))
            .insert_resource(RenderChunkSize(chunk_size))
            .insert_resource(RenderYSort(y_sort))
            .insert_resource(RenderChunk2dStorage::default())
            .insert_resource(SecondsSinceStartup(0.0))
            .add_systems(
                ExtractSchedule,
                (extract::extract, extract::extract_removal),
            )
            .add_systems(
                Render,
                (prepare::prepare_removal, prepare::prepare)
                    .chain()
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                (
                    queue::queue_meshes,
                    queue::queue_transform_bind_group,
                    queue::queue_tilemap_bind_group,
                )
                    .in_set(RenderSet::Queue),
            )
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedRenderPipelines<TilemapPipeline>>()
            .init_resource::<MeshUniformResource>()
            .init_resource::<TilemapUniformResource>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();

        #[cfg(not(feature = "atlas"))]
        render_app
            .init_resource::<TextureArrayCache>()
            .add_systems(Render, prepare_textures.in_set(RenderSet::Prepare));
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app.init_resource::<TilemapPipeline>();
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

#[derive(Component)]
pub struct RemovedTileEntity(pub Entity);

#[derive(Component)]
pub struct RemovedMapEntity(pub Entity);

fn removal_helper(mut commands: Commands, mut removed_query: RemovedComponents<TilePos>) {
    for entity in removed_query.iter() {
        commands.spawn(RemovedTileEntity(entity));
    }
}

fn removal_helper_tilemap(
    mut commands: Commands,
    mut removed_query: RemovedComponents<TileStorage>,
) {
    for entity in removed_query.iter() {
        commands.spawn(RemovedMapEntity(entity));
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
    render_images: Res<bevy::render::render_asset::RenderAssets<Image>>,
) {
    for extracted_texture in extracted_tilemap_textures.iter() {
        texture_array_cache.add_extracted_texture(extracted_texture);
    }

    texture_array_cache.prepare(&render_device, &render_images);
}
