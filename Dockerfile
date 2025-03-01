FROM rust:1.83.0-slim AS builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app

COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/photos_backend-cli /usr/app/photos_backend-cli

ENTRYPOINT ["/usr/app/photos_backend-cli"]
