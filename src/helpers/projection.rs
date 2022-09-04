use bevy::math::Vec2;

/// Projects a 2D screen space point into isometric diamond space.
///
/// `grid_width` and `grid_height` are the dimensions of the grid in pixels.
pub fn project_column_odd(x: f32, y: f32, grid_width: f32, grid_height: f32) -> Vec2 {
    let dx = grid_width / 2.0;
    let dy = grid_height / 2.0;

    let new_x = (x + y) * dx;
    let new_y = (-x + y) * dy;
    Vec2::new(new_x, new_y)
}

/// Projects a 2D screen space point into isometric diamond space.
///
/// `grid_width` and `grid_height` are the dimensions of the grid in pixels.
pub fn project_iso_diamond(x: f32, y: f32, grid_width: f32, grid_height: f32) -> Vec2 {
    let dx = grid_width / 2.0;
    let dy = grid_height / 2.0;

    let new_x = (x + y) * dx;
    let new_y = (-x + y) * dy;
    Vec2::new(new_x, new_y)
}

/// Projects a 2D screen space point into isometric staggered space.
///
/// `grid_width` and `grid_height` are the dimensions of the grid in pixels.
pub fn project_iso_staggered(x: f32, y: f32, grid_width: f32, grid_height: f32) -> Vec2 {
    let dx = grid_width / 2.0;
    let dy = grid_height / 2.0;

    let new_x = x * grid_width + y * dx;
    let new_y = y * dy;
    Vec2::new(new_x, new_y)
}
