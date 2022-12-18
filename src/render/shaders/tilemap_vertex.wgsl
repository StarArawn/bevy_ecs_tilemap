#import bevy_ecs_tilemap::common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    #import bevy_ecs_tilemap::vertex_output
}

#ifdef SQUARE
    #import bevy_ecs_tilemap::square
#endif

#ifdef ISO_DIAMOND
    #import bevy_ecs_tilemap::diamond_iso
#endif

#ifdef ISO_STAGGERED
    #import bevy_ecs_tilemap::staggered_iso
#endif

#ifdef COLUMN_EVEN_HEX
    #import bevy_ecs_tilemap::column_even_hex
#endif

#ifdef COLUMN_HEX
    #import bevy_ecs_tilemap::column_hex
#endif

#ifdef COLUMN_ODD_HEX
    #import bevy_ecs_tilemap::column_odd_hex
#endif

#ifdef ROW_EVEN_HEX
    #import bevy_ecs_tilemap::row_even_hex
#endif

#ifdef ROW_HEX
    #import bevy_ecs_tilemap::row_hex
#endif

#ifdef ROW_ODD_HEX
    #import bevy_ecs_tilemap::row_odd_hex
#endif


@vertex
fn vertex(vertex_input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var animation_speed = vertex_input.position.z;

    var mesh_data: MeshOutput = get_mesh(vertex_input.v_index, vec3(vertex_input.position.xy, 0.0));

    var frames: f32 = f32(vertex_input.uv.w - vertex_input.uv.z);

    var current_animation_frame = fract(tilemap_data.time * animation_speed) * frames;

    current_animation_frame = clamp(f32(vertex_input.uv.z) + current_animation_frame, f32(vertex_input.uv.z), f32(vertex_input.uv.w));

    var texture_index: u32 = u32(current_animation_frame);

    #ifdef ATLAS
    // Get the top-left corner of the current frame in the texture, accounting for padding around the whole texture
    // as well as spacing between the tiles.
    var columns: u32 = u32(round((tilemap_data.texture_size.x - tilemap_data.spacing.x) / (tilemap_data.tile_size.x + tilemap_data.spacing.x)));
    var sprite_sheet_x: f32 = tilemap_data.spacing.x + floor(f32(texture_index % columns)) * (tilemap_data.tile_size.x + tilemap_data.spacing.x);
    var sprite_sheet_y: f32 = tilemap_data.spacing.y + floor(f32(texture_index / columns)) * (tilemap_data.tile_size.y + tilemap_data.spacing.y);

    var start_u: f32 = sprite_sheet_x / tilemap_data.texture_size.x;
    var end_u: f32 = (sprite_sheet_x + tilemap_data.tile_size.x) / tilemap_data.texture_size.x;
    var start_v: f32 = sprite_sheet_y / tilemap_data.texture_size.y;
    var end_v: f32 = (sprite_sheet_y + tilemap_data.tile_size.y) / tilemap_data.texture_size.y;
    #else
    var start_u: f32 = 0.0;
    var end_u: f32 = 1.0;
    var start_v: f32 = 0.0;
    var end_v: f32 = 1.0;
    #endif

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
    out.position = view.view_proj * mesh_data.world_position;
    out.color = vertex_input.color;
    return out;
}
