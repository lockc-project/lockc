use std::convert::TryInto;

use crate::bpfstructs;

/// Path to Pseudo-Terminal Device, needed for -it option in container
/// runtimes.
static DIR_PTS: &str = "/dev/pts";

// Storage directories

/// Storage directory used by libpod (podman, cri-o).
static DIR_STORAGE_LIBPOD: &str = "/var/lib/containers/storage";
/// Storage directory used by docker (overlay2 driver).
static DIR_STORAGE_DOCKER_OVERLAY2: &str = "/var/lib/docker/overlay2";
/// Storage directory used by containerd.
static DIR_STORAGE_CONTAINERD: &str = "/var/run/container";
/// Storage directory used by CRI containerd.
static DIR_STORAGE_CRI_CONTAINERD: &str = "/run/containerd/io.containerd.runtime.v1.linux";

/// Data directory used by docker.
static DIR_DATA_DOCKER: &str = "/var/lib/docker/containers";

/// Sandbox directory used by containerd.
static DIR_SANDBOXES_CRI_CONTAINERD1: &str = "/run/containerd/io.containerd.grpc.v1.cri/sandboxes";
/// Sandbox directory used by containerd.
static DIR_SANDBOXES_CRI_CONTAINERD2: &str =
    "/var/lib/containerd/io.containerd.grpc.v1.cri/sandboxes";

// Cgroup directories

/// Misc cgroup controller.
static DIR_CGROUP_MISC: &str = "/sys/fs/cgroup/misc";
/// RDMA controller.
static DIR_CGROUP_RDMA: &str = "/sys/fs/cgroup/rdma";
/// Block I/O controller for libpod (podman, cri-o).
static DIR_CGROUP_BLKIO_LIBPOD: &str = "/sys/fs/cgroup/blkio/machine.slice";
/// CPU accounting controller for libpod (podman, cri-o).
static DIR_CGROUP_CPU_LIBPOD: &str = "/sys/fs/cgroup/cpu,cpuacct/machine.slice";
/// Cpusets for libpod (podman, cri-o).
static DIR_CGROUP_CPUSET_LIBPOD: &str = "/sys/fs/cgroup/cpuset/machine.slice";
/// Device allowlist controller for libpod (podman, cri-o).
static DIR_CGROUP_DEVICES_LIBPOD: &str = "/sys/fs/cgroup/devices/machine.slice";
/// Cgroup freezer for libpod (podman, cri-o).
static DIR_CGROUP_FREEZER_LIBPOD: &str = "/sys/fs/cgroup/freezer/machine.slice";
/// HugeTLB controller for libpod (podman, cri-o).
static DIR_CGROUP_HUGETLB_LIBPOD: &str = "/sys/fs/cgroup/hugetlb/machine.slice";
/// Memory controller for libpod (podman, cri-o).
static DIR_CGROUP_MEMORY_LIBPOD: &str = "/sys/fs/cgroup/memory/machine.slice";
/// Network classifier and priority controller for libpod (podman, cri-o).
static DIR_CGROUP_NET_LIBPOD: &str = "/sys/fs/cgroup/net_cls,net_prio/machine.slice";
/// Perf event controller for libpod (podman, cri-o).
static DIR_CGROUP_PERF_LIBPOD: &str = "/sys/fs/cgroup/perf_event/machine.slice";
/// Process number controller for libpod (podman, cri-o).
static DIR_CGROUP_PIDS_LIBPOD: &str = "/sys/fs/cgroup/pids/machine.slice";
/// Cgroup v1 hierarchy (used by systemd) for libpod (podman, cri-o).
static DIR_CGROUP_SYSTEMD_LIBPOD: &str = "/sys/fs/cgroup/systemd/machine.slice";
/// Cgroup v2 hierarchy (used by systemd) for libpod (podman, cri-o).
static DIR_CGROUP_UNIFIED_LIBPOD: &str = "/sys/fs/cgroup/unified/machine.slice";
/// Block I/O controller for kubelet.
static DIR_CGROUP_BLKIO_K8S: &str = "/sys/fs/cgroup/blkio/kubepods.slice";
/// CPU accounting controller for kubelet.
static DIR_CGROUP_CPU_K8S: &str = "/sys/fs/cgroup/cpu,cpuacct/kubepods.slice";
/// Cpusets for libpod for kubelet.
static DIR_CGROUP_CPUSET_K8S: &str = "/sys/fs/cgroup/cpuset/kubepods.slice";
/// Device allowlist controller for kubelet.
static DIR_CGROUP_DEVICES_K8S: &str = "/sys/fs/cgroup/devices/kubepods.slice";
/// Cgroup freezer for kubelet.
static DIR_CGROUP_FREEZER_K8S: &str = "/sys/fs/cgroup/freezer/kubepods.slice";
/// HugeTLB controller for kubelet.
static DIR_CGROUP_HUGETLB_K8S: &str = "/sys/fs/cgroup/hugetlb/kubepods.slice";
/// Memory controller for kubelet.
static DIR_CGROUP_MEMORY_K8S: &str = "/sys/fs/cgroup/memory/kubepods.slice";
/// Network classifier and priority controller for kubelet.
static DIR_CGROUP_NET_K8S: &str = "/sys/fs/cgroup/net_cls,net_prio/kubepods.slice";
/// Perf event controller for kubelet.
static DIR_CGROUP_PERF_K8S: &str = "/sys/fs/cgroup/perf_event/kubepods.slice";
/// Process number controller for kubelet.
static DIR_CGROUP_PIDS_K8S: &str = "/sys/fs/cgroup/pids/kubepods.slice";
/// Cgroup v1 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_SYSTEMD_K8S: &str = "/sys/fs/cgroup/systemd/kubepods.slice";
/// Cgroup v2 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_UNIFIED_K8S: &str = "/sys/fs/cgroup/unified/kubepods.slice";
/// Block I/O controller for docker.
static DIR_CGROUP_BLKIO_DOCKER: &str = "/sys/fs/cgroup/blkio/docker";
/// CPU accounting controller for docker.
static DIR_CGROUP_CPU_DOCKER: &str = "/sys/fs/cgroup/cpu,cpuacct/docker";
/// Cpusets for docker.
static DIR_CGROUP_CPUSET_DOCKER: &str = "/sys/fs/cgroup/cpuset/docker";
/// Device allowlist controller for docker.
static DIR_CGROUP_DEVICES_DOCKER: &str = "/sys/fs/cgroup/devices/docker";
/// Cgroup freezer for docker.
static DIR_CGROUP_FREEZER_DOCKER: &str = "/sys/fs/cgroup/freezer/docker";
/// HugeTLB controller for docker.
static DIR_CGROUP_HUGETLB_DOCKER: &str = "/sys/fs/cgroup/hugetlb/docker";
/// Memory controller for docker.
static DIR_CGROUP_MEMORY_DOCKER: &str = "/sys/fs/cgroup/memory/docker";
/// Network classifier and priority controller for docker.
static DIR_CGROUP_NET_DOCKER: &str = "/sys/fs/cgroup/net_cls,net_prio/docker";
/// Perf event controller for docker.
static DIR_CGROUP_PERF_DOCKER: &str = "/sys/fs/cgroup/perf_event/docker";
/// Process number controller for docker.
static DIR_CGROUP_PIDS_DOCKER: &str = "/sys/fs/cgroup/pids/docker";
/// Cgroup v1 hierarchy (used by systemd) for docker.
static DIR_CGROUP_SYSTEMD_DOCKER: &str = "/sys/fs/cgroup/systemd/docker";
/// Cgroup v2 hierarchy (used by systemd) for docker.
static DIR_CGROUP_UNIFIED_DOCKER: &str = "/sys/fs/cgroup/unified/docker";

