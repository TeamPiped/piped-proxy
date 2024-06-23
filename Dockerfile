FROM rust:alpine as BUILD

RUN apk add --no-cache nasm git g++

RUN git clone https://github.com/TeamPiped/piped-proxy.git /app

WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target/   \
    cargo build --release --target=$(rustc -vV | grep host | cut -d ' ' -f2) && \
    mv target/$(rustc -vV | grep host | cut -d ' ' -f2)/release/piped-proxy .

FROM scratch

WORKDIR /app/

COPY --from=BUILD /app/piped-proxy .

EXPOSE 8080

CMD ["/app/piped-proxy"]
