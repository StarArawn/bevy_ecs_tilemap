use bevy::prelude::Vec2;
use bevy_ecs_tilemap::anchor::TilemapAnchor;

#[allow(dead_code)]
pub fn rotate_right(anchor: &TilemapAnchor) -> TilemapAnchor {
    use TilemapAnchor::*;
    match anchor {
        TopLeft => TopCenter,
        TopCenter => TopRight,
        TopRight => CenterRight,
        CenterRight => BottomRight,
        BottomRight => BottomCenter,
        BottomCenter => BottomLeft,
        BottomLeft => CenterLeft,
        CenterLeft => Center,
        Center => Custom(Vec2::splat(0.25)),
        Custom(_) => None,
        None => TopLeft,
    }
}
