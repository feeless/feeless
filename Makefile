test:
	cargo test
	cargo build
	cargo run --example cli -- target/debug/feeless

	docker build . --progress=plain -t feeless

release:
	cargo release
