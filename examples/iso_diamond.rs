use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;
mod helpers;

// This example demonstrates a tilemap laid out isometrically using the "Diamond" coordinate system.

// Side length of a colored quadrant (in "number of tiles").
const QUADRANT_SIDE_LENGTH: u32 = 80;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("iso_color.png");

    // In total, there will be `(QUADRANT_SIDE_LENGTH * 2) * (QUADRANT_SIDE_LENGTH * 2)` tiles.
    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };
    let quadrant_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTextureIndex(0),
        TilePos { x: 0, y: 0 },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(1),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: 0,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(2),
        TilePos {
            x: 0,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(3),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 64.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

/// handle_tile_click is a function designed for responding to the user clicking on a tile-map entity in a Bevy engine game.
///
/// When the user clicks with the left or right mouse button, the system will calculate the tile position from the cursor position, accounting for
/// the camera/perspective scale and transforms, and the map's location in the world space.
///
/// When the left or right mouse button has been just pressed, it increments or decrements, respectively, the current tile texture index for the tile.

/// If the cursor's position does not exist within the window, no action is taken.
///
/// # Errors
/// This function may panic if there is not exactly one camera entity or one primary window.
fn handle_tile_click(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_cameras: Query<(&Camera, &Transform)>,
    mut mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut q_tile_texture: Query<&mut TileTextureIndex>,
    mut q_tilemap: Query<(&mut TileStorage, &mut TilemapSize, &mut TilemapGridSize, &mut TilemapType, &Transform)>) {

    let mut delta = 0;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        delta += 1
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        delta -= 1;
    }

    let (camera, camera_transform) = q_cameras.get_single().expect("Expected a single camera entity");
    let window = q_windows.get_single().expect("Expected a single window");
    match window.cursor_position() {
        Some(position) => {
            // Window size in pixels
            let win_size = Vec2::new(window.width(), window.height());

            // The cursor's position in Normalized Device Coordinates, ranging from (-1, -1) bottom left to (1, 1) top right.
            let ndc_pos = ((position / win_size) * 2.0 - Vec2::new(1.0, 1.0) ) * Vec2::new(1.0, -1.0);

            // Create a point in NDC with a depth of 0 (this is arbitrary and should be adjusted if your game isn't in a flat 2D plane).
            let point_ndc = Vec3::new(ndc_pos.x, ndc_pos.y, 0.0);

            //Apply the camera's projection matrix to the NDC
            let camera_matrix_inverse = camera.projection_matrix().inverse();
            let point_world = camera_matrix_inverse.transform_point3(point_ndc);

            // Apply the camera's transformation.
            let world_space_position = camera_transform.compute_matrix() * point_world.extend(1f32);

            for (mut tile_storage, map_size, grid_size, map_type, map_transform) in q_tilemap.iter_mut() {
                //convert world space to map space
                let map_position = map_transform.compute_matrix().inverse() * world_space_position;

                //use the map space point to file the tile
                let tile_pos = TilePos::from_world_pos(&map_position.xy(), &map_size, &grid_size, &map_type).unwrap();
                let tile_entity = tile_storage.get(&tile_pos).unwrap();
                let tti = q_tile_texture.get_mut(tile_entity).unwrap();
                let new_index = if tti.0 == 6 { 0 } else { tti.0 + delta };

                commands.entity(tile_entity).insert(TileTextureIndex(new_index));
            }

        },
        None => {
            //Cursor is not inside window
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Iso Diamond Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, handle_tile_click)
        .run();
}
