#
# Copyright (C) 2021 Red Hat, Inc.  All rights reserved.
#
# Author: Christine Caulfield <ccaulfie@redhat.com>
#
# This software licensed under GPL-2.0+
#

MAINTAINERCLEANFILES	= Makefile.in

include $(top_srcdir)/build-aux/check.mk

SUBDIRS			= . tests

EXTRA_DIST		= Cargo.toml \
			  regenerate-sys.sh \
			  build.rs \
			  src/libknet.rs \
			  src/lib.rs \
			  src/sys/libknet.rs \
			  src/sys/mod.rs

all-local:
	$(CARGO) build $(RUST_FLAGS)

clean-local:
	-$(CARGO) clean
	rm -rf Cargo.lock
