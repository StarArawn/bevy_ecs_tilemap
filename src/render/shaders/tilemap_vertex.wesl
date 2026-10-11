import package::render::shaders::common::{VertexInput, tilemap_data};
import package::render::shaders::mesh_output::MeshOutput;
import package::render::shaders::tilemap_vertex_output::MeshVertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::{view, globals};

@if(SQUARE)
import package::render::shaders::square::get_mesh;
@if(ISO_DIAMOND)
import package::render::shaders::diamond_iso::get_mesh;
@if(ISO_STAGGERED)
import package::render::shaders::staggered_iso::get_mesh;
@if(COLUMN_EVEN_HEX)
import package::render::shaders::column_even_hex::get_mesh;
@if(COLUMN_HEX)
import package::render::shaders::column_hex::get_mesh;
@if(COLUMN_ODD_HEX)
import package::render::shaders::column_odd_hex::get_mesh;
@if(ROW_EVEN_HEX)
import package::render::shaders::row_even_hex::get_mesh;
@if(ROW_HEX)
import package::render::shaders::row_hex::get_mesh;
@if(ROW_ODD_HEX)
import package::render::shaders::row_odd_hex::get_mesh;

// UV bounds (start_u, end_u, start_v, end_v) of the given frame in the texture.
@if(ATLAS)
fn frame_uv_bounds(texture_index: u32) -> vec4<f32> {
    // Get the top-left corner of the current frame in the texture, accounting for padding around the whole texture
    // as well as spacing between the tiles.
    let columns: u32 = u32(round((tilemap_data.texture_size.x - tilemap_data.spacing.x) / (tilemap_data.tile_size.x + tilemap_data.spacing.x)));
    let sprite_sheet_x: f32 = tilemap_data.spacing.x + floor(f32(texture_index % columns)) * (tilemap_data.tile_size.x + tilemap_data.spacing.x);
    let sprite_sheet_y: f32 = tilemap_data.spacing.y + floor(f32(texture_index / columns)) * (tilemap_data.tile_size.y + tilemap_data.spacing.y);

    let start_u: f32 = sprite_sheet_x / tilemap_data.texture_size.x;
    let end_u: f32 = (sprite_sheet_x + tilemap_data.tile_size.x) / tilemap_data.texture_size.x;
    let start_v: f32 = sprite_sheet_y / tilemap_data.texture_size.y;
    let end_v: f32 = (sprite_sheet_y + tilemap_data.tile_size.y) / tilemap_data.texture_size.y;
    return vec4<f32>(start_u, end_u, start_v, end_v);
}
@else
fn frame_uv_bounds(texture_index: u32) -> vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}

@vertex
fn vertex(vertex_input: VertexInput) -> MeshVertexOutput {
    var out: MeshVertexOutput;
    let animation_speed = vertex_input.position.z;

    let mesh_data: MeshOutput = get_mesh(vertex_input.v_index, vec3(vertex_input.position.xy, 0.0));

    let frames: f32 = f32(vertex_input.uv.w - vertex_input.uv.z);

    var current_animation_frame = fract(globals.time * animation_speed) * frames;

    current_animation_frame = clamp(f32(vertex_input.uv.z) + current_animation_frame, f32(vertex_input.uv.z), f32(vertex_input.uv.w));

    let texture_index: u32 = u32(current_animation_frame);

    let uv_bounds = frame_uv_bounds(texture_index);
    let start_u: f32 = uv_bounds.x;
    let end_u: f32 = uv_bounds.y;
    let start_v: f32 = uv_bounds.z;
    let end_v: f32 = uv_bounds.w;

    var atlas_uvs: array<vec4<f32>, 4>;

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

    atlas_uvs = array<vec4<f32>, 4>(
        x1[u32(vertex_input.uv.y)],
        x2[u32(vertex_input.uv.y)],
        x3[u32(vertex_input.uv.y)],
        x4[u32(vertex_input.uv.y)]
    );

    out.uv = atlas_uvs[vertex_input.v_index % 4u];
    out.tile_id = i32(texture_index);
    // out.uv = out.uv + 1e-5;
    out.position = view.clip_from_world * mesh_data.world_position;
    out.color = vertex_input.color;
    out.storage_position = vec2<u32>(vertex_input.position.xy);
    return out;
}
