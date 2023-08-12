FROM scratch
ENV RUST_LOG=info
COPY target/x86_64-unknown-linux-musl/release/beacon /bin/beacon
EXPOSE 7878
ENTRYPOINT [ "/bin/beacon" ]
