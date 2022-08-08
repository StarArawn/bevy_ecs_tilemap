struct Output {
    world_position: vec4<f32>,
    uv: vec2<f32>,
};

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;
    var position = vertex_position.xy;

    var positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(position.x, position.y),
        vec2<f32>(position.x, position.y + 1.0),
        vec2<f32>(position.x + 1.0, position.y + 1.0),
        vec2<f32>(position.x + 1.0, position.y)
    );

    position = positions[v_index % 4u];

    var position = position * tilemap_data.tile_size;
    out.world_position = mesh.model * vec4<f32>(position, 0.0, 1.0);

    return out;
}