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

    var columns: u32 = u32((tilemap_data.texture_size.x + tilemap_data.spacing.x) / (tilemap_data.tile_size.x + tilemap_data.spacing.x));

    var sprite_sheet_x: f32 = floor(f32(texture_index % columns)) * (tilemap_data.tile_size.x + tilemap_data.spacing.x);
    var sprite_sheet_y: f32 = floor(f32(texture_index / columns)) * (tilemap_data.tile_size.y + tilemap_data.spacing.y);

    #ifdef ATLAS
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
        x1[u32(vertex_input.uv.y)],
        x2[u32(vertex_input.uv.y)],
        x3[u32(vertex_input.uv.y)],
        x4[u32(vertex_input.uv.y)]
    );

    out.uv = vec3<f32>(atlas_uvs[vertex_input.v_index % 4u], f32(texture_index));
    // out.uv = out.uv + 1e-5;
    out.position = view.view_proj * mesh_data.world_position;
    out.color = vertex_input.color;
    return out;
}