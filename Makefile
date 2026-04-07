.PHONY: build run test clean

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean
