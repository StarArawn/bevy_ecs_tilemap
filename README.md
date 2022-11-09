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

## Upcoming Features
 - [x] Support for isometric and hexagon rendering.
 - [x] Built in animation support  – see [`animation` example](examples/animation.rs).
 - [x] Texture array support.


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
 - [`colors`](examples/colors.rs) - Showcases how each tile can have an individual color.
 - [`game_of_life`](examples/game_of_life.rs) - A game of life simulator.
 - [`hex_column`](examples/hexagon_column.rs) - A map that is meshed using “pointy” hexagons.
 - [`hex_row`](examples/hexagon_row.rs) - A map that is meshed using flat hexagons.
 - [`iso_diamond`](examples/iso_diamond.rs) - An isometric meshed map using diamond ordering.
 - [`iso_staggered`](examples/iso_staggered.rs) - An isometric meshed map using staggered ordering.
 - [`layers`](examples/layers.rs) - An example of how you can use multiple map entities/components for “layers”.
 - [`ldtk`](examples/ldtk/ldtk.rs) - An example of loading and rendering of a LDTK map. Use: `cargo run --example ldtk`. We recommend checking out: [`bevy_ecs_ldtk`](https://crates.io/crates/bevy_ecs_ldtk).
 - [`random_map`](examples/random_map.rs) - A bench of editing all of the tiles every 100 ms.
 - [`remove_tiles`](examples/remove_tiles.rs) - An example showing how you can remove tiles by using map_query
 - [`tiled_rotate`](examples/tiled_rotate.rs) - An example of loading and rendering of a tiled map editor map with flipping and rotation. Use: `cargo run --example tiled_rotate`
 - [`tiled`](examples/tiled.rs) - An example of loading and rendering of a tiled map editor map. Use: `cargo run --example tiled`
 - [`visibility`](examples/visibility.rs) - An example showcasing visibility of tiles and chunks.

### Running Examples

```
cargo run --release --example basic
```

### Running examples on web!
```
cargo build --target wasm32-unknown-unknown --example animation --release --features atlas
wasm-server-runner .\target\wasm32-unknown-unknown\release\examples\animation.wasm
```

## Known Issues
 - Tile flipping by x, y and d, should work for all maps, however "d" (anti diagonal) flipping is not implemented for non-square maps.
 - Besides the above no known issues.

## Bevy Compatibility

|bevy|bevy_ecs_tilemap|
|---|---|
|`main`|`bevy-track`|
|0.8|0.8|
|0.8|0.7|
|0.7|0.6|
|0.6|0.5|

## Asset credits
 - Field of green by [GuttyKreum](https://guttykreum.itch.io/).
