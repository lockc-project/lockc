/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "map_structs.h"

/*
 * Max configurable PID limit (for x86_64, for the other architectures it's less
 * or equal).
 */
#define PID_MAX_LIMIT 4194304

/*
 * runtimes - BPF map containing the process names of container runtime init
 * processes (for example: `runc:[2:INIT]` which is the name of every init
 * process for runc).
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 16);
	__type(key, struct runtime_key);
	__type(value, u32);
} runtimes SEC(".maps");

/*
 * containers - BPF map containing the info about a policy which should be
 * enforced on the given container.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PID_MAX_LIMIT);
	__type(key, struct container_key);
	__type(value, struct container);
} containers SEC(".maps");

/*
 * processes - BPF map which maps the PID to a container it belongs to. The
 * value of this map, which represents the container, is a key of `containers`
 * BPF map, so it can be used immediately for lookups in `containers` map.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PID_MAX_LIMIT);
	__type(key, pid_t);
	__type(value, struct container_key);
} processes SEC(".maps");
