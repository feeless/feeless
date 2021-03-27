test:
	cargo test
	cargo build
	cargo run --example cli -- target/debug/feeless

# Build a docker image, similar to the published one.
docker:
	docker build . --progress=plain -t feeless

# You'll need: `cargo install cargo-release`
release:
	cargo release patch
