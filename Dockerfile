FROM rust:1.81 AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/pokemon_api /usr/local/bin/pokemon_api
COPY --from=builder /usr/src/app/Rocket.toml /Rocket.toml
CMD ["pokemon_api"]