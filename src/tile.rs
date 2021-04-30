use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub chunk: Entity,
    pub texture_index: u32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            chunk: Entity::new(0),
            texture_index: 0,
        }
    }
}
