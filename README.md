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
- Examples for integration with [Tiled](https://www.mapeditor.org/) and [LDTK](https://ldtk.io/) editors.
- Can `Anchor` tilemap like a sprite.

## Screenshots

![iso](screenshots/iso.png)
![hex](screenshots/hex.png)

### How Does This Work?

Quite simply there is a tile per entity. Behind the scenes the tiles are split into chunks that each have their own mesh which is sent to the GPU in an optimal way.

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

- [`3d_iso`](examples/3d_iso.rs) - An example showing y-sorting on a "3d" isometric tile map.
- [`accessing_tiles`](examples/accessing_tiles.rs) – An example showing how one can access tiles from the map object by using tile map coordinates.
- [`animation`](examples/animation.rs) – Basic animation example.
- [`basic`](examples/basic.rs) - The simplest example of how to create a tile map.
- [`bench`](examples/bench.rs) - A stress test of the map rendering system. Takes a while to load.
- [`chunking`](examples/chunking.rs) - A simple example showing how to implement an infinite tilemap by spawning multiple chunks.
- [`colors`](examples/colors.rs) - Showcases how each tile can have an individual color.
- [`custom_shader`](examples/custom_shader.rs) - An example showing how to use a custom shader.
- [`frustum_cull_test`](examples/frustum_cull_test.rs) - An environment for testing frustum culling.
- [`game_of_life`](examples/game_of_life.rs) - A game of life simulator.
- [`hex_neighbors`](examples/hex_neighbors.rs) - An example showing how to get neighbors on hexagonal maps.
- [`hex_neighbors_radius_chunks`](examples/hex_neighbors_radius_chunks.rs) - An example showing how to get neighbors within a radius on hexagonal maps.
- [`hexagon_column`](examples/hexagon_column.rs) - A map that is meshed using flat top hexagons.
- [`hexagon_generation`](examples/hexagon_generation.rs) - Shows how to generate hexagonal maps.
- [`hexagon_row`](examples/hexagon_row.rs) - A map that is meshed using pointy top hexagons.
- [`iso_diamond`](examples/iso_diamond.rs) - An isometric meshed map using diamond ordering.
- [`iso_staggered`](examples/iso_staggered.rs) - An isometric meshed map using staggered ordering.
- [`layers`](examples/layers.rs) - An example of how you can use multiple map entities/components for “layers”.
- [`ldtk`](examples/ldtk.rs) - An example of loading and rendering of a [LDTK](https://ldtk.io/) map. We recommend checking out [`bevy_ecs_ldtk`](https://crates.io/crates/bevy_ecs_ldtk).
- [`mouse_to_tile`](examples/mouse_to_tile.rs) - Shows how to convert a mouse cursor position into a tile position.
- [`move_tile`](examples/move_tile.rs) - Shows how to move a tile without despawning and respawning it.
- [`random_map`](examples/random_map.rs) - A bench of editing all of the tiles every 100 ms.
- [`remove_tiles`](examples/remove_tiles.rs) - An example showing how you can remove tiles by using map_query
- [`spacing`](examples/spacing.rs) - Shows how to load tilemap textures that contain spacing between the tiles.
- [`spawn_despawn_tilemap`](examples/spawn_despawn_tilemap.rs) - Shows how spawn and despawn tilemaps.
- [`texture_container`](examples/texture_container.rs) - An example showing how to load tiles from array layers inside a KTX2 or DDS container.
- [`texture_vec`](examples/texture_vec.rs) - An example showing how to load tiles from a list of individual image assets.
- [`tiled`](examples/tiled.rs) - An example of loading and rendering of a [Tiled](https://www.mapeditor.org/) editor map. We recommend checking out [`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled).
- [`tiled_rotated`](examples/tiled_rotated.rs) - An example of loading and rendering of a [Tiled](https://www.mapeditor.org/) editor map with flipping and rotation.
- [`visibility`](examples/visibility.rs) - An example showcasing visibility of tiles and chunks.

### Running Examples

```bash
cargo run --release --example basic
```

### Running examples on web

This can be made simple with [wasm-server-runner](https://github.com/jakobhellermann/wasm-server-runner).

After that's installed and configured, run:

#### WebGL2

```bash
cargo run --target wasm32-unknown-unknown --example animation
```

#### WebGPU

WebGPU is not yet well [supported](https://caniuse.com/webgpu) by many browsers.

```
cargo run --example animation --target=wasm32-unknown-unknown --features=bevy/webgpu
```

## Bevy Compatibility

| bevy   | bevy_ecs_tilemap |
| ------ | ---------------- |
| `main` | `bevy-track`     |
| 0.15   | 0.15             |
| 0.14   | 0.14             |
| 0.13   | -                |
| 0.12   | 0.12             |
| 0.11   | 0.11             |
| 0.10   | 0.10             |
| 0.9    | 0.9              |
| 0.8    | 0.8              |
| 0.8    | 0.7              |
| 0.7    | 0.6              |
| 0.6    | 0.5              |

## Asset credits

- Field of green by [GuttyKreum](https://guttykreum.itch.io/).
