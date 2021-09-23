/* SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

/*
 * container_policy_level - enum defining all supported policy levels for
 * containers.
 */
enum container_policy_level {
	POLICY_LEVEL_LOOKUP_ERR = -2,
	POLICY_LEVEL_NOT_FOUND = -1,

	POLICY_LEVEL_RESTRICTED,
	POLICY_LEVEL_BASELINE,
	POLICY_LEVEL_PRIVILEGED
};
