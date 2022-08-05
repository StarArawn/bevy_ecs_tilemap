struct View {
    view_proj: mat4x4<f32>,
    projection: mat4x4<f32>,
    world_position: vec3<f32>,
};
@group(0) @binding(0)
var<uniform> view: View;

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
@group(2) @binding(0)
var<uniform> tilemap_data: TilemapData;

#include 0

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex(
    @builtin(vertex_index) v_index: u32,
    @location(0) vertex_uv: vec4<f32>,
    @location(1) vertex_position: vec4<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    var animation_speed = vertex_position.z;

    var mesh_data: Output = get_mesh(v_index, vec3(vertex_position.xy, 0.0));

    var frames: f32 = f32(vertex_uv.w - vertex_uv.z);

    var current_animation_frame = fract(tilemap_data.time * animation_speed) * frames;

    current_animation_frame = clamp(f32(vertex_uv.z) + current_animation_frame, f32(vertex_uv.z), f32(vertex_uv.w));

    var texture_index: u32 = u32(current_animation_frame);

    var columns: u32 = u32((tilemap_data.texture_size.x + tilemap_data.spacing.x) / (tilemap_data.tile_size.x + tilemap_data.spacing.x));

    var sprite_sheet_x: f32 = floor(f32(texture_index % columns)) * (tilemap_data.tile_size.x + tilemap_data.spacing.x);
    var sprite_sheet_y: f32 = floor(f32(texture_index / columns)) * (tilemap_data.tile_size.y + tilemap_data.spacing.y);

    var start_u: f32 = sprite_sheet_x / tilemap_data.texture_size.x;
    var end_u: f32 = (sprite_sheet_x + tilemap_data.tile_size.x) / tilemap_data.texture_size.x;
    var start_v: f32 = sprite_sheet_y / tilemap_data.texture_size.y;
    var end_v: f32 = (sprite_sheet_y + tilemap_data.tile_size.y) / tilemap_data.texture_size.y;

    var uvs: array<vec4<f32>, 4>;

    var x1: array<vec4<f32>, 8> = array<vec4<f32>, 8>(
        // The x and y are the texture UV, and the z and w and the local tile UV
        vec4<f32>(start_u, end_v, 0.0, 1.0),       // no flip/rotation
        vec4<f32>(end_u, end_v, 1.0, 1.0),         // flip x
        vec4<f32>(start_u, start_v, 0.0, 0.0),     // flip y
        vec4<f32>(end_u, start_v, 1.0, 0.0),       // flip x y
        vec4<f32>(end_u, start_v, 1.0, 0.0),       // flip     d
        vec4<f32>(end_u, end_v, 1.0, 1.0),         // flip x   d
        vec4<f32>(start_u, start_v, 0.0, 0.0),     // flip y   d
        vec4<f32>(start_u, end_v, 0.0, 1.0)
    );

    var x2: array<vec4<f32>, 8> = array<vec4<f32>, 8>(
        vec4<f32>(start_u, start_v, 0.0, 0.0),
        vec4<f32>(end_u, start_v, 1.0, 0.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(end_u, end_v, 1.0, 1.0),
        vec4<f32>(start_u, start_v, 0.0, 0.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(end_u, start_v, 1.0, 0.0),
        vec4<f32>(end_u, end_v, 1.0, 1.0)
    );

    var x3: array<vec4<f32>, 8> = array<vec4<f32>, 8>(
        vec4<f32>(end_u, start_v, 1.0, 0.0),
        vec4<f32>(start_u, start_v, 0.0, 0.0),
        vec4<f32>(end_u, end_v, 1.0, 1.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(start_u, start_v, 0.0, 0.0),
        vec4<f32>(end_u, end_v, 1.0, 1.0),
        vec4<f32>(end_u, start_v, 1.0, 0.0)
    );

    var x4: array<vec4<f32>, 8> = array<vec4<f32>, 8>(
        vec4<f32>(end_u, end_v, 1.0, 1.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(end_u, start_v, 1.0, 0.0),
        vec4<f32>(start_u, start_v, 0.0, 0.0),
        vec4<f32>(end_u, end_v, 1.0, 1.0),
        vec4<f32>(end_u, start_v, 1.0, 0.0),
        vec4<f32>(start_u, end_v, 0.0, 1.0),
        vec4<f32>(start_u, start_v, 0.0, 0.0),
    );

    uvs = array<vec4<f32>, 4>(
        x1[u32(vertex_uv.y)],
        x2[u32(vertex_uv.y)],
        x3[u32(vertex_uv.y)],
        x4[u32(vertex_uv.y)]
    );

    out.uv = uvs[v_index % 4u];
    out.position = view.view_proj * mesh_data.world_position;
    out.color = color;
    return out;
} 

@group(3) @binding(0)
var sprite_texture: texture_2d<f32>;

@group(3) @binding(1)
var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var half_texture_pixel_size_u = 0.5 / tilemap_data.texture_size.x;
    var half_texture_pixel_size_v = 0.5 / tilemap_data.texture_size.y;
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

    var color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset) * in.color;
    if (color.a < 0.001) {
        discard;
    }
    return color;
}