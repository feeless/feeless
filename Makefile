test:
	cargo fmt -- --check
	cargo test
	cargo build --features deny_warnings
	cargo run --example cli --features deny_warnings -- target/debug/feeless

# Build a docker image, similar to the published one.
docker:
	docker build . --progress=plain -t feeless

# You'll need: `cargo install cargo-release`
release:
	cargo release patch
