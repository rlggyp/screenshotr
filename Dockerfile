FROM rust:bookworm AS builder

WORKDIR /app
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo build --release
RUN mkdir -p /assets/screenshots

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/screenshotr /screenshotr
COPY --from=builder /assets /assets

ENV TZ=Asia/Jakarta
EXPOSE 12009
USER nonroot
ENTRYPOINT ["/screenshotr"]