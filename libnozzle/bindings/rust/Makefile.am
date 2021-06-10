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
			  build.rs \
			  regenerate-sys.sh \
			  src/libnozzle.rs \
			  src/lib.rs \
			  src/sys/libnozzle.rs \
			  src/sys/mod.rs

all-local:
	$(CARGO) build $(RUST_FLAGS)

clean-local:
	-$(CARGO) clean
	rm -rf Cargo.lock
