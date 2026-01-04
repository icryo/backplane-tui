.PHONY: build static clean install

# Default build (dynamic, faster compile)
build:
	cargo build --release

# Static build (portable, no glibc dependency)
static:
	cargo build --release --target x86_64-unknown-linux-musl

# Clean build artifacts
clean:
	cargo clean

# Install to /usr/local/bin
install: static
	cp target/x86_64-unknown-linux-musl/release/backplane-tui /usr/local/bin/
