struct Output {
    world_position: vec4<f32>,
    uv: vec2<f32>,
};

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;

    var position = vertex_position.xy * tilemap_data.grid_size;
    var offset = floor(0.25 * tilemap_data.grid_size.y);
    if (u32(vertex_position.x) % 2u == 0u) {
        position.y = position.y - offset;
    } else {
        position.y = position.y + offset;
    }
    position.x = position.x - vertex_position.x * ceil(0.25 * tilemap_data.grid_size.x);

    var positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(position.x, position.y),
        vec2<f32>(position.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y)
    );
    position = positions[v_index % 4u];

    out.world_position = mesh.model * vec4<f32>(position, 0.0, 1.0);

    return out;
}