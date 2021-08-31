use std::convert::TryInto;

use crate::bpfstructs;

/// Path to Pseudo-Terminal Device, needed for -it option in container
/// runtimes.
static DIR_PTS: &str = "/dev/pts";

// Storage

/// Path to image layers used by libpod (podman, cri-o).
static DIR_STORAGE_LIBPOD: &str = "/var/lib/containers/storage";
/// Path to image layers used by containerd.
static DIR_STORAGE_CONTAINERD: &str = "/var/run/container";
static DIR_STORAGE_CRI_CONTAINERD: &str = "/run/containerd/io.containerd.runtime.v1.linux";
static DIR_SANDBOXES_CRI_CONTAINERD1: &str = "/run/containerd/io.containerd.grpc.v1.cri/sandboxes";
static DIR_SANDBOXES_CRI_CONTAINERD2: &str =
    "/var/lib/containerd/io.containerd.grpc.v1.cri/sandboxes";

// cgroups

static DIR_CGROUP_MISC: &str = "/sys/fs/cgroup/misc";
static DIR_CGROUP_RDMA: &str = "/sys/fs/cgroup/rdma";
static DIR_CGROUP_BLKIO_LIBPOD: &str = "/sys/fs/cgroup/blkio/machine.slice";
static DIR_CGROUP_CPU_LIBPOD: &str = "/sys/fs/cgroup/cpu,cpuacct/machine.slice";
static DIR_CGROUP_CPUSET_LIBPOD: &str = "/sys/fs/cgroup/cpuset/machine.slice";
static DIR_CGROUP_DEVICES_LIBPOD: &str = "/sys/fs/cgroup/devices/machine.slice";
static DIR_CGROUP_FREEZER_LIBPOD: &str = "/sys/fs/cgroup/freezer/machine.slice";
static DIR_CGROUP_HUGETLB_LIBPOD: &str = "/sys/fs/cgroup/hugetlb/machine.slice";
static DIR_CGROUP_MEMORY_LIBPOD: &str = "/sys/fs/cgroup/memory/machine.slice";
static DIR_CGROUP_NET_LIBPOD: &str = "/sys/fs/cgroup/net_cls,net_prio/machine.slice";
static DIR_CGROUP_PERF_LIBPOD: &str = "/sys/fs/cgroup/perf_event/machine.slice";
static DIR_CGROUP_PIDS_LIBPOD: &str = "/sys/fs/cgroup/pids/machine.slice";
static DIR_CGROUP_SYSTEMD_LIBPOD: &str = "/sys/fs/cgroup/systemd/machine.slice";
static DIR_CGROUP_UNIFIED_LIBPOD: &str = "/sys/fs/cgroup/unified/machine.slice";
static DIR_CGROUP_BLKIO_K8S: &str = "/sys/fs/cgroup/blkio/kubepods.slice";
static DIR_CGROUP_CPU_K8S: &str = "/sys/fs/cgroup/cpu,cpuacct/kubepods.slice";
static DIR_CGROUP_CPUSET_K8S: &str = "/sys/fs/cgroup/cpuset/kubepods.slice";
static DIR_CGROUP_DEVICES_K8S: &str = "/sys/fs/cgroup/devices/kubepods.slice";
static DIR_CGROUP_FREEZER_K8S: &str = "/sys/fs/cgroup/freezer/kubepods.slice";
static DIR_CGROUP_HUGETLB_K8S: &str = "/sys/fs/cgroup/hugetlb/kubepods.slice";
static DIR_CGROUP_MEMORY_K8S: &str = "/sys/fs/cgroup/memory/kubepods.slice";
static DIR_CGROUP_NET_K8S: &str = "/sys/fs/cgroup/net_cls,net_prio/kubepods.slice";
static DIR_CGROUP_PERF_K8S: &str = "/sys/fs/cgroup/perf_event/kubepods.slice";
static DIR_CGROUP_PIDS_K8S: &str = "/sys/fs/cgroup/pids/kubepods.slice";
static DIR_CGROUP_SYSTEMD_K8S: &str = "/sys/fs/cgroup/systemd/kubepods.slice";
static DIR_CGROUP_UNIFIED_K8S: &str = "/sys/fs/cgroup/unified/kubepods.slice";

static DIR_PODS_KUBELET: &str = "/var/lib/kubelet/pods";

static DIR_HOME: &str = "/home";
static DIR_VAR_DATA: &str = "/var/data";

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub runtimes: Vec<String>,
    /// Paths which are allowed in restricted policy. These are only paths
    /// which are used by default by container runtimes, not paths mounted
    /// with the -v option.
    pub allowed_paths_restricted: Vec<String>,
    /// Paths which are allowed in baseline policy. These are both paths
    /// used by default by container runtimes and few directories which we
    /// allow to mount with -v option.
    pub allowed_paths_baseline: Vec<String>,
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
            "allowed_paths_restricted",
            vec![
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
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
                DIR_PODS_KUBELET.to_string(),
            ],
        )?;
        s.set(
            "allowed_paths_baseline",
            vec![
                // Paths used by container runtimes.
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
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
                DIR_PODS_KUBELET.to_string(),
                // Paths we allow to mount with -v option.
                DIR_HOME.to_string(),
                DIR_VAR_DATA.to_string(),
            ],
        )?;

        s.merge(config::File::with_name("/etc/lockc/lockc.toml").required(false))?;
        s.try_into()
    }
}
