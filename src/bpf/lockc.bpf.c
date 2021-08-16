/* SPDX-License-Identifier: GPL-2.0-or-later */

#include "vmlinux.h"
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

#include "maps.h"
#include "strutils.h"

#ifndef NULL
#define NULL 0
#endif

#define EPERM 1

/*
 * handle_process - the handler which monitors all new tasks/functions created
 * in the system and checks whether:
 * - it's a child of some already containerized process (either the container
 *   runtime init process or any of its children)
 * In any other case, it does not do anything.
 * @parent: the parent task
 * @child: the new task
 *
 * Return: 0 if all operations sucessful, otherwise an error code.
 */
static __always_inline int handle_new_process(struct task_struct *parent,
					      struct task_struct *child)
{
	int err;
	pid_t pid = BPF_CORE_READ(child, pid);
	pid_t ppid = BPF_CORE_READ(parent, pid);

	/* Check if parent process is containerized. */
	struct container_key *parent_lookup = bpf_map_lookup_elem(&processes,
								  &ppid);
	if (!parent_lookup) {
		/* If not, check whether it's a container runtime process. */
		// const char *comm = BPF_CORE_READ(child, comm);
		// struct runtime_key rk = {};
		// __builtin_memcpy(rk.comm, comm, strlen(comm));
		// u32 *runtime_lookup = bpf_map_lookup_elem(&runtimes,
		// 					  &rk);
		// if (runtime_lookup) {
		// 	/*
		// 	 * If yes, it means that's an unwrapped container
		// 	 * runtime process. Deny it.
		// 	 */
		// 	bpf_printk("deny: unwrapped runtime process %d: %s\n",
		// 		   pid,
		// 		   BPF_CORE_READ(child, comm));
		// 	return -EPERM;
		// }
		return 0;
	}

	/* Skip registration if process entry already exists. */
	struct container_key *v = bpf_map_lookup_elem(&processes, &pid);
	if (v != NULL)
		return 0;

	bpf_printk("found parent containerized process: %d\n", ppid);
	bpf_printk("comm: %s\n", BPF_CORE_READ(child, comm));

	u32 *container_lookup = bpf_map_lookup_elem(&containers, parent_lookup);
	if (!container_lookup) {
		/* Shouldn't happen */
		bpf_printk("error: handle_new_process: cound not find a "
			   "container for a registered process %d, container id: "
			   "%d\n",
			   pid, parent_lookup->container_id);
		return -EPERM;
	}

	bpf_printk("adding containerized process: %d\n", pid);
	err = bpf_map_update_elem(&processes, &pid, parent_lookup, 0);
	if (err < 0)
		return err;

	return 0;
}

/*
 * get_policy_level - find the policy level for the given process.
 * @pid: the PID of the process to find the policy for
 *
 * Return: corresponding policy level (or POLICY_LEVEL_NOT_FOUND when the
 * process is not containerized, or POLICY_LEVEL_LOOKUP_ERROR when the state
 * of BPF maps is inconsistent).
 *
 * TODO: Think of some better way to handle the POLICY_LEVEL_LOOKUP_ERROR - if
 * that value is ever returned, it means that the container/process
 * registration went wrong and we have insonsistent data.
 */
static __always_inline enum container_policy_level get_policy_level(pid_t pid)
{
	int err;

	struct container_key *p = bpf_map_lookup_elem(&processes, &pid);
	if (!p) {
		bpf_printk("could not find policy for pid %d\n", pid);
		return POLICY_LEVEL_NOT_FOUND;
	}

	struct container *c = bpf_map_lookup_elem(&containers,
						  &p->container_id);
	if (!c) {
		/* Shouldn't happen */
		bpf_printk("error: get_policy_level: could not found a "
			   "container for a registered process\n");
		return POLICY_LEVEL_LOOKUP_ERR;
	}

	return c->policy_level;
}

/*
 * BPF programs
 * ============
 */

/*
 * NOTE(mrostecki): Apparently, to monitor **all** the processes in the system,
 * I had to use both the `sched_process_fork` tracepoint and the `task_alloc`
 * LSM hook. When using only one of them, some child processes created inside
 * containers were missing. To be sure we track everything, use both kind of
 * programs for now. They both use the common handle_new_process() function.
 */

/*
 * sched_process_fork - tracepoint program triggered by fork() function.
 */
SEC("tp_btf/sched_process_fork")
int sched_process_fork(struct bpf_raw_tracepoint_args *args)
{
	struct task_struct *parent = (struct task_struct *)args->args[0];
	struct task_struct *child = (struct task_struct *)args->args[1];
	if (parent == NULL || child == NULL) {
		/* Shouldn't happen */
		bpf_printk("error: sched_process_fork: parent or child is NULL\n");
		return -EPERM;
	}

	return handle_new_process(parent, child);
}

/*
 * clone_audit - LSM program triggered by clone().
 */
SEC("lsm/task_alloc")
int BPF_PROG(clone_audit, struct task_struct *task,
	     unsigned long clone_flags, int ret_prev)
{
	struct task_struct *parent = BPF_CORE_READ(task, real_parent);
	if (parent == NULL) {
		/* Shouldn't happen */
		bpf_printk("error: clone_audit: parent is NULL\n");
		return -EPERM;
	}

	int ret = handle_new_process(parent, task);

	/* Handle results of previous programs */
	if (ret_prev != 0)
		return ret_prev;
	return ret;
}

/*
 * syslog_audit - LSM program trigerred by attemps to access the kernel logs.
 * Behavior based on policy levels:
 * - restricted: deny
 * - baseline: deny
 * - privileged: allow
 */
SEC("lsm/syslog")
int BPF_PROG(syslog_audit, int type, int ret_prev)
{
	int ret = 0;
	pid_t pid = bpf_get_current_pid_tgid() >> 32;
	enum container_policy_level policy_level = get_policy_level(pid);

	switch (policy_level) {
	case POLICY_LEVEL_LOOKUP_ERR:
		/* Shouldn't happen */
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_NOT_FOUND:
		goto out;
	case POLICY_LEVEL_RESTRICTED:
		bpf_printk("syslog: restricted: deny\n");
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_BASELINE:
		bpf_printk("syslog: baseline: deny\n");
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_PRIVILEGED:
		bpf_printk("syslog: privileged: allow\n");
		goto out;
	}

out:
	/* Handle results of previous programs */
	if (ret_prev != 0) {
		bpf_printk("syslog previous result\n");
		return ret_prev;
	}
	return ret;
}

char __license[] SEC("license") = "GPL";
