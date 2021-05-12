

EXTRA_DIST = Cargo.toml \
	regenerate-sys.sh \
	build.rs \
	src/libknet.rs \
	src/lib.rs \
	src/sys/libknet.rs \
	src/sys/mod.rs

all-local:
	cargo build $(RUSTFLAGS)

clean-local:
	-cargo clean

