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
    @location(0) uv: vec3<f32>,
    @location(1) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
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

    var start_u: f32 = 0.0; //sprite_sheet_x / tilemap_data.texture_size.x;
    var end_u: f32 = 1.0; //(sprite_sheet_x + tilemap_data.tile_size.x) / tilemap_data.texture_size.x;
    var start_v: f32 = 0.0; //sprite_sheet_y / tilemap_data.texture_size.y;
    var end_v: f32 = 1.0; //(sprite_sheet_y + tilemap_data.tile_size.y) / tilemap_data.texture_size.y;

    var atlas_uvs: array<vec2<f32>, 4>;

    var x1: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
        vec2<f32>(start_u, end_v),       // no flip/rotation
        vec2<f32>(end_u, end_v),         // flip x
        vec2<f32>(start_u, start_v),     // flip y
        vec2<f32>(end_u, start_v),       // flip x y
        vec2<f32>(end_u, start_v),       // flip     d
        vec2<f32>(end_u, end_v),         // flip x   d
        vec2<f32>(start_u, start_v),     // flip y   d
        vec2<f32>(start_u, end_v)
    );

    var x2: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
        vec2<f32>(start_u, start_v),
        vec2<f32>(end_u, start_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(end_u, end_v),
        vec2<f32>(start_u, start_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(end_u, start_v),
        vec2<f32>(end_u, end_v)
    );

    var x3: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
        vec2<f32>(end_u, start_v),
        vec2<f32>(start_u, start_v),
        vec2<f32>(end_u, end_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(start_u, start_v),
        vec2<f32>(end_u, end_v),
        vec2<f32>(end_u, start_v)
    );

    var x4: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
        vec2<f32>(end_u, end_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(end_u, start_v),
        vec2<f32>(start_u, start_v),
        vec2<f32>(end_u, end_v),
        vec2<f32>(end_u, start_v),
        vec2<f32>(start_u, end_v),
        vec2<f32>(start_u, start_v),
    );

    atlas_uvs = array<vec2<f32>, 4>(
        x1[u32(vertex_uv.y)],
        x2[u32(vertex_uv.y)],
        x3[u32(vertex_uv.y)],
        x4[u32(vertex_uv.y)]
    );

    out.uv = vec3<f32>(atlas_uvs[v_index % 4u], f32(texture_index));
    // out.uv = out.uv + 1e-5;
    out.position = view.view_proj * mesh_data.world_position;
    out.color = color;
    return out;
} 

@group(3) @binding(0)
var sprite_texture: texture_2d_array<f32>;
@group(3) @binding(1)
var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(sprite_texture, sprite_sampler, in.uv.xy, i32(in.uv.z)) * in.color;
    if (color.a < 0.001) {
        discard;
    }
    return color;
}