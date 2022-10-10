# `bevy_ecs_tilemap`

[![Crates.io](https://img.shields.io/crates/v/bevy_ecs_tilemap)](https://crates.io/crates/bevy_ecs_tilemap)
[![docs](https://docs.rs/bevy_ecs_tilemap/badge.svg)](https://docs.rs/bevy_ecs_tilemap/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_ecs_tilemap)](https://crates.io/crates/bevy_ecs_tilemap)

A tilemap rendering plugin for [`bevy`](https://bevyengine.org/). It is more ECS friendly as it makes tiles entities.

## Features
 - A tile per entity.
 - Fast rendering using a chunked approach.
 - Layers and sparse tile maps.
 - GPU powered animations.
 - Isometric and Hexagonal tile maps.
 - Initial support for Tiled file exports.
 - World position <-> tile position helpers.
 - Support for texture arrays or texture atlases.
 - Support for import of texture containers (KTX2/DDS).

## Screenshots
![iso](screenshots/iso.png)
![hex](screenshots/hex.png)

### How Does This Work?
Quite simple there is a tile per entity. Behind the scenes the tiles are split into chunks that each have their own mesh which is sent to the GPU in an optimal way.

### Why Use This Instead of X?
Because each tile is an entity of its own editing tiles is super easy and convenient. This allows you to tag entities for updating and makes stuff like animation easier. Want to have a mining simulation where damage is applied to tiles? That’s easy with this plugin:

```rust
struct Damage {
    amount: u32,
}

fn update_damage(
    mut query: Query<(&mut TileTexture, &Damage), Changed<Damage>>,
) {
    for (mut tile_texture, damage) in query.iter_mut() {
        tile_texture.0 = TILE_DAMAGE_OFFSET + damage.amount;
    }
}
```

## Examples
 - [`accessing_tiles`](examples/accessing_tiles.rs) – An example showing how one can access tiles from the map object by using tile map coordinates.
 - [`animation`](examples/animation.rs) – Basic CPU animation example.
 - [`basic`](examples/basic.rs) - The simplest example of how to create a tile map.
 - [`bench`](examples/bench.rs) - A stress test of the map rendering system. Takes a while to load.
 - [`chunking`](examples/chunking.rs) - A simple example showing how to implement an infinite tilemap by spawning multiple chunks.
 - [`frustum_cull_test`](examples/frustum_cull_test.rs) - An example demonstrating culling of chunks that are outside of the view frustum. 
 - [`colors`](examples/colors.rs) - Showcases how each tile can have an individual color.
 - [`game_of_life`](examples/game_of_life.rs) - A game of life simulator.
 - [`hexagon_column`](examples/hexagon_column.rs) - A map that is using “pointy”-topped hexagons.
 - [`hexagon_generation`](examples/hexagon_column.rs) - Demonstrates how to generate hexagon-shaped hexagon maps.
 - [`hexagon_row`](examples/hexagon_row.rs) - A map that is using "flat"-topped hexagons.
 - [`iso_diamond`](examples/iso_diamond.rs) - An isometric map using diamond ordering.
 - [`iso_staggered`](examples/iso_staggered.rs) - An isometric map using staggered ordering.
 - [`layers`](examples/layers.rs) - An example of how you can use multiple map entities/components for “layers”.
 - [`mouse_to_tile`](examples/mouse_to_tile.rs) - Demonstrates how to convert mouse position to tile position. 
 - [`move_tile`](examples/move_tile.rs) - An example showing how to change tile positions.
 - [`neighbors`](examples/neighbors.rs) - Demonstrates how to get the neighbors of a tile, for differen tile map types.
 - [`ldtk`](examples/ldtk/ldtk.rs) - An example of loading and rendering of a LDTK map. Use: `cargo run --example ldtk`. We recommend checking out: [`bevy_ecs_ldtk`](https://crates.io/crates/bevy_ecs_ldtk).
 - [`random_map`](examples/random_map.rs) - A bench of editing all of the tiles every 100 ms.
 - [`remove_tiles`](examples/remove_tiles.rs) - An example showing how you can remove tiles by using map_query
 - [`texture_container`](examples/texture_container.rs) - An example showing to load tile textures from KTX2 files.
 - [`texture_vec`](examples/texture_container.rs) - An example showing how to provide tile textures as a vector of bevy `Image`s.
 - [`tiled_rotate`](examples/tiled_rotate.rs) - An example of loading and rendering of a tiled map editor map with flipping and rotation. 
 - [`tiled`](examples/tiled.rs) - An example of loading and rendering of a tiled map editor map. 
 - [`visibility`](examples/visibility.rs) - An example showcasing visibility of tiles and chunks.

### Running Examples

```
cargo run --release --example basic
```

### Running examples on web!
```
cargo serve --example animation --release --features atlas
```

## FAQ

* **Can I have differently sized tiles in the same tilemap?**

No. The reason why we do not allow tiles of different sizes and z-indices within the same tilemap is in order to improve rendering performance. A `TilemapBundle` can effectively be thought of as a "batch" of data for the GPU: all items in one batch have the same properties.

A `TilemapBundle` should not be confused for a logical separation between entities, even if sometimes, it ends up working out that way in simple cases. A Tilemap shoud be thought of as a rendering abstraction. Therefore, do not use TilemapTileSize to check for example, collision between objects. 

In other words, your logical separation between entities will probably have to be maintained using standard bevy techniques: i.e. using marker components and so on.

* Is there a branch that tracks `bevy`'s `main`?

Yes, the `bevy-track` branch.

## Known Issues
 - Tile flipping by x, y and d, should work for all maps, however "d" (anti diagonal) flipping is not implemented for non-square maps.
 - https://github.com/StarArawn/bevy_ecs_tilemap/issues/308

## Bevy Compatibility

|bevy|bevy_ecs_tilemap|
|---|---|
|0.8|0.8|
|0.8|0.7|
|0.7|0.6|
|0.6|0.5|

## Asset credits
 - Field of green by [GuttyKreum](https://guttykreum.itch.io/).
