FROM rust:1.51.0-slim as build

WORKDIR /feeless
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /feeless/target/release/feeless /usr/bin/feeless

# Do not run as root, do not allow a shell
RUN groupadd -g 7075 feeless
RUN useradd -g 7075 -l -M -s /bin/false -u 7075 feeless
USER feeless

ENTRYPOINT ["/usr/bin/feeless"]