/// State and ephemeral storage for kubelet.
static DIR_PODS_KUBELET: &str = "/var/lib/kubelet/pods";

static DIR_HOME: &str = "/home";
static DIR_VAR_DATA: &str = "/var/data";

static DIR_BIN: &str = "/bin";
static DIR_DEV_CONSOLE: &str = "/dev/console";
static DIR_DEV_FULL: &str = "/dev/full";
static DIR_DEV_NULL: &str = "/dev/null";
static DIR_DEV_PTS: &str = "/dev/pts";
static DIR_DEV_TTY: &str = "/dev/tty";
static DIR_DEV_URANDOM: &str = "/dev/urandom";
static DIR_DEV_ZERO: &str = "/dev/zero";
static DIR_ETC: &str = "/etc";
static DIR_LIB: &str = "/lib";
static DIR_PROC: &str = "/proc";
static DIR_CGROUP: &str = "/sys/fs/cgroup";
static DIR_TMP: &str = "/tmp";
static DIR_USR: &str = "/usr";
static DIR_VAR: &str = "/var";

static DIR_PROC_ACPI: &str = "/proc/acpi";
static DIR_PROC_SYS: &str = "/proc/sys";

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub runtimes: Vec<String>,
    /// Paths which are allowed in restricted policy. These are only paths
    /// which are used by default by container runtimes, not paths mounted
    /// with the -v option.
    pub allowed_paths_mount_restricted: Vec<String>,
    /// Paths which are allowed in baseline policy. These are both paths
    /// used by default by container runtimes and few directories which we
    /// allow to mount with -v option.
    pub allowed_paths_mount_baseline: Vec<String>,
    pub allowed_paths_access_restricted: Vec<String>,
    pub allowed_paths_access_baseline: Vec<String>,
    pub denied_paths_access_restricted: Vec<String>,
    pub denied_paths_access_baseline: Vec<String>,
}

