# social_schedule
[![Build Status](https://travis-ci.org/mkb2091/social_schedule.svg?branch=master)](https://travis-ci.org/mkb2091/social_schedule)

Contains wasm-opt binary from https://github.com/WebAssembly/binaryen to allow use in CI

Commands:

To create release wasm: ./build.sh

To create debug wasm: ./debug_build.sh

To run: env RUSTFLAGS="-C target-cpu=native" cargo run --release --no-default-features --features cli -- {player} {games}

To benchmark: env RUSTFLAGS="-C target-cpu=native" cargo bench --no-default-features

To test: cargo test --no-default-features