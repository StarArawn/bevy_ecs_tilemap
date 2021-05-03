use bevy::prelude::*;

/// TODO: DOCS
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub chunk: Entity,
    pub texture_index: u32,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            chunk: Entity::new(0),
            texture_index: 0,
            flip_x: false,
            flip_y: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Visible;
