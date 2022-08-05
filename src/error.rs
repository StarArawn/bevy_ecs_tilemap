use crate::TilePos;

/// General errors that are returned by bevy_ecs_tilemap.
#[derive(Debug, Copy, Clone)]
pub enum MapTileError {
    /// The tile was out of bounds.
    OutOfBounds(TilePos),
    /// The tile already exists.
    AlreadyExists(TilePos),
    /// Doesn't exist
    NonExistent(TilePos),
}

impl std::error::Error for MapTileError {}

impl std::fmt::Display for MapTileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MapTileError::OutOfBounds(pos) => write!(f, "Tile out of bounds (@ {:?})", pos),
            MapTileError::AlreadyExists(pos) => write!(f, "Tile already exists (@ {:?})", pos),
            MapTileError::NonExistent(pos) => write!(f, "Tile does not exist (@ {:?})", pos),
        }
    }
}
