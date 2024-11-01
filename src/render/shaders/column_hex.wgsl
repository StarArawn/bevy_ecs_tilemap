#define_import_path bevy_ecs_tilemap::column_hex

#import bevy_ecs_tilemap::common::{VertexInput, tilemap_data, mesh}
#import bevy_ecs_tilemap::mesh_output::MeshOutput

// Gets the screen space coordinates of the bottom left of an isometric tile position.
fn hex_col_tile_pos_to_world_pos(pos: vec2<f32>, grid_width: f32, grid_height: f32) -> vec2<f32> {
    let SQRT_3: f32 = 1.7320508;
    let HALF_SQRT_3: f32 = 0.8660254;
    let COL_BASIS_X: vec2<f32> = vec2<f32>(HALF_SQRT_3, 0.5);
    let COL_BASIS_Y: vec2<f32> = vec2<f32>(0.0, 1.0);

    let unscaled_pos = pos.x * COL_BASIS_X + pos.y * COL_BASIS_Y;
    return vec2<f32>(COL_BASIS_X.x * grid_width * unscaled_pos.x, grid_height * unscaled_pos.y);
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> MeshOutput {
    var out: MeshOutput;

    let center = hex_col_tile_pos_to_world_pos(vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
    let bot_left = center - 0.5 * tilemap_data.in_world_tile_size;
    let top_right = bot_left + tilemap_data.in_world_tile_size;

    var positions = array<vec2<f32>, 4>(
        bot_left,
        vec2<f32>(bot_left.x, top_right.y),
        top_right,
        vec2<f32>(top_right.x, bot_left.y)
    );

    out.world_position = mesh.model * vec4<f32>(positions[v_index % 4u], 0.0, 1.0);

    return out;
}
