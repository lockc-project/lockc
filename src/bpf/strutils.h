/* SPDX-License-Identifier: GPL-2.0-or-later */

#ifndef STRUTILS_H
#define STRUTILS_H

/*
 * strlen - check the length of a null-terminated string.
 * @s: the string
 *
 * Return: length of the string.
 */
static __always_inline size_t strlen(const char *s)
{
	size_t len = 0;

	while (1) {
		if (*s++ == '\0')
			return len;
		len++;
	}
}

#endif
