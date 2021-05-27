use bevy::{prelude::*, render::camera::Camera};

// A simple camera system for moving and zooming the camera.
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let scale = transform.scale.x;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            let scale = scale + 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if keyboard_input.pressed(KeyCode::X) {
            let scale = scale - 0.1;
            transform.scale = Vec3::splat(scale);
        }

        if transform.scale.x < 1.0 {
            transform.scale = Vec3::splat(1.0)
        }

        transform.translation += time.delta_seconds() * direction * 500.;
    }
}
