FROM rust:1.51.0-slim as build

WORKDIR /feeless
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /feeless/target/release/feeless /usr/bin/feeless
ENTRYPOINT ["/usr/bin/feeless"]
