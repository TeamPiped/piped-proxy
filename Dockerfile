FROM rust:slim as BUILD

WORKDIR /app/

COPY . .

RUN --mount=type=chache,target=/var/cache/apt \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    nasm && \
    rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/   \
    cargo build --release && \
    mv target/release/piped-proxy .

FROM debian:stable-slim

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/

COPY --from=BUILD /app/piped-proxy .

EXPOSE 8080

CMD ["/app/piped-proxy"]
