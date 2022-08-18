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
    var world_pos = mesh.model * vec4<f32>(vertex_position.xy, 0.0, 1.0);
    var position = vertex_position.xy * tilemap_data.grid_size;
    var world_translation = mesh.model * vec4<f32>(0.0, 0.0, 0.0, 1.0);

    var offset = floor(0.25 * tilemap_data.grid_size.x);
    if (u32(world_pos.y) % 2u == 0u) {
        position.x = position.x + offset;
    } else {
        position.x = position.x - offset;
    }
    position.y = position.y - (world_pos.y * (tilemap_data.grid_size.y / 2.0));
    position.x = position.x + world_translation.x;

    var positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(position.x, position.y),
        vec2<f32>(position.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y)
    );
    position = positions[v_index % 4u];

    out.world_position = vec4<f32>(position.xy, world_pos.zw);

    return out;
}