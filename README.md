# bevy_ecs_tilemap
A tilemap rendering plugin for bevy which is more ECS friendly by having an entity per tile.

## Features
 - A tile per entity
 - Fast rendering using a chunked approach.
 - Layers and sparse tile maps.
 - GPU powered animations
 - Isometric and Hexagonal tile maps

## Upcoming Features
 - ~~Support for isometric and hexagon rendering~~ done
 - ~~Built in animation support.~~ done see animation example
 - Texture array support
 - ~~Layers and add/remove tiles. (High Priority)~~ done


## Screenshots
![iso](screenshots/iso.png)
![hex](screenshots/hex.png)

### How this works?
Quite simple there is a tile per entity. Behind the scenes the tiles are split into chunks that each have their own mesh which is sent to the GPU in an optimal way.

### Why use this over another bevy tile map plugin?
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

## Examples
 - accessing_tiles - An example showing how one can access tiles from the map object by using tile map coordinates.
 - animation - Basic cpu animation example.
 - bench - A stress test of the map rendering system. Takes a while to load.
 - dynamic_map - A random map that is only partial filled with tiles that changes every so often.
 - game_of_life - A game of life simulator.
 - hex_column - A map that is meshed using "pointy" hexagons.
 - hex_row - A map that is meshed using flat hexagons.
 - iso - An isometric meshed map.
 - layers - An example of how you can use multiple map entities/components for "layers".
 - map - The simplest example of how to create a tile map.
 - random_map - A bench of editing all of the tiles every 100 ms.
 - remove_tiles - An example showing how you can remove tiles by using map.remove_tile or by applying the RemoveTile tag component.
 - sparse_tiles - An example showing how to generate a map where not all of the tiles exist for a given square in the tile map.
 - visibility - An example showcasing visibility of tiles and chunks.

### Running examples
`cargo run --release --example map`


## Known Issues
 - Hex tile maps ignore proper chunk hex based positioning.

## Asset credits
 - Field of green by GuttyKreum(https://guttykreum.itch.io/)