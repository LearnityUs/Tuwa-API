# Builder
FROM rust:buster as builder

WORKDIR /usr/runner/app
COPY . .

RUN cargo install --path ./crates/migrations/

# Runner
FROM ubuntu:latest
COPY --from=builder /usr/local/cargo/bin/migrations /usr/local/bin/migrations

EXPOSE 8080

CMD ["migrations", "up"]