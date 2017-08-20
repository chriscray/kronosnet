/*
 * Copyright (C) 2010-2017 Red Hat, Inc.  All rights reserved.
 *
 * Author: Fabio M. Di Nitto <fabbione@kronosnet.org>
 *
 * This software licensed under GPL-2.0+, LGPL-2.0+
 */

#ifndef __KNET_COMPRESS_H__
#define __KNET_COMPRESS_H__

#include "internals.h"

typedef struct {
	uint8_t		model_id;
	uint8_t		built_in;
	uint8_t		loaded;
	const char	*model_name;
	int (*is_init)  (knet_handle_t knet_h, int method_idx);
	int (*init)     (knet_handle_t knet_h, int method_idx);
	void (*fini)    (knet_handle_t knet_h, int method_idx);
	int (*val_level)(knet_handle_t knet_h,
			 int compress_level);
	int (*compress)	(knet_handle_t knet_h,
			 const unsigned char *buf_in,
			 const ssize_t buf_in_len,
			 unsigned char *buf_out,
			 ssize_t *buf_out_len);
	int (*decompress)(knet_handle_t knet_h,
			 const unsigned char *buf_in,
			 const ssize_t buf_in_len,
			 unsigned char *buf_out,
			 ssize_t *buf_out_len);
} compress_model_t;

int compress_cfg(
	knet_handle_t knet_h,
	struct knet_handle_compress_cfg *knet_handle_compress_cfg);

int compress_init(
	knet_handle_t knet_h);

void compress_fini(
	knet_handle_t knet_h);

int compress(
	knet_handle_t knet_h,
	const unsigned char *buf_in,
	const ssize_t buf_in_len,
	unsigned char *buf_out,
	ssize_t *buf_out_len);

int decompress(
	knet_handle_t knet_h,
	int compress_model,
	const unsigned char *buf_in,
	const ssize_t buf_in_len,
	unsigned char *buf_out,
	ssize_t *buf_out_len);

#endif
