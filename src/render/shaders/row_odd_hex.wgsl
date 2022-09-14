struct Output {
    world_position: vec4<f32>,
    uv: vec2<f32>,
};


let SQRT_3: f32 = 1.7320508;
let HALF_SQRT_3: f32 = 0.8660254;
let ROW_BASIS_X: vec2<f32> = vec2<f32>(1.0, 0.0);
let ROW_BASIS_Y: vec2<f32> = vec2<f32>(0.5, HALF_SQRT_3);

// Gets the screen space coordinates of the bottom left of an isometric tile position.
fn hex_row_tile_pos_to_world_pos(pos: vec2<f32>, grid_width: f32, grid_height: f32) -> vec2<f32> {
    let unscaled_pos = pos.x * ROW_BASIS_X + pos.y * ROW_BASIS_Y;
    return vec2<f32>(grid_width * unscaled_pos.x, grid_height * unscaled_pos.y);
}

fn row_odd_to_axial(offset_pos: vec2<f32>) -> vec2<f32> {
    let delta: f32 = floor(offset_pos.y / 2.0);
    return vec2<f32>(offset_pos.x - delta, offset_pos.y);
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> Output {
    var out: Output;

    var axial_pos = row_odd_to_axial(vertex_position.xy);
    var center = hex_row_tile_pos_to_world_pos(axial_pos, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
    var bot_left = vec2<f32>(center.x - (tilemap_data.grid_size.x / 2.0), center.y - (tilemap_data.grid_size.y / 2.0));
    var top_right = vec2<f32>(bot_left.x + tilemap_data.tile_size.x, bot_left.y + tilemap_data.tile_size.y);

    var positions = array<vec2<f32>, 4>(
        bot_left,
        vec2<f32>(bot_left.x, top_right.y),
        top_right,
        vec2<f32>(top_right.x, bot_left.y)
    );

    out.world_position = mesh.model * vec4<f32>(positions[v_index % 4u], 0.0, 1.0);

    return out;
}
