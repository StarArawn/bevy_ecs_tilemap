use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        mesh::MeshVertexAttribute,
        render_phase::AddRenderCommand,
        render_resource::{
            DynamicUniformBuffer, FilterMode, SpecializedRenderPipelines, TextureUsages,
            VertexFormat,
        },
        RenderApp, RenderStage,
    },
};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderDevice;

use crate::{
    prelude::{TilemapRenderSettings, TilemapTexture},
    tiles::{TilePos, TileStorage},
};

use self::{
    chunk::{RenderChunk2dStorage, TilemapUniformData},
    draw::DrawTilemap,
    pipeline::{
        TilemapPipeline, HEX_COLUMN_EVEN_SHADER_HANDLE, HEX_COLUMN_ODD_SHADER_HANDLE,
        HEX_COLUMN_SHADER_HANDLE, HEX_ROW_EVEN_SHADER_HANDLE, HEX_ROW_ODD_SHADER_HANDLE,
        HEX_ROW_SHADER_HANDLE, ISO_DIAMOND_SHADER_HANDLE, ISO_STAGGERED_SHADER_HANDLE,
        SQUARE_SHADER_HANDLE,
    },
    prepare::MeshUniform,
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

/// Size of the chunks used to render the tilemap.
///
/// Initialized from [`TilemapRenderSettings`](crate::map::TilemapRenderSettings) resource, if
/// provided. Otherwise, defaults to `64 x 64`.
#[derive(Debug, Copy, Clone, Deref)]
pub(crate) struct RenderChunkSize(UVec2);

impl RenderChunkSize {
    pub fn new(chunk_size: UVec2) -> RenderChunkSize {
        RenderChunkSize(chunk_size)
    }

    #[inline]
    pub fn map_tile_to_chunk(&self, tile_position: &TilePos) -> UVec2 {
        let tile_pos: UVec2 = tile_position.into();
        tile_pos / self.0
    }

    #[inline]
    pub fn map_tile_to_chunk_tile(&self, tile_position: &TilePos, chunk_position: &UVec2) -> UVec2 {
        let tile_pos: UVec2 = tile_position.into();
        tile_pos - (*chunk_position * self.0)
    }
}

pub struct TilemapRenderingPlugin;
#[derive(Default, Deref, DerefMut)]
pub struct SecondsSinceStartup(f32);

impl Plugin for TilemapRenderingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "atlas"))]
        app.add_system(set_texture_to_copy_src);

        app.add_system_to_stage(CoreStage::First, clear_removed);
        app.add_system_to_stage(CoreStage::PostUpdate, removal_helper_tilemap);
        app.add_system_to_stage(CoreStage::PostUpdate, removal_helper);

        // Extract the chunk size from the TilemapRenderSettings used to initialize the
        // ChunkCoordinate resource to insert into the render pipeline
        let chunk_size = {
            match app.world.get_resource::<TilemapRenderSettings>() {
                Some(settings) => settings.render_chunk_size,
                None => CHUNK_SIZE_2D,
            }
        };

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

        #[cfg(not(feature = "atlas"))]
        let tilemap_shader = include_str!("shaders/tilemap.wgsl");
        #[cfg(feature = "atlas")]
        let tilemap_shader = include_str!("shaders/tilemap-atlas.wgsl");

        let square_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/square.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(SQUARE_SHADER_HANDLE, square_shader);

        let iso_diamond_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/diamond_iso.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(ISO_DIAMOND_SHADER_HANDLE, iso_diamond_shader);

        let iso_staggered_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/staggered_iso.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(ISO_STAGGERED_SHADER_HANDLE, iso_staggered_shader);

        let hex_column_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_COLUMN_SHADER_HANDLE, hex_column_shader);

        let hex_column_odd_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_odd_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_COLUMN_ODD_SHADER_HANDLE, hex_column_odd_shader);

        let hex_column_even_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_even_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_COLUMN_EVEN_SHADER_HANDLE, hex_column_even_shader);

        let hex_row_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_ROW_SHADER_HANDLE, hex_row_shader);

        let hex_row_odd_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_odd_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_ROW_ODD_SHADER_HANDLE, hex_row_odd_shader);

        let hex_row_even_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_even_hex.wgsl")],
            tilemap_shader,
        ));
        shaders.set_untracked(HEX_ROW_EVEN_SHADER_HANDLE, hex_row_even_shader);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(RenderChunkSize(chunk_size))
            .insert_resource(RenderChunk2dStorage::default())
            .insert_resource(SecondsSinceStartup);
        render_app
            .add_system_to_stage(RenderStage::Extract, extract::extract)
            .add_system_to_stage(RenderStage::Extract, extract::extract_removal);
        render_app
            .add_system_to_stage(RenderStage::Prepare, prepare::prepare)
            .add_system_to_stage(RenderStage::Prepare, prepare::prepare_removal)
            .add_system_to_stage(RenderStage::Queue, queue::queue_meshes)
            .add_system_to_stage(RenderStage::Queue, queue::queue_transform_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue::queue_tilemap_bind_group)
            .init_resource::<TilemapPipeline>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedRenderPipelines<TilemapPipeline>>()
            .init_resource::<DynamicUniformBuffer<MeshUniform>>()
            .init_resource::<DynamicUniformBuffer<TilemapUniformData>>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();

        #[cfg(not(feature = "atlas"))]
        render_app
            .init_resource::<TextureArrayCache>()
            .add_system_to_stage(RenderStage::Prepare, prepare_textures);
    }
}

pub fn set_texture_to_copy_src(mut textures: ResMut<Assets<Image>>, query: Query<&TilemapTexture>) {
    // quick and dirty, run this for all textures anytime a texture component is created.
    for texture in query.iter() {
        if let Some(mut texture) = textures.get_mut(&texture.0) {
            if !texture
                .texture_descriptor
                .usage
                .contains(TextureUsages::COPY_SRC)
            {
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST;
            }
        }
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

fn removal_helper(mut commands: Commands, removed_query: RemovedComponents<TilePos>) {
    for entity in removed_query.iter() {
        commands.spawn().insert(RemovedTileEntity(entity));
    }
}

fn removal_helper_tilemap(mut commands: Commands, removed_query: RemovedComponents<TileStorage>) {
    for entity in removed_query.iter() {
        commands.spawn().insert(RemovedMapEntity(entity));
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
    extracted_tilemaps: Query<&ExtractedTilemapTexture>,
) {
    for tilemap in extracted_tilemaps.iter() {
        let tile_size: Vec2 = tilemap.tile_size.into();
        let texture_size: Vec2 = tilemap.texture_size.into();
        let spacing: Vec2 = tilemap.spacing.into();
        texture_array_cache.add(
            &tilemap.texture.0,
            tile_size,
            texture_size,
            spacing,
            FilterMode::Nearest,
        );
    }

    texture_array_cache.prepare(&render_device);
}
