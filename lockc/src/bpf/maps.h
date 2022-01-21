/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "map_structs.h"

/*
 * containers - BPF map containing the info about a policy which should be
 * enforced on the given container.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PID_MAX_LIMIT);
	__type(key, u32);
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
	__type(value, struct process);
} processes SEC(".maps");

/*
 * ap_mnt_restr - BPF map which contains the source path prefixes allowed to
 * bind mount from host to restricted containers. It should contain only
 * paths used by default by container runtimes, not paths mounted with the -v
 * option.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} ap_mnt_restr SEC(".maps");

/*
 * ap_mnt_base - BPF map which contains the source path prefixes allowed to
 * bind mount from host to baseline containers. It should contain both paths
 * used by default by container runtimes and paths we allow to mount with -v
 * option.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} ap_mnt_base SEC(".maps");

/*
 * ap_acc_restr - BPF map which contains the path prefixes allowed to access
 * (open, create, delete, move etc.) inside filesystems of restricted
 * containers.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} ap_acc_restr SEC(".maps");

/*
 * ap_acc_base - BPF map which contains the path prefixes allowed to access
 * (open, create, delete, move etc.) inside filesystems of baseline containers.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} ap_acc_base SEC(".maps");

/*
 * dp_acc_restr - BPF map which contains the path prefixes denied to access
 * (open, create, delete, move etc.) inside filesystems of restricted
 * containers.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} dp_acc_restr SEC(".maps");

/*
 * dp_acc_base - BPF map which contains the path prefixes denied to access
 * (open, create, delete, move etc.) inside filesystems of baseline containers.
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PATH_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct accessed_path);
} dp_acc_base SEC(".maps");