fn trim_task_comm_len(mut s: std::string::String) -> std::string::String {
    s.truncate((bpfstructs::TASK_COMM_LEN - 1).try_into().unwrap());
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_task_comm_len() {
        assert_eq!(
            trim_task_comm_len("abcdefgijklmnopqrstuvwxyz".to_string()),
            "abcdefgijklmnop".to_string()
        );
        assert_eq!(trim_task_comm_len("foo".to_string()), "foo".to_string());
    }
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::default();

        s.set("runtimes", vec![trim_task_comm_len("runc".to_string())])?;
        s.set(
            "allowed_paths_mount_restricted",
            vec![
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY2.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
                DIR_DATA_DOCKER.to_string(),
                DIR_SANDBOXES_CRI_CONTAINERD1.to_string(),
                DIR_SANDBOXES_CRI_CONTAINERD2.to_string(),
                DIR_CGROUP_MISC.to_string(),
                DIR_CGROUP_RDMA.to_string(),
                DIR_CGROUP_BLKIO_LIBPOD.to_string(),
                DIR_CGROUP_CPU_LIBPOD.to_string(),
                DIR_CGROUP_CPUSET_LIBPOD.to_string(),
                DIR_CGROUP_DEVICES_LIBPOD.to_string(),
                DIR_CGROUP_FREEZER_LIBPOD.to_string(),
                DIR_CGROUP_HUGETLB_LIBPOD.to_string(),
                DIR_CGROUP_MEMORY_LIBPOD.to_string(),
                DIR_CGROUP_NET_LIBPOD.to_string(),
                DIR_CGROUP_PERF_LIBPOD.to_string(),
                DIR_CGROUP_PIDS_LIBPOD.to_string(),
                DIR_CGROUP_SYSTEMD_LIBPOD.to_string(),
                DIR_CGROUP_UNIFIED_LIBPOD.to_string(),
                DIR_CGROUP_BLKIO_K8S.to_string(),
                DIR_CGROUP_CPU_K8S.to_string(),
                DIR_CGROUP_CPUSET_K8S.to_string(),
                DIR_CGROUP_DEVICES_K8S.to_string(),
                DIR_CGROUP_FREEZER_K8S.to_string(),
                DIR_CGROUP_HUGETLB_K8S.to_string(),
                DIR_CGROUP_MEMORY_K8S.to_string(),
                DIR_CGROUP_NET_K8S.to_string(),
                DIR_CGROUP_PERF_K8S.to_string(),
                DIR_CGROUP_PIDS_K8S.to_string(),
                DIR_CGROUP_SYSTEMD_K8S.to_string(),
                DIR_CGROUP_UNIFIED_K8S.to_string(),
                DIR_CGROUP_BLKIO_DOCKER.to_string(),
                DIR_CGROUP_CPU_DOCKER.to_string(),
                DIR_CGROUP_CPUSET_DOCKER.to_string(),
                DIR_CGROUP_DEVICES_DOCKER.to_string(),
                DIR_CGROUP_FREEZER_DOCKER.to_string(),
                DIR_CGROUP_HUGETLB_DOCKER.to_string(),
                DIR_CGROUP_MEMORY_DOCKER.to_string(),
                DIR_CGROUP_NET_DOCKER.to_string(),
                DIR_CGROUP_PERF_DOCKER.to_string(),
                DIR_CGROUP_PIDS_DOCKER.to_string(),
                DIR_CGROUP_SYSTEMD_DOCKER.to_string(),
                DIR_CGROUP_UNIFIED_DOCKER.to_string(),
                DIR_PODS_KUBELET.to_string(),
            ],
        )?;
        s.set(
            "allowed_paths_mount_baseline",
            vec![
                // Paths used by container runtimes.
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY2.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
                DIR_DATA_DOCKER.to_string(),
                DIR_SANDBOXES_CRI_CONTAINERD1.to_string(),
                DIR_SANDBOXES_CRI_CONTAINERD2.to_string(),
                DIR_CGROUP_MISC.to_string(),
                DIR_CGROUP_RDMA.to_string(),
                DIR_CGROUP_BLKIO_LIBPOD.to_string(),
                DIR_CGROUP_CPU_LIBPOD.to_string(),
                DIR_CGROUP_CPUSET_LIBPOD.to_string(),
                DIR_CGROUP_DEVICES_LIBPOD.to_string(),
                DIR_CGROUP_FREEZER_LIBPOD.to_string(),
                DIR_CGROUP_HUGETLB_LIBPOD.to_string(),
                DIR_CGROUP_MEMORY_LIBPOD.to_string(),
                DIR_CGROUP_NET_LIBPOD.to_string(),
                DIR_CGROUP_PERF_LIBPOD.to_string(),
                DIR_CGROUP_PIDS_LIBPOD.to_string(),
                DIR_CGROUP_SYSTEMD_LIBPOD.to_string(),
                DIR_CGROUP_UNIFIED_LIBPOD.to_string(),
                DIR_CGROUP_BLKIO_K8S.to_string(),
                DIR_CGROUP_CPU_K8S.to_string(),
                DIR_CGROUP_CPUSET_K8S.to_string(),
                DIR_CGROUP_DEVICES_K8S.to_string(),
                DIR_CGROUP_FREEZER_K8S.to_string(),
                DIR_CGROUP_HUGETLB_K8S.to_string(),
                DIR_CGROUP_MEMORY_K8S.to_string(),
                DIR_CGROUP_NET_K8S.to_string(),
                DIR_CGROUP_PERF_K8S.to_string(),
                DIR_CGROUP_PIDS_K8S.to_string(),
                DIR_CGROUP_SYSTEMD_K8S.to_string(),
                DIR_CGROUP_UNIFIED_K8S.to_string(),
                DIR_CGROUP_BLKIO_DOCKER.to_string(),
                DIR_CGROUP_CPU_DOCKER.to_string(),
                DIR_CGROUP_CPUSET_DOCKER.to_string(),
                DIR_CGROUP_DEVICES_DOCKER.to_string(),
                DIR_CGROUP_FREEZER_DOCKER.to_string(),
                DIR_CGROUP_HUGETLB_DOCKER.to_string(),
                DIR_CGROUP_MEMORY_DOCKER.to_string(),
                DIR_CGROUP_NET_DOCKER.to_string(),
                DIR_CGROUP_PERF_DOCKER.to_string(),
                DIR_CGROUP_PIDS_DOCKER.to_string(),
                DIR_CGROUP_SYSTEMD_DOCKER.to_string(),
                DIR_CGROUP_UNIFIED_DOCKER.to_string(),
                DIR_PODS_KUBELET.to_string(),
                // Paths we allow to mount with -v option.
                DIR_HOME.to_string(),
                DIR_VAR_DATA.to_string(),
            ],
        )?;
        s.set(
            "allowed_paths_access_restricted",
            vec![
                DIR_BIN.to_string(),
                DIR_DEV_CONSOLE.to_string(),
                DIR_DEV_FULL.to_string(),
                DIR_DEV_NULL.to_string(),
                DIR_DEV_PTS.to_string(),
                DIR_DEV_TTY.to_string(),
                DIR_DEV_URANDOM.to_string(),
                DIR_DEV_ZERO.to_string(),
                DIR_ETC.to_string(),
                DIR_HOME.to_string(),
                DIR_LIB.to_string(),
                DIR_PROC.to_string(),
                DIR_CGROUP.to_string(),
                DIR_TMP.to_string(),
                DIR_USR.to_string(),
                DIR_VAR.to_string(),
            ],
        )?;
        s.set(
            "allowed_paths_access_baseline",
            vec![
                DIR_BIN.to_string(),
                DIR_DEV_CONSOLE.to_string(),
                DIR_DEV_FULL.to_string(),
                DIR_DEV_NULL.to_string(),
                DIR_DEV_PTS.to_string(),
                DIR_DEV_TTY.to_string(),
                DIR_DEV_URANDOM.to_string(),
                DIR_DEV_ZERO.to_string(),
                DIR_ETC.to_string(),
                DIR_HOME.to_string(),
                DIR_LIB.to_string(),
                DIR_PROC.to_string(),
                DIR_CGROUP.to_string(),
                DIR_TMP.to_string(),
                DIR_USR.to_string(),
                DIR_VAR.to_string(),
            ],
        )?;
        s.set(
            "denied_paths_access_restricted",
            vec![DIR_PROC_ACPI.to_string()],
        )?;
        s.set(
            "denied_paths_access_baseline",
            vec![DIR_PROC_ACPI.to_string(), DIR_PROC_SYS.to_string()],
        )?;

        s.merge(config::File::with_name("/etc/lockc/lockc.toml").required(false))?;
        s.try_into()
    }
}
