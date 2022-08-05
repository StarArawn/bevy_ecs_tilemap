struct Output {
    world_position: vec4<f32>,
};

fn project_iso(pos: vec2<f32>, tile_width: f32, tile_height: f32) -> vec2<f32> {
    var x = (pos.x - pos.y) * tile_width / 2.0;
    var y = (pos.x + pos.y) * tile_height / 2.0;
    return vec2<f32>(x, -y);
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;
    var position = vertex_position.xy;

    var center = project_iso(vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
    var z_center = project_iso(tilemap_data.chunk_pos + vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);

    var start = vec2<f32>(
        center.x - tilemap_data.tile_size.x / 2.0,
        center.y - tilemap_data.tile_size.y
    );
    var end = vec2<f32>(center.x + tilemap_data.tile_size.x / 2.0, center.y);

    var positions = array<vec2<f32>, 4>(
        vec2<f32>(start.x, start.y),
        vec2<f32>(start.x, end.y),
        vec2<f32>(end.x, end.y),
        vec2<f32>(end.x, start.y)
    );

    out.world_position = mesh.model * vec4<f32>(vec3<f32>(positions[v_index % 4u], 1.0 - (z_center.y / tilemap_data.map_size.y)), 1.0);

    return out;
}