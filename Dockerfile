FROM ekidd/rust-musl-builder as builder
ADD --chown=rust:rust ./ /home/rust/src
RUN cargo build --release

# FROM alpine:latest as certs
# RUN apk add --no-cache ca-certificates

FROM scratch
ENV RUST_LOG=info
# COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/beacon /bin/beacon
EXPOSE 7878
ENTRYPOINT [ "/bin/beacon" ]
