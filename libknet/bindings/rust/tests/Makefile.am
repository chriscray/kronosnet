#
# Copyright (C) 2021 Red Hat, Inc.  All rights reserved.
#
# Author: Christine Caulfield <ccaulfie@redhat.com>
#
# This software licensed under GPL-2.0+
#

MAINTAINERCLEANFILES	= Makefile.in

include $(top_srcdir)/build-aux/check.mk

EXTRA_DIST		= Cargo.toml \
			  src/bin/knet-test.rs

check_PROGRAMS		= target/$(RUST_TARGET_DIR)/knet-test

noinst_PROGRAMS		= $(check_PROGRAMS)

testsuitedir		= $(TESTDIR)
testsuite_PROGRAMS	= $(check_PROGRAMS)

AM_TESTS_ENVIRONMENT=LD_LIBRARY_PATH="$(abs_top_builddir)/libknet/.libs"

TESTS			= $(check_PROGRAMS)

target/$(RUST_TARGET_DIR)/knet-test$(EXEEXT): src/bin/knet-test.rs
	$(CARGO) build $(RUST_FLAGS)

clean-local:
	-$(CARGO) clean
	rm -rf Cargo.lock
