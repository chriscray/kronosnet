
EXTRA_DIST = Cargo.toml \
	src/bin/nozzle-test.rs \
	up.d/tap33

if BUILD_LIBNOZZLE
check_PROGRAMS = target/$(RUST_TARGET_DIR)/nozzle-test

noinst_PROGRAMS = $(check_PROGRAMS)

testsuitedir = $(TESTDIR)
testsuite_PROGRAMS = $(check_PROGRAMS)

TESTS = $(check_PROGRAMS)

noinst_SCRIPTS = up.d/tap33

target/$(RUST_TARGET_DIR)/nozzle-test: src/bin/nozzle-test.rs
	cargo build $(RUSTFLAGS)

clean-local:
	cargo clean
endif
