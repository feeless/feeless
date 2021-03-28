test:
	cargo test
	cargo build
	cargo run --example cli -- target/debug/feeless

docker:
	docker build . --progress=plain -t feeless