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
    grid_size: vec2<f32>,
    spacing: vec2<f32>,
    chunk_pos: vec2<f32>,
    map_size: vec2<f32>,
    time: f32,
    _padding: vec3<f32>, // hack for webgl2 16 byte alignment
};
@group(1) @binding(1)
var<uniform> tilemap_data: TilemapData;

struct VertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) uv: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) color: vec4<f32>,
}

#ifdef ATLAS
@group(2) @binding(0)
var sprite_texture: texture_2d<f32>;
#else
@group(2) @binding(0)
var sprite_texture: texture_2d_array<f32>;
#endif

@group(2) @binding(1)
var sprite_sampler: sampler;

#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

fn process_fragment(in: MeshVertexOutput) -> vec4<f32> {
    #ifdef ATLAS
    let half_texture_pixel_size_u = 0.5 / tilemap_data.texture_size.x;
    let half_texture_pixel_size_v = 0.5 / tilemap_data.texture_size.y;
    let half_tile_pixel_size_u = 0.5 / tilemap_data.tile_size.x;
    let half_tile_pixel_size_v = 0.5 / tilemap_data.tile_size.y;

    // Offset the UV 1/2 pixel from the sides of the tile, so that the sampler doesn't bleed onto
    // adjacent tiles at the edges.
    var uv_offset: vec2<f32> = vec2<f32>(0.0, 0.0);
    if (in.uv.z < half_tile_pixel_size_u) {
        uv_offset.x = half_texture_pixel_size_u;
    } else if (in.uv.z > (1.0 - half_tile_pixel_size_u)) {
        uv_offset.x = - half_texture_pixel_size_u;
    }
    if (in.uv.w < half_tile_pixel_size_v) {
        uv_offset.y = half_texture_pixel_size_v;
    } else if (in.uv.w > (1.0 - half_tile_pixel_size_v)) {
        uv_offset.y = - half_texture_pixel_size_v;
    }

    let color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset) * in.color;
    if (color.a < 0.001) {
        discard;
    }
    return color;
    #else
    let color = textureSample(sprite_texture, sprite_sampler, in.uv.xy, in.tile_id) * in.color;
    if (color.a < 0.001) {
        discard;
    }
    return color;
    #endif
}