FROM rust:1.85-slim as builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/

COPY --from=builder /build/target/release/dhcp-editor /usr/local/bin/

WORKDIR /app
EXPOSE 8080

CMD ["dhcp-editor", "--config", "/app/dhcpd.conf", "--port", "8080"]
