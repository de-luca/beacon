# Beacon

> Interplanar Beacon

A "room oriented" signaling server using websockets.

## Usage
```shell
# command
beacon [interface]
# with specific interface
beacon 0.0.0.0:3030
# with specific log level
RUST_LOG=debug beacon
```

## Dev tools

### Format
```shell
cargo install rustfmt
cargo fmt
```

### Lint
```shell
cargo install clippy
cargo clippy
```

### Static build
```shell
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
```
