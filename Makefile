dependencies:
	rustup component add clippy
	rustup component add rustfmt

clean:
	git clean -x -f tests/tmp
	cargo clean

test: dependencies
	yamllint .
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check
	cargo test

fmt: dependencies
	cargo fmt

build: test
	cargo build
