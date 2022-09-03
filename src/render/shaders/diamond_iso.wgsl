struct Output {
    world_position: vec4<f32>,
};

fn project_iso(pos: vec2<f32>, tile_width: f32, tile_height: f32) -> vec2<f32> {
    var dx = tile_width / 2.0;
    var dy = tile_height / 2.0;

    // ux/uy is the effect of moving one tile (ux: in the x direction, uy: in the y direction) on our position
    var ux = vec2<f32>(dx, -dy);
    var uy = vec2<f32>(dx, dy);
    
    return pos.x * ux + pos.y * uy;
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;

    var bot_left = project_iso(vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
    var tile_z = project_iso(tilemap_data.chunk_pos + vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
    var top_right = vec2<f32>(bot_left.x + tilemap_data.tile_size.x, bot_left.y + tilemap_data.tile_size.y);

    var positions = array<vec2<f32>, 4>(
        bot_left,
        vec2<f32>(bot_left.x, top_right.y),
        top_right,
        vec2<f32>(top_right.x, bot_left.y)
    );

    out.world_position = mesh.model * vec4<f32>(vec3<f32>(positions[v_index % 4u], 1.0 - (tile_z.y / tilemap_data.map_size.y)), 1.0);

    return out;
}