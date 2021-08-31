/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include "limits.h"
#include "policy.h"

struct container {
	enum container_policy_level policy_level;
};

struct process {
	unsigned int container_id;
};

struct allowed_path {
	unsigned char path[PATH_LEN];
};
