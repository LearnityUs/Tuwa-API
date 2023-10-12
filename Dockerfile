# Builder
FROM rust:buster as builder

WORKDIR /usr/runner/app
COPY . .

RUN cargo install --path ./crates/app/

# Okay, now we build the actual image, we now need to migrate the DB

RUN cargo run --release --bin migrations -- up

# Runner
FROM ubuntu:latest
COPY --from=builder /usr/local/cargo/bin/app /usr/local/bin/app

EXPOSE 8080

CMD ["app"]