# Beacon

> Interplanar Beacon

A "room oriented" signaling server using websockets.  

## Usage
```shell
Usage: beacon [<addr>] -c <cert> -k <key>

Beacon server

Options:
  -c, --cert        cert file
  -k, --key         key file
  --help            display usage information
```

```shell
# with specific log level
RUST_LOG=debug beacon [...]
```

Starts without TLS if no cert nor key are given.

## Dev tools

### Local certs
Use [mkcert](https://github.com/FiloSottile/mkcert) for simplicity
```shell
mkcert -install localhost
```

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
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target x86_64-unknown-linux-musl
```
