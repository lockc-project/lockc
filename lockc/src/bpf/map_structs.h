/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "limits.h"
#include "policy.h"

struct container_id {
	unsigned char id[CONTAINER_ID_LIMIT];
};

struct container {
	enum container_policy_level policy_level;
};

struct process {
	struct container_id container_id;
};

struct accessed_path {
	unsigned char path[PATH_LEN];
};
