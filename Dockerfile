# Build
FROM rust:slim AS build

WORKDIR /tmp/app

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

ADD Cargo.toml Cargo.lock ./
ADD src/ ./src
ADD crates/ ./crates

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev && \
    cargo install sqlx-cli && cargo sqlx prepare && \
    rm -rf /var/lib/apt/lists/*

RUN cargo test && cargo build --release

# Run
FROM gcr.io/distroless/cc-debian12

COPY --from=build /tmp/app/target/release/todo-service ./todo-service

CMD ["./todo-service"]
