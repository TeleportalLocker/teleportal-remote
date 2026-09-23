# Build depuis la racine du monorepo :
#   docker build -f infra/docker/Dockerfile -t teleportal-relay .
#
# Multi-stage : compile uniquement teleportal-relay-server.

FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build -p teleportal-relay-server --release \
    && strip target/release/teleportal-relay-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/teleportal-relay-server /usr/local/bin/teleportal-relay-server

ENV TELEPORTAL_RELAY_BIND=0.0.0.0:7800
EXPOSE 7800
USER nobody
CMD ["teleportal-relay-server"]
