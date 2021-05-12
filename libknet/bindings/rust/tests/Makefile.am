
EXTRA_DIST = Cargo.toml \
		src/bin/knet-test.rs

check_PROGRAMS = \
	target/$(RUST_TARGET_DIR)/knet-test

noinst_PROGRAMS = $(check_PROGRAMS)

testsuitedir = $(TESTDIR)
testsuite_PROGRAMS = $(check_PROGRAMS)

TESTS = $(check_PROGRAMS)

target/$(RUST_TARGET_DIR)/knet-test: src/bin/knet-test.rs
	cargo build $(RUSTFLAGS)

clean-local:
	cargo clean
