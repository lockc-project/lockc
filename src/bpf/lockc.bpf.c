/* SPDX-License-Identifier: GPL-2.0-or-later */
#include "vmlinux.h"
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <errno.h>

#include "compiler.h"
#include "maps.h"
#include "strutils.h"

#ifndef NULL
#define NULL 0
#endif

// #define EPERM 1
// #define ENOENT 2
// #define EFAULT 14

/*
 * The `type` pointer coming from the sb_mount LSM hook has allocatted a full
 * page size, but since we are interested only in "bind" mounts, allocating a
 * buffer of size 5 is enough.
 */
#define MOUNT_TYPE_LEN 5
#define MOUNT_TYPE_BIND "bind"

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
	struct process *parent_lookup = bpf_map_lookup_elem(&processes, &ppid);
	if (!parent_lookup) {
		/* If not, check whether it's a container runtime process. */
		// const char *comm = BPF_CORE_READ(child, comm);
		// u32 runtime_key = hash(comm, TASK_COMM_LEN);
		// u32 *runtime_lookup = bpf_map_lookup_elem(&runtimes,
		// 					  &runtime_key);
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
	struct process *v = bpf_map_lookup_elem(&processes, &pid);
	if (v != NULL)
		return 0;

	bpf_printk("found parent containerized process: %d\n", ppid);
	bpf_printk("comm: %s\n", BPF_CORE_READ(child, comm));

	u32 container_id = parent_lookup->container_id;
	u32 *container_lookup = bpf_map_lookup_elem(&containers, &container_id);
	if (!container_lookup) {
		/* Shouldn't happen */
		bpf_printk("error: handle_new_process: cound not find a "
			   "container for a registered process %d, "
			   "container id: %d\n",
			   pid, container_id);
		return -EPERM;
	}

	struct process new_p = {
		.container_id = container_id
	};

	bpf_printk("adding containerized process: %d\n", pid);
	err = bpf_map_update_elem(&processes, &pid, &new_p, 0);
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

	struct process *p = bpf_map_lookup_elem(&processes, &pid);
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
		bpf_printk("error: sched_process_fork: parent or child is "
			   "NULL\n");
		return -EPERM;
	}

	return handle_new_process(parent, child);
}

