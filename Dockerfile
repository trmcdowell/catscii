FROM rust:1.80.1-bookworm as chef
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin catscii

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the cachin Docker layer!
RUN cargo chef cook --recipe-path recipe.json --bin catscii
# Build application
COPY . .
RUN cargo build --bin catscii

FROM debian:bookworm-slim AS runtime
WORKDIR app

RUN apt-get update -y \ 
&& apt-get install -y --no-install-recommends curl ca-certificates gcc libc6-dev pkg-config libssl-dev \
# Clean up
&& apt-get autoremove -y \ 
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/* \
;

COPY --from=builder /app/target/debug/catscii catscii

ENTRYPOINT ["./catscii"]
