FROM rust:1.83 as builder

WORKDIR /usr/src/drizzle
COPY . .

# Build gatewayd
RUN cargo build --release -p gatewayd

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/drizzle/target/release/gatewayd /usr/local/bin/gatewayd

# Expose Proxy Port and Admin Port
EXPOSE 6188 9111

CMD ["gatewayd"]