SEC("fentry/do_exit")
int BPF_PROG(do_exit, long code)
{
	pid_t pid = bpf_get_current_pid_tgid() >> 32;
	int err;

	struct process *p = bpf_map_lookup_elem(&processes, &pid);
	/* Ignore if process is not containerized */
	if (!p)
		return 0;

	bpf_printk("process %d is exiting", pid);
	err = bpf_map_delete_elem(&processes, &pid);
	if (err < 0) {
		bpf_printk("error: do_exit: could not remove old process");
	}

	return 0;
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

/*
 * callback_ctx - input/output data for the `check_allowed_paths` callback
 * function.
 */
struct callback_ctx {
	/* Input path to compare all the allowed paths with. */
	unsigned char *path;
	/* Output whether a match was found. */
	bool found;
};

/*
 * check_allowed_paths - callback function which checks whether the given source
 * path (about which we make decision whether to mount it) matches the currently
 * checked allowed path.
 * @map: the BPF map with allowed paths
 * @key: the key of the checked BPF map element
 * @allowed_path: the checked BPF map element
 * @data: input/output data shared between this callback and the BPF program
 *
 * Return: 0 if the match was found and next iterations should be stopped.
 * 1 if the match was not found and the search for a possible match should be
 * continued.
 */
static u64 check_allowed_paths(struct bpf_map *map, u32 *key,
			       struct allowed_path *allowed_path,
			       struct callback_ctx *data)
{
	/*
	 * Shouldn't happen, but if in any case the checked path is NULL, skip
	 * it and go to the next element. Comparing it would result in a match
	 * (because of comparing with 0 length).
	 */
	if (unlikely(allowed_path == NULL))
		return 0;

	bpf_printk("checking path: key: %u, dev_name: %s, current: %s\n",
		   *key, data->path, allowed_path->path);

	size_t allowed_path_len = strlen(allowed_path->path, PATH_LEN);

	/*
	 * Shouldn't happen, but if in any case the checked path is empty, skip
	 * it and go to the next element. Comparing it could result in a match
	 * (because of comparing with 0 length).
	 */
	if (unlikely(allowed_path_len < 1))
		return 0;

	if (strcmp(allowed_path->path, data->path, allowed_path_len) == 0) {
		bpf_printk("path check matched\n");
		data->found = true;
		return 1;
	}

	return 0;
}

/*
 * mount_audit - LSM program triggered by any mount attempt. Its goal is to deny
 * the bind mounts to restricted and baseline containers whose source prefixes
 * are not specified as allowed in BPF maps.
 * @dev_name: source path
 * @path: destination path
 * @type: type of mount
 * @flags: mount flags
 * @data: filesystem-specific data
 * @ret_prev: return code of a previous BPF program using the sb_mount hook
 *
 * Return: 0 if mount allowed. -EPERM if mount not allowed. -EFAULT if there was
 * a problem with reading the kernel strings into buffers or any important
 * buffer is NULL.
 */
SEC("lsm/sb_mount")
int BPF_PROG(mount_audit, const char *dev_name, const struct path *path,
	     const char *type, unsigned long flags, void *data, int ret_prev)
{
	int ret = 0;
	pid_t pid = bpf_get_current_pid_tgid() >> 32;
	enum container_policy_level policy_level = get_policy_level(pid);
	struct path *path_mut = (struct path *) path;
	unsigned char type_bind[MOUNT_TYPE_LEN] = MOUNT_TYPE_BIND;
	unsigned char type_safe[MOUNT_TYPE_LEN];
	unsigned char dev_name_safe[PATH_LEN];

	/* Retrieve the mount type. */
	if (unlikely(type == NULL)) {
		/*
		 * TODO(vadorovsky): Investigate the "empty type" mounts more.
		 * Apparently denying them was breaking bwrap and flatpak...
		 */
		bpf_printk("warning: mount type is NULL\n");
		goto out;
	}
	if (unlikely(bpf_probe_read_kernel_str(&type_safe,
					       MOUNT_TYPE_LEN, type) < 0)) {
		bpf_printk("error: could not read the mount type\n");
		ret = -EFAULT;
		goto out;
	}

	/* Apply the policy only on bind mounts. */
	if (strcmp(type_safe, type_bind, MOUNT_TYPE_LEN) != 0)
		goto out;

	/* Check and retrieve the dev_name (source path). */
	if (unlikely(dev_name == NULL)) {
		bpf_printk("error: bind mount without source\n");
		ret = -EFAULT;
		goto out;
	}
	if (unlikely(bpf_probe_read_kernel_str(&dev_name_safe, PATH_LEN,
					       dev_name) < 0)) {
		bpf_printk("error: could not read the mount dev_name\n");
		ret = -EFAULT;
		goto out;
	}

	struct callback_ctx cb = {
		.path = dev_name_safe,
		.found = false
	};

	switch (policy_level) {
	case POLICY_LEVEL_LOOKUP_ERR:
		/* Shouldn't happen */
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_NOT_FOUND:
		goto out;
	case POLICY_LEVEL_RESTRICTED:
		bpf_for_each_map_elem(&allowed_paths_restricted,
				      check_allowed_paths,
				      &cb, 0);
		if (cb.found) {
			bpf_printk("mount: restricted: allow\n");
			goto out;
		}
		bpf_printk("mount: baseline: deny\n");
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_BASELINE:
		bpf_for_each_map_elem(&allowed_paths_baseline,
				      check_allowed_paths,
				      &cb, 0);
		if (cb.found) {
			bpf_printk("mount: baseline: allow\n");
			goto out;
		}
		bpf_printk("mount: baseline: deny\n");
		ret = -EPERM;
		goto out;
	case POLICY_LEVEL_PRIVILEGED:
		bpf_printk("mount: privileged: allow\n");
		goto out;
	}
out:
	if (ret_prev != 0)
		return ret_prev;
	return ret;
}

char __license[] SEC("license") = "GPL";
