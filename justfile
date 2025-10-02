# justfile for building marty and plugins

# Build the main marty binary
build-marty:
    cargo build --release --workspace

# Build all plugins to WASM (wasm32-wasip1 target)
build-plugins:
    just build-cargo-plugin
    just build-pnpm-plugin
    just build-typescript-plugin

# Build the cargo plugin to WASM and copy to .marty/plugins
build-cargo-plugin:
    cargo build --release --target wasm32-wasip1 --manifest-path plugins/cargo/Cargo.toml
    mkdir -p .marty/plugins
    cp target/wasm32-wasip1/release/marty-plugin-cargo.wasm .marty/plugins/

# Build the pnpm plugin to WASM and copy to .marty/plugins
build-pnpm-plugin:
    cargo build --release --target wasm32-wasip1 --manifest-path plugins/pnpm/Cargo.toml
    mkdir -p .marty/plugins
    cp target/wasm32-wasip1/release/marty-plugin-pnpm.wasm .marty/plugins/

# Build the typescript plugin to WASM and copy to .marty/plugins
build-typescript-plugin:
    cargo build --release --target wasm32-wasip1 --manifest-path plugins/typescript/Cargo.toml
    mkdir -p .marty/plugins
    cp target/wasm32-wasip1/release/marty-plugin-typescript.wasm .marty/plugins/

# Clean all build artifacts
clean:
    cargo clean
    rm -rf .marty/plugins/*.wasm
