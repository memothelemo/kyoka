# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.74.0
ARG USER=kyoka

FROM rust:${RUST_VERSION} AS compile

# TODO: Remove libssl because we're using rustls for this.
WORKDIR /usr/build/kyoka
COPY . .

# Debug build or release
RUN --mount=type=cache,target=/usr/build/kyoka/target set -ex; \
    cargo build --release; \
    mkdir -p dist; \
    mkdir -p cfg; \
    mv target/release/setup dist/; \
    mv target/release/shard dist/;

# Safely copy environment files and entrypoint script
RUN if [ -f "config/kyoka.toml" ]; then \
    cp config/kyoka.toml cfg/; \
fi

RUN if [ -f ".env" ]; then \
    cp .env cfg/; \
fi

# As I really want this env run in Alpine Linux, we'll use Debian for now
FROM debian:bookworm-slim AS runner
# FROM alpine:3.19.0 AS runner

RUN apt update && apt install -y libssl-dev ca-certificates
# RUN apk update && apk add --no-cache ca-certificates
# RUN update-ca-certificates

# Setup unprivileged user
RUN adduser \
    --disabled-password \
    --home "/dev/null" \
    --no-create-home \
    --gecos "" \
    worker

# RUN addgroup -S appgroup && adduser -S worker -G appgroup

WORKDIR /app

# as long as almost nobody will try to deploy a dev container
# on a critical/production environment
COPY --chmod=0755 --from=compile /usr/build/kyoka/cfg/* ./
COPY --chmod=0755 --from=compile /usr/build/kyoka/dist/* ./
COPY --chmod=0755 --from=compile /usr/build/kyoka/dev/entrypoint.sh ./

USER worker

ENTRYPOINT [ "./entrypoint.sh" ]

# RUN update-ca-certificates

# # Initialize musl environment
# RUN rustup target add x86_64-unknown-linux-musl
# RUN apt update && apt install -y musl-tools musl-dev libssl-dev

# # Cache build dependencies to take advantage called
# # as `incremental compilation`
# WORKDIR /usr/build/
# RUN cargo new kyoka
# COPY Cargo.toml Cargo.lock /usr/build/kyoka/
# COPY src/ /usr/build/kyoka/src

# WORKDIR /usr/build/kyoka/
# RUN cargo build --target=x86_64-unknown-linux-musl

# # Copy actual application source in and force a modified
# # timestamp so that Cargo will rebuild.
# COPY . .
# RUN cargo build --target=x86_64-unknown-linux-musl

# # Very exciting part!
# FROM alpine:3.19.0 AS runner

# RUN apk update && apk add --no-cache ca-certificates
# RUN update-ca-certificates

# WORKDIR /app

# # Setup unprivileged user
# RUN addgroup -S appgroup && adduser -S worker -G appgroup

# COPY --chown=worker --from=compile_stage /usr/build/kyoka/target/x86_64-unknown-linux-musl/debug/setup /app/
# COPY --chown=worker --from=compile_stage /usr/build/kyoka/target/x86_64-unknown-linux-musl/debug/shard /app/

# # Setup bot and some requirements
# USER worker
# WORKDIR /app

# CMD [ "./shard" ]
