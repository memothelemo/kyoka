# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.74.0
ARG USER=kyoka

FROM rust:${RUST_VERSION} AS compile

WORKDIR /usr/build/kyoka
COPY . .

# Debug build or release
RUN --mount=type=cache,target=/usr/local/cargo/registry set -ex; cargo fetch

RUN --mount=type=cache,target=/usr/build/kyoka/target,target=/usr/local/cargo/registry set -ex; \
    cargo build --release; \
    mkdir -p dist; \
    mv target/release/setup dist/; \
    mv target/release/shard dist/;

###############################################################
FROM debian:bookworm-slim AS runner

RUN apt update && apt install -y ca-certificates

# Setup unprivileged user
RUN adduser \
    --disabled-password \
    --home "/dev/null" \
    --no-create-home \
    --gecos "" \
    worker

WORKDIR /app

COPY --chmod=0755 --from=compile /usr/build/kyoka/dist/* ./
COPY --chmod=0755 --from=compile /usr/build/kyoka/dev/entrypoint.sh ./

USER worker

LABEL org.opencontainers.image.authors="memothelemo"
LABEL org.opencontainers.image.source="https://github.com/memothelemo/kyoka"
LABEL org.opencontainers.image.description=""

ENTRYPOINT [ "./entrypoint.sh" ]
