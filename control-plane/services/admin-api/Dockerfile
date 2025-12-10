FROM rust:1.83 as builder

WORKDIR /usr/src/drizzle
COPY . .

# Build admin-api
RUN cargo build --release -p admin-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/drizzle/target/release/admin-api /usr/local/bin/admin-api
COPY --from=builder /usr/src/drizzle/control-plane/crates/storage/migrations /migrations

# We might want to install sqlx-cli or run migration in code
# For now, assuming external migration or future improvement

EXPOSE 3000

CMD ["admin-api"]
