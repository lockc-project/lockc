/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "policy.h"

#define TASK_COMM_LEN 16
#define CONTAINER_ID_MAX_LIMIT 1024

/*
 * runtime_key - key of the `runtimes` BPF map containing the runtime process
 * name.
 */
struct runtime_key {
	unsigned char comm[TASK_COMM_LEN];
};

/*
 * container_key - key of the `containers` BPF map which contains the container
 * ID.
 */
struct container_key {
	unsigned char container_id[CONTAINER_ID_MAX_LIMIT];
};

/*
 * container - value of the `containers` BPF map which contains the info about
 * a policy which should be enforced on the given container.
 */
struct container {
	enum container_policy_level policy_level;
};
