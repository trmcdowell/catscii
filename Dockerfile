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
&& apt-get install -y --no-install-recommends \
ca-certificates libc6-dev pkg-config libssl-dev libsqlite3-dev \
# Clean up
&& apt-get autoremove -y \ 
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/* \
;

COPY --from=builder /app/target/debug/catscii catscii

# Copy Geolite2 db
RUN mkdir /db
COPY ./db/GeoLite2-Country.mmdb /db/

ENTRYPOINT ["./catscii"]
