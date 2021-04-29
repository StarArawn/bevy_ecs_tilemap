# bevy_ecs_tilemap
A tilemap rendering plugin for bevy which is more ECS friendly by having an entity per tile.

## Features
 - A tile per entity
 - Fast rendering using a chunked approach.


## Upcoming Features
 - Support for isometric and hexagon rendering?
 - Built in animation support.


### How this works?
Quite simple there is a tile per entity. Behind the scenes the tiles are split into chunks that each have their own mesh which is sent to the GPU in an optimal way.

### Why use this over another bevy tilemap plugin?
Because each tile is an entity of its own editing tiles is super easy, and convenient. This allows you tag entities for updating and makes stuff like animation easier. Want to have a mining simulation where damage is applied to tiles? That's easy with this plugin:

```rust
struct Damage {
    amount: u32,
}

fn update_damage(
    mut query: Query<(&mut Tile, &Damage), Changed<Damage>>,
) {
    for (mut tile, damage) in query.iter_mut() {
        tile.texture_index = TILE_DAMAGE_OFFSET + damage.amount;
    }
}
```