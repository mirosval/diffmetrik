test:
	cargo test

test-linux:
	cross test --target x86_64-unknown-linux-gnu
