# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.74.0
ARG USER=kyoka
ARG RUST_BUILD_MODE=release

FROM rust:${RUST_VERSION} AS compile

ARG RUST_BUILD_MODE

WORKDIR /usr/build/kyoka
COPY . .

# Required dependencies for compilation
RUN apt update && apt install -y libopus-dev

# Download dependencies and debug build
RUN --mount=type=cache,target=/usr/local/cargo/registry set -ex; cargo fetch

RUN --mount=type=cache,target=/usr/build/kyoka/target,target=/usr/local/cargo/registry set -ex; \
    if [ "${RUST_BUILD_MODE}" = "debug" ]; then \
        cargo build; \
    elif [ "${RUST_BUILD_MODE}" = "release" ]; then \
        cargo build --release; \
    else \
        echo "Please specify whether RUST_BUILD_MODE is in 'debug' or 'release'"; \
        exit 1;\
    fi; \
    cargo build; \
    mkdir -p dist; \
    mv target/${RUST_BUILD_MODE}/kyoka-setup dist/; \
    mv target/${RUST_BUILD_MODE}/kyoka-shard dist/;

###############################################################
FROM debian:bookworm-slim AS runner

ARG USER

# songbird relies on `ytdl` binary anyway
RUN apt update && apt install -y ca-certificates youtube-dl

# Testing dependencies
RUN set -ex; yt-dlp --help

# Setup unprivileged user
RUN adduser \
    --disabled-password \
    --home "/dev/null" \
    --no-create-home \
    --gecos "" \
    ${USER}

WORKDIR /app

COPY --chmod=0755 --from=compile /usr/build/kyoka/dist/* ./
COPY --chmod=0755 --from=compile /usr/build/kyoka/dev/entrypoint.sh ./

USER ${USER}

LABEL org.opencontainers.image.authors="memothelemo"
LABEL org.opencontainers.image.source="https://github.com/memothelemo/kyoka"
LABEL org.opencontainers.image.description=""

ENTRYPOINT [ "./entrypoint.sh" ]
