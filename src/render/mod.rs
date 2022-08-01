use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        extract_component::UniformComponentPlugin,
        mesh::MeshVertexAttribute,
        render_phase::AddRenderCommand,
        render_resource::{FilterMode, SpecializedRenderPipelines, VertexFormat},
        RenderApp, RenderStage,
    },
};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderDevice;

#[cfg(not(feature = "atlas"))]
use crate::{TextureSize, TileSize};

use crate::render::pipeline::{
    DrawTilemap, ImageBindGroups, MeshUniform, TilemapPipeline, HEX_COLUMN_EVEN_SHADER_HANDLE,
    HEX_COLUMN_ODD_SHADER_HANDLE, HEX_COLUMN_SHADER_HANDLE, HEX_ROW_EVEN_SHADER_HANDLE,
    HEX_ROW_ODD_SHADER_HANDLE, HEX_ROW_SHADER_HANDLE, ISO_DIAMOND_SHADER_HANDLE,
    ISO_STAGGERED_SHADER_HANDLE, SQUARE_SHADER_HANDLE,
};

mod include_shader;
mod pipeline;
mod tilemap_data;

pub use tilemap_data::TilemapUniformData;

#[cfg(not(feature = "atlas"))]
mod texture_array_cache;

#[cfg(not(feature = "atlas"))]
use self::texture_array_cache::TextureArrayCache;

#[derive(Default)]
pub struct TilemapRenderPlugin;

#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct ExtractedFilterMode(FilterMode);

pub const ATTRIBUTE_TEXTURE: MeshVertexAttribute =
    MeshVertexAttribute::new("Texture", 222922753, VertexFormat::Sint32x4);
pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Color", 231497124, VertexFormat::Float32x4);

impl Plugin for TilemapRenderPlugin {
    fn build(&self, app: &mut App) {
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

        app.add_plugin(UniformComponentPlugin::<MeshUniform>::default());
        app.add_plugin(UniformComponentPlugin::<TilemapUniformData>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_system_to_stage(RenderStage::Extract, pipeline::extract_tilemaps)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_meshes)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_transform_bind_group)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_tilemap_bind_group)
            .init_resource::<TilemapPipeline>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedRenderPipelines<TilemapPipeline>>();

        #[cfg(not(feature = "atlas"))]
        render_app
            .init_resource::<TextureArrayCache>()
            .add_system_to_stage(RenderStage::Prepare, prepare_textures);

        render_app.add_render_command::<Transparent2d, DrawTilemap>();
    }
}

#[cfg(not(feature = "atlas"))]
fn prepare_textures(
    render_device: Res<RenderDevice>,
    mut texture_array_cache: ResMut<TextureArrayCache>,
    extracted_query: Query<(&Handle<Image>, &TilemapUniformData, &ExtractedFilterMode)>,
) {
    for (atlas_image, tilemap_data, filter) in extracted_query.iter() {
        texture_array_cache.add(
            atlas_image,
            TileSize(tilemap_data.tile_size.x, tilemap_data.tile_size.y),
            TextureSize(tilemap_data.texture_size.x, tilemap_data.texture_size.y),
            tilemap_data.spacing,
            filter.0,
        );
    }

    texture_array_cache.prepare(&render_device);
}
