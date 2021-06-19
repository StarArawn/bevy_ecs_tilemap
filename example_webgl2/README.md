# Example of bevy_ecs_tilemap + wasm

This example runs a Tiled example (isometric grid).

Template based on https://github.com/mrk-its/bevy_webgl2_app_template.

## Prerequisites

```
cargo install cargo-make
```

## Build and serve WASM version
```
cargo make -p release serve
```
then point your browser to http://127.0.0.1:4000/


## Build and run native version
```
cargo make run
```
