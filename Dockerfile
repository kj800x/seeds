# Build Stage
FROM rust:1.93-alpine AS builder
WORKDIR /usr/src/
# Install required build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static gcc g++ make

# - Install dependencies
WORKDIR /usr/src
RUN USER=root cargo new seeds-rs
WORKDIR /usr/src/seeds-rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# - Copy source
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---- Runtime Stage ----
FROM alpine:latest AS runtime
WORKDIR /app
COPY --from=builder /usr/src/seeds-rs/target/release/seeds-rs /usr/local/bin/seeds-rs
COPY static ./static
COPY migrations ./migrations
USER 1000
CMD ["seeds-rs"]
