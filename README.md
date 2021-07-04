# Beacon

> Interplanar Beacon

## Dev tools

### Format
```shell
cargo fmt
```

### Lint
```shell
cargo clippy
```

### Static build
```shell
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
```
