use bevy::prelude::*;
use std::collections::HashMap;

/// A simple component used to keep track of layer entities.
#[derive(Clone)]
pub struct Map {
    pub map_entity: Entity,
    pub id: u16,
    layers: HashMap<u16, Entity>,
}

impl Map {
    /// Creates a new map component
    pub fn new<T: Into<u16>>(id: T, map_entity: Entity) -> Self {
        Self {
            map_entity,
            id: id.into(),
            layers: HashMap::new(),
        }
    }

    /// Creates a new layer.
    pub fn add_layer<T: Into<u16>>(
        &mut self,
        commands: &mut Commands,
        layer_id: T,
        layer_entity: Entity,
    ) {
        commands
            .entity(self.map_entity)
            .push_children(&[layer_entity]);
        self.layers.insert(layer_id.into(), layer_entity);
    }

    /// Retrieves the entity for a given layer id.
    pub fn get_layer_entity<T: Into<u16>>(&self, layer_id: T) -> Option<&Entity> {
        self.layers.get(&layer_id.into())
    }
}
