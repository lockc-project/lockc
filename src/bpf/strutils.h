/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>

/*
 * hash - simple string hash function which allows to use strings as keys for
 * BPF maps even though they use u32 as a key type.
 * @str: string to hash
 * @len: length of the string
 *
 * Return: an u32 value representing the given string.
 */
static __always_inline u32 hash(const char *str, size_t len)
{
	u32 hash = 0;
	int i;

	for (i = 0; i < len; i++) {
		if (str[i] == '\0')
			return hash;
		hash += str[i];
	}

	return hash;
}

/*
 * strlen - check the length of a null-terminated string.
 * @s: the string
 *
 * Return: length of the string.
 */
static __always_inline int strlen(const unsigned char *s, size_t buf_len)
{
	size_t i;

	for (i = 0; i < buf_len; i++) {
		if (s[i] == '\0')
			return i;
	}

	return i;
}

/*
 * strcmp - compare two given strings.
 * @p1: the first string
 * @p2: the second string
 *
 * Return: 0 if strings are equal, otherwise the lexicographic difference
 * between those strings.
 */
static __always_inline int strcmp(const unsigned char *p1,
				  const unsigned char *p2, size_t len)
{
	unsigned char c1, c2;
	size_t i;

	for (i = 0; i < len; i++) {
		c1 = (unsigned char) p1[i];
		c2 = (unsigned char) p2[i];

		if (c1 != c2 || c1 == '\0' || c2 == '\0')
			return c1 - c2;
	}

	return 0;
}
