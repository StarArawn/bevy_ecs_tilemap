struct Output {
    world_position: vec4<f32>,
    uv: vec2<f32>,
};

fn project_hex_col_1(xy: vec2<f32>, hex_radius: f32) -> vec2<f32> {
    let b0 = vec2<f32>(hex_radius * 1.7320508, 0.0);
    let b1 = vec2<f32>(hex_radius * 0.8660254, hex_radius * 1.5);

    return xy.x * b0 + xy.y * b1;
}

fn project_hex_col(xy: vec2<f32>, grid_size: vec2<f32>) -> vec2<f32> {
    let b0 = vec2<f32>(0.0, grid_size.y);
    let b1 = vec2<f32>(0.75 * grid_size.x, 0.5 * grid_size.y);

    return xy.x * b0 + xy.y * b1;
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;

    var position = project_hex_col(vertex_position.xy, tilemap_data.grid_size.xy);

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