/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "map_structs.h"
#include <bpf/bpf_helpers.h>

#define PIN_BY_NAME 1

// NOTE(vadorovsky): The bpf_map_def struct from libbpf doesn't contain the
// `pinning` field. Aya uses it (for pinning maps, obviously). This kind of
// structure is used also in Cilium and even in few selftests in the kernel
// tree[1].
//
// [0] https://github.com/cilium/cilium/blob/v1.11.1/bpf/include/bpf/loader.h#L19-L29
// [1] https://elixir.bootlin.com/linux/v5.16.8/source/samples/bpf/tc_l2_redirect_kern.c#L23
/*
 * bpf_elf_map - description of BPF map attributes. Saved in the ELF object.
 */
struct bpf_elf_map {
	u32 type;
	u32 key_size;
	u32 value_size;
	u32 max_entries;
	u32 flags;
	u32 id;
	u32 pinning;
};

/*
 * containers - BPF map containing the info about a policy which should be
 * enforced on the given container.
 */
struct bpf_elf_map SEC("maps/containers") containers = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PID_MAX_LIMIT,
	.key_size = sizeof(struct container_id),
	.value_size = sizeof(struct container),
	.pinning = PIN_BY_NAME,
};

/*
 * processes - BPF map which maps the PID to a container it belongs to. The
 * value of this map, which represents the container, is a key of `containers`
 * BPF map, so it can be used immediately for lookups in `containers` map.
 */
struct bpf_elf_map SEC("maps/processes") processes = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PID_MAX_LIMIT,
	.key_size = sizeof(pid_t),
	.value_size = sizeof(struct process),
	.pinning = PIN_BY_NAME,
};

/*
 * ap_mnt_restr - BPF map which contains the source path prefixes allowed to
 * bind mount from host to restricted containers. It should contain only
 * paths used by default by container runtimes, not paths mounted with the -v
 * option.
 */
struct bpf_elf_map SEC("maps/ap_mnt_restr") ap_mnt_restr = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};

/*
 * ap_mnt_base - BPF map which contains the source path prefixes allowed to
 * bind mount from host to baseline containers. It should contain both paths
 * used by default by container runtimes and paths we allow to mount with -v
 * option.
 */
struct bpf_elf_map SEC("maps/ap_mnt_base") ap_mnt_base = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};

/*
 * ap_acc_restr - BPF map which contains the path prefixes allowed to access
 * (open, create, delete, move etc.) inside filesystems of restricted
 * containers.
 */
struct bpf_elf_map SEC("maps/ap_acc_restr") ap_acc_restr = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};

/*
 * ap_acc_base - BPF map which contains the path prefixes allowed to access
 * (open, create, delete, move etc.) inside filesystems of baseline containers.
 */
struct bpf_elf_map SEC("maps/ap_acc_base") ap_acc_base = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};

/*
 * dp_acc_restr - BPF map which contains the path prefixes denied to access
 * (open, create, delete, move etc.) inside filesystems of restricted
 * containers.
 */
struct bpf_elf_map SEC("maps/dp_acc_restr") dp_acc_restr = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};

/*
 * dp_acc_base - BPF map which contains the path prefixes denied to access
 * (open, create, delete, move etc.) inside filesystems of baseline containers.
 */
struct bpf_elf_map SEC("maps/dp_acc_base") dp_acc_base = {
	.type = BPF_MAP_TYPE_HASH,
	.max_entries = PATH_MAX_LIMIT,
	.key_size = sizeof(u32),
	.value_size = sizeof(struct accessed_path),
	.pinning = PIN_BY_NAME,
};
