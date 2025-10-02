# justfile for building marty and plugins

# Build the main marty binary
build-marty:
    cargo build --release --workspace

# Build all plugins as dynamic libraries
build-plugins:
    just build-cargo-plugin
    just build-pnpm-plugin
    just build-typescript-plugin

# Build the cargo plugin as dynamic library and copy to .marty/plugins
build-cargo-plugin:
    #!/usr/bin/env sh
    cargo build --release --manifest-path plugins/cargo/Cargo.toml
    mkdir -p .marty/plugins
    if [ "$(uname)" = "Darwin" ]; then
        cp target/release/libmarty_plugin_cargo.dylib .marty/plugins/
    else
        cp target/release/libmarty_plugin_cargo.so .marty/plugins/
    fi

# Build the pnpm plugin as dynamic library and copy to .marty/plugins  
build-pnpm-plugin:
    #!/usr/bin/env sh
    cargo build --release --manifest-path plugins/pnpm/Cargo.toml
    mkdir -p .marty/plugins
    if [ "$(uname)" = "Darwin" ]; then
        cp target/release/libmarty_plugin_pnpm.dylib .marty/plugins/
    else
        cp target/release/libmarty_plugin_pnpm.so .marty/plugins/
    fi

# Build the typescript plugin as dynamic library and copy to .marty/plugins
build-typescript-plugin:
    #!/usr/bin/env sh
    cargo build --release --manifest-path plugins/typescript/Cargo.toml
    mkdir -p .marty/plugins
    if [ "$(uname)" = "Darwin" ]; then
        cp target/release/libmarty_plugin_typescript.dylib .marty/plugins/
    else
        cp target/release/libmarty_plugin_typescript.so .marty/plugins/
    fi

# Clean all build artifacts
clean:
    cargo clean
    rm -rf .marty/plugins/*
