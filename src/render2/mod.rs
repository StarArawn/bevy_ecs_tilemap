use bevy::{
    core_pipeline::Transparent2d,
    prelude::{App, Assets, Plugin},
    render2::{
        render_component::UniformComponentPlugin,
        render_phase::AddRenderCommand,
        render_resource::{Shader, SpecializedPipelines},
        RenderApp, RenderStage,
    },
};

use crate::render2::pipeline::{DrawTilemap, ImageBindGroups, MeshUniform, TILEMAP_SHADER_HANDLE, TilemapPipeline};

mod pipeline;
mod tilemap_data;

pub use tilemap_data::TilemapUniformData;

#[derive(Default)]
pub struct TilemapRenderPlugin;

impl Plugin for TilemapRenderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let pbr_shader = Shader::from_wgsl(include_str!("square.wgsl"));
        shaders.set_untracked(TILEMAP_SHADER_HANDLE, pbr_shader);

        app.add_plugin(UniformComponentPlugin::<MeshUniform>::default());
        app.add_plugin(UniformComponentPlugin::<TilemapUniformData>::default());

        let render_app = app.sub_app(RenderApp);
        render_app
            .add_system_to_stage(RenderStage::Extract, pipeline::extract_tilemaps)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_meshes)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_transform_bind_group)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_tilemap_bind_group)
            .init_resource::<TilemapPipeline>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<SpecializedPipelines<TilemapPipeline>>();

        render_app.add_render_command::<Transparent2d, DrawTilemap>();
    }
}
