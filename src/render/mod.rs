use bevy::{
    core_pipeline::Transparent2d,
    prelude::*,
    render::{
        render_component::UniformComponentPlugin, render_phase::AddRenderCommand,
        render_resource::SpecializedPipelines, renderer::RenderDevice, RenderApp, RenderStage,
    },
};

use crate::{
    render::pipeline::{
        DrawTilemap, ImageBindGroups, MeshUniform, TilemapPipeline, HEX_COLUMN_EVEN_SHADER_HANDLE,
        HEX_COLUMN_ODD_SHADER_HANDLE, HEX_COLUMN_SHADER_HANDLE, HEX_ROW_EVEN_SHADER_HANDLE,
        HEX_ROW_ODD_SHADER_HANDLE, HEX_ROW_SHADER_HANDLE, ISO_DIAMOND_SHADER_HANDLE,
        ISO_STAGGERED_SHADER_HANDLE, SQUARE_SHADER_HANDLE,
    },
    TextureSize, TileSize,
};

mod include_shader;
mod pipeline;
mod texture_array_cache;
mod tilemap_data;

pub use tilemap_data::TilemapUniformData;

use self::texture_array_cache::TextureArrayCache;

#[derive(Default)]
pub struct TilemapRenderPlugin;

impl Plugin for TilemapRenderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let square_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/square.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(SQUARE_SHADER_HANDLE, square_shader);

        let iso_diamond_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/diamond_iso.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(ISO_DIAMOND_SHADER_HANDLE, iso_diamond_shader);

        let iso_staggered_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/staggered_iso.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(ISO_STAGGERED_SHADER_HANDLE, iso_staggered_shader);

        let hex_column_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_COLUMN_SHADER_HANDLE, hex_column_shader);

        let hex_column_odd_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_odd_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_COLUMN_ODD_SHADER_HANDLE, hex_column_odd_shader);

        let hex_column_even_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/column_even_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_COLUMN_EVEN_SHADER_HANDLE, hex_column_even_shader);

        let hex_row_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_ROW_SHADER_HANDLE, hex_row_shader);

        let hex_row_odd_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_odd_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_ROW_ODD_SHADER_HANDLE, hex_row_odd_shader);

        let hex_row_even_shader = Shader::from_wgsl(include_shader::include_shader(
            vec![include_str!("shaders/row_even_hex.wgsl")],
            include_str!("shaders/tilemap.wgsl"),
        ));
        shaders.set_untracked(HEX_ROW_EVEN_SHADER_HANDLE, hex_row_even_shader);

        app.add_plugin(UniformComponentPlugin::<MeshUniform>::default());
        app.add_plugin(UniformComponentPlugin::<TilemapUniformData>::default());

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<TextureArrayCache>()
            .add_system_to_stage(RenderStage::Extract, pipeline::extract_tilemaps)
            .add_system_to_stage(RenderStage::Prepare, prepare_textures)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_meshes)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_transform_bind_group)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_tilemap_bind_group)
            .init_resource::<TilemapPipeline>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedPipelines<TilemapPipeline>>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();
    }
}

fn prepare_textures(
    render_device: Res<RenderDevice>,
    mut texture_array_cache: ResMut<TextureArrayCache>,
    extracted_query: Query<(&Handle<Image>, &TilemapUniformData)>,
) {
    for (atlas_image, tilemap_data) in extracted_query.iter() {
        texture_array_cache.add(
            atlas_image,
            TileSize(tilemap_data.tile_size.x, tilemap_data.tile_size.y),
            TextureSize(tilemap_data.texture_size.x, tilemap_data.texture_size.y),
            tilemap_data.spacing,
        );
    }

    texture_array_cache.prepare(&render_device);
}
