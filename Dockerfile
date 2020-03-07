# ** base **
FROM rustlang/rust:nightly-slim as base

RUN apt-get update && apt-get install -y libpq-dev

RUN cargo install diesel_cli --no-default-features --features postgres


# ** builder **
FROM base as builder

WORKDIR /walriust

COPY Cargo.toml Cargo.lock rust-toolchain ./

# HACK: make rust be able to cache dockerfile
RUN mkdir -p ./src && \
        echo fn main\(\) \{\} > src/main.rs && \
        cargo build --release && \
        rm ./src/main.rs && \
        rm -rf ./target/release/deps/walriust*

COPY . .

RUN cargo build --release

# ** server **
FROM base as server

COPY --from=builder /walriust/target/release/walriust /walriust
COPY --from=builder /walriust/migrations /migrations

CMD ["/walriust", "server"]

