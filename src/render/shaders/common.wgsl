#define_import_path bevy_ecs_tilemap::common

#import bevy_sprite::mesh2d_view_bindings

struct Mesh {
    model: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> mesh: Mesh;

struct TilemapData {
    texture_size: vec2<f32>,
    tile_size: vec2<f32>,
    physical_tile_size: vec2<f32>,
    grid_size: vec2<f32>,
    spacing: vec2<f32>,
    chunk_pos: vec2<f32>,
    map_size: vec2<f32>,
    time: f32,
    _padding: f32, // hack for webgl2 16 byte alignment
};
@group(2) @binding(0)
var<uniform> tilemap_data: TilemapData;

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) uv: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) color: vec4<f32>,
}

#ifdef ATLAS
@group(3) @binding(0)
var sprite_texture: texture_2d<f32>;
#else
@group(3) @binding(0)
var sprite_texture: texture_2d_array<f32>;
#endif

@group(3) @binding(1)
var sprite_sampler: sampler;
