use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{
            BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction,
            DepthBiasState, DepthStencilState, PipelineDescriptor, RenderPipeline,
            StencilFaceState, StencilState,
        },
        render_graph::{base, RenderGraph, RenderResourcesNode},
        shader::{ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};

use crate::TilemapMeshType;

use super::TilemapData;

macro_rules! create_chunk_pipeline {
    ($pipeline_handle: ident, $pipeline_id: expr, $function: ident, $vert_file: expr, $frag_file: expr) => {
        /// The constant render pipeline for a chunk.
        pub(crate) const $pipeline_handle: HandleUntyped =
            HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, $pipeline_id);

        /// Builds the chunk render pipeline.
        fn $function(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
            PipelineDescriptor {
                color_target_states: vec![ColorTargetState {
                    format: TextureFormat::default(),
                    color_blend: BlendState {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha_blend: BlendState {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    write_mask: ColorWrite::ALL,
                }],
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil: StencilState {
                        front: StencilFaceState::IGNORE,
                        back: StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: DepthBiasState {
                        constant: 0,
                        slope_scale: 0.0,
                        clamp: 0.0,
                    },
                    clamp_depth: false,
                }),
                ..PipelineDescriptor::new(ShaderStages {
                    vertex: shaders.add(Shader::from_glsl(
                        ShaderStage::Vertex,
                        include_str!($vert_file),
                    )),
                    fragment: Some(shaders.add(Shader::from_glsl(
                        ShaderStage::Fragment,
                        include_str!($frag_file),
                    ))),
                })
            }
        }
    };
}

create_chunk_pipeline!(
    SQUARE_PIPELINE,
    8094008129742001941,
    create_square_pipeline,
    "square-tilemap.vert",
    "square-tilemap.frag"
);

create_chunk_pipeline!(
    DIAMOND_ISO_PIPELINE,
    5716002228110903793,
    create_iso_diamond_pipeline,
    "diamondiso-tilemap.vert",
    "iso-tilemap.frag"
);

create_chunk_pipeline!(
    STAGGERED_ISO_PIPELINE,
    6571326172373592468,
    create_iso_staggered_pipeline,
    "staggerediso-tilemap.vert",
    "iso-tilemap.frag"
);

create_chunk_pipeline!(
    COLUMN_EVEN_HEX_PIPELINE,
    5336568075571462317,
    create_hex_column_even_pipeline,
    "columnevenhex-tilemap.vert",
    "hex-tilemap.frag"
);

create_chunk_pipeline!(
    COLUMN_ODD_HEX_PIPELINE,
    11472021184100190415,
    create_hex_column_odd_pipeline,
    "columnoddhex-tilemap.vert",
    "hex-tilemap.frag"
);

create_chunk_pipeline!(
    COLUMN_HEX_PIPELINE,
    12158158650956014109,
    create_hex_column_pipeline,
    "columnhex-tilemap.vert",
    "hex-tilemap.frag"
);

create_chunk_pipeline!(
    ROW_EVEN_HEX_PIPELINE,
    14433932828806852042,
    create_hex_row_even_pipeline,
    "rowevenhex-tilemap.vert",
    "hex-tilemap.frag"
);

create_chunk_pipeline!(
    ROW_ODD_HEX_PIPELINE,
    14864388685772956547,
    create_hex_row_odd_pipeline,
    "rowoddhex-tilemap.vert",
    "hex-tilemap.frag"
);

create_chunk_pipeline!(
    ROW_HEX_PIPELINE,
    15900471900964169180,
    create_hex_row_pipeline,
    "rowhex-tilemap.vert",
    "hex-tilemap.frag"
);

pub mod node {
    pub const TILEMAP_DATA: &'static str = "tile_map_data";
}

impl Into<RenderPipelines> for TilemapMeshType {
    fn into(self) -> RenderPipelines {
        match self {
            TilemapMeshType::Square => {
                RenderPipelines::from_pipelines(vec![RenderPipeline::new(SQUARE_PIPELINE.typed())])
            }
            TilemapMeshType::Isometric(iso_type) => match iso_type {
                crate::IsoType::Diamond => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        DIAMOND_ISO_PIPELINE.typed(),
                    )])
                }
                crate::IsoType::Staggered => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        STAGGERED_ISO_PIPELINE.typed(),
                    )])
                }
            }
            TilemapMeshType::Hexagon(hex_type) => match hex_type {
                crate::HexType::Column => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        COLUMN_HEX_PIPELINE.typed(),
                    )])
                }
                crate::HexType::ColumnEven => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        COLUMN_EVEN_HEX_PIPELINE.typed(),
                    )])
                }
                crate::HexType::ColumnOdd => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        COLUMN_ODD_HEX_PIPELINE.typed(),
                    )])
                }
                crate::HexType::Row => RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    ROW_HEX_PIPELINE.typed(),
                )]),
                crate::HexType::RowEven => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        ROW_EVEN_HEX_PIPELINE.typed(),
                    )])
                }
                crate::HexType::RowOdd => {
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        ROW_ODD_HEX_PIPELINE.typed(),
                    )])
                }
            },
        }
    }
}

pub(crate) fn add_tile_map_graph(world: &mut World) {
    world.resource_scope(|world, mut pipelines: Mut<Assets<PipelineDescriptor>>| {
        world.resource_scope(|world, mut shaders: Mut<Assets<Shader>>| {
            let mut graph = world.get_resource_mut::<RenderGraph>().unwrap();
            pipelines.set_untracked(
                SQUARE_PIPELINE,
                create_square_pipeline(&mut shaders)
            );

            pipelines.set_untracked(
                DIAMOND_ISO_PIPELINE,
                create_iso_diamond_pipeline(&mut shaders)
            );

            pipelines.set_untracked(
                STAGGERED_ISO_PIPELINE,
                create_iso_staggered_pipeline(&mut shaders)
            );

            pipelines.set_untracked(
                ROW_HEX_PIPELINE,
                create_hex_row_pipeline(&mut shaders)
            );

            pipelines.set_untracked(
                ROW_ODD_HEX_PIPELINE,
                create_hex_row_odd_pipeline(&mut shaders),
            );

            pipelines.set_untracked(
                ROW_EVEN_HEX_PIPELINE,
                create_hex_row_even_pipeline(&mut shaders),
            );

            pipelines.set_untracked(
                COLUMN_HEX_PIPELINE,
                create_hex_column_pipeline(&mut shaders),
            );

            pipelines.set_untracked(
                COLUMN_ODD_HEX_PIPELINE,
                create_hex_column_odd_pipeline(&mut shaders),
            );

            pipelines.set_untracked(
                COLUMN_EVEN_HEX_PIPELINE,
                create_hex_column_even_pipeline(&mut shaders),
            );

            graph.add_system_node(
                node::TILEMAP_DATA,
                RenderResourcesNode::<TilemapData>::new(true),
            );

            graph
                .add_node_edge(node::TILEMAP_DATA, base::node::MAIN_PASS)
                .unwrap();
        });
    });
}
