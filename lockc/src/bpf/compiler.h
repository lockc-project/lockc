/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

/*
 * unlikely - provides the compiler with branch prediction information that the
 * given condition is unlikely to be true.
 */
#ifndef unlikely
#define unlikely(X) __builtin_expect(!!(X), 0)
#endif
