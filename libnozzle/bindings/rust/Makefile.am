

EXTRA_DIST = Cargo.toml \
	build.rs \
	regenerate-sys.sh \
	src/libnozzle.rs \
	src/lib.rs \
	src/sys/libnozzle.rs \
	src/sys/mod.rs

all-local:
	cargo build $(RUSTFLAGS)

clean-local:
	-cargo clean

