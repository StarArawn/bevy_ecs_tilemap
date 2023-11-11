#!/bin/bash

cargo clean

cargo check
if [ $? -gt 0 ]; then
	exit 1
fi

cargo run --example tiled
cargo run --example ldtk
cargo run --example random_map
cargo run --example bench
cargo run --example colors
