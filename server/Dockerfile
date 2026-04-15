FROM rust:1.94-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates sqlite3 tini \
 && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 --home /var/lib/moestuin moestuin \
 && mkdir -p /var/lib/moestuin \
 && chown moestuin:moestuin /var/lib/moestuin
COPY --from=build /src/target/release/server /usr/local/bin/moestuin
USER moestuin
WORKDIR /var/lib/moestuin
EXPOSE 8080
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/moestuin"]
