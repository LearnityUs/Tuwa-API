# Builder
FROM rust:buster as builder

WORKDIR /usr/runner/app
COPY . .

RUN cargo install --path ./crates/app/

# Runner
FROM ubuntu:latest
COPY --from=builder /usr/local/cargo/bin/app /usr/local/bin/app

RUN apt-get update && apt install -y openssl

EXPOSE 8080

CMD ["app"]