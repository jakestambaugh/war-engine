FROM lukemathwalker/cargo-chef:latest-rust-1.70 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin war

FROM debian:bullseye-slim AS runtime
WORKDIR /www
COPY --from=builder /app/target/release/war /usr/local/bin
# The HTML file is included at build time
# COPY *.html /www/
ENV RUST_LOG=debug
ENTRYPOINT ["/usr/local/bin/war"]
# CMD [ "--data-path=/www/"]

