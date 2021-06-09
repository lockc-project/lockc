#include "vmlinux.h"
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

#ifndef NULL
#define NULL 0
#endif

#define EPERM 1
#define TASK_COMM_LEN 16
/*
 * Max configurable PID limit (for x86_64, for the other architectures it's less
 * or equal).
 */
#define PID_MAX_LIMIT 4194304

/*
 * Maps and related structures
 * ===========================
 *
 * All structs below are either BPF maps or structures used as values of those
 * maps.
 */

/*
 * runtimes - BPF map containing the process names of container runtime init
 * processes (for example: `runc:[2:INIT]` which is the name of every init
 * process for runc).
 */
struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 16);
	__type(key, u32);
	__type(value, u32);
} runtimes SEC(".maps");

enum container_policy_level {
	POLICY_LEVEL_LOOKUP_ERR = -2,
	POLICY_LEVEL_NOT_FOUND = -1,

	POLICY_LEVEL_RESTRICTED,
	POLICY_LEVEL_BASELINE,
	POLICY_LEVEL_PRIVILEGED
};

struct container {
	enum container_policy_level policy_level;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PID_MAX_LIMIT);
	__type(key, u32);
	__type(value, struct container);
} containers SEC(".maps");

struct process {
	u32 container_id;
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, PID_MAX_LIMIT);
	__type(key, u64);
	__type(value, struct process);
} processes SEC(".maps");

/*
 * Utils
 * =====
 *
 * Util functions.
 */

/*
 * hash - simple string hash function which allows to use strings as keys for
 * BPF maps even though they use u32 as a key type.
 * @str: string to hash
 * @len: length of the string
 *
 * Return: an u32 value representing the given string.
 */
static __always_inline u32 hash(const char *str, size_t len)
{
	u32 hash = 0;
	int i;

	for (i = 0; i < len; i++) {
		if (str[i] == '\0')
			return hash;
		hash += str[i];
	}

	return hash;
}

/*
 * task_pid - find a PID of the given process
 * @task: task to find a PID for
 *
 * Return: a PID.
 */
static __always_inline u64 task_pid(struct task_struct *task)
{
	return (u64) BPF_CORE_READ(task, tgid) << 32 | BPF_CORE_READ(task, pid);
}

/*
 * add_container - register the container and its init process in BPF maps
 * @task: the init process of the container runtime
 *
 * Return: 0 if all BPF map operations are successful, otherwise an error code.
 */
static __always_inline int add_container(struct task_struct *task)
{
	int ret;

	u32 container_id = bpf_get_prandom_u32();
	struct container container = {
		.policy_level = POLICY_LEVEL_BASELINE
	};

	ret = bpf_map_update_elem(&containers, &container_id, &container, 0);
	if (ret < 0)
		return ret;

	u64 pid = task_pid(task);
	struct process process = {
		.container_id = container_id
	};

	ret = bpf_map_update_elem(&processes, &pid, &process, 0);
	if (ret < 0)
		return ret;

	return 0;
}

/*
 * handle_process - the handler which monitors all new tasks/functions created
 * in the system and checks whether:
 * - it's an init process of a container runtime - in that case, it calls the
 *   add_container() function to register a new container
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
	u64 pid = task_pid(child);

	const char *comm = BPF_CORE_READ(child, comm);
	u32 runtime_key = hash(comm, TASK_COMM_LEN);
	u32 *runtime_lookup = bpf_map_lookup_elem(&runtimes, &runtime_key);

	if (runtime_lookup) {
		bpf_printk("found runtime init process: %d\n", pid);
		err = add_container(child);
		if (err < 0)
			return err;
		return 0;
	}

	u64 process_key = task_pid(parent);
	struct process *parent_lookup = bpf_map_lookup_elem(&processes, &process_key);
	if (!parent_lookup)
		/* Nothing to do */
		return 0;

	bpf_printk("found parent containerized process: %d\n", process_key);

	u32 container_key = parent_lookup->container_id;
	u32 *container_lookup = bpf_map_lookup_elem(&containers, &container_key);
	if (!container_lookup) {
		/* Shouldn't happen */
		bpf_printk("error: handle_new_process: cound not find a "
			   "container for a registered process\n");
		return -EPERM;
	}

	struct process new_p = {
		.container_id = container_key
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
static __always_inline enum container_policy_level get_policy_level(u64 pid)
{
	int err;

	struct process *p = bpf_map_lookup_elem(&processes, &pid);
	if (!p)
		return POLICY_LEVEL_NOT_FOUND;

	struct container *c = bpf_map_lookup_elem(&containers, &p->container_id);
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
	u64 pid = bpf_get_current_pid_tgid();
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
