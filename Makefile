all: build

build:
	cargo build --release

build-dev:
	cargo build

run:
	cargo run --release

run-dev:
	cargo run

test:
	cargo test

clean:
	rm -rf target/
