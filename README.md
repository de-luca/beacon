# Beacon

> Interplanar Beacon

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
