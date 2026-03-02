FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache ca-certificates libssl3

WORKDIR /app

COPY --from=builder /app/target/release/driftguard /usr/local/bin/

RUN mkdir -p /data/screenshots

ENV DATABASE_URL=/data/driftguard.db
ENV SCREENSHOT_DIR=/data/screenshots
ENV PORT=8080
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["driftguard"]
CMD ["serve"]
