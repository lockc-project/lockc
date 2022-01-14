use lazy_static::lazy_static;

use crate::bpfstructs;

/// Path to Pseudo-Terminal Device, needed for -it option in container
/// runtimes.
static DIR_PTS: &str = "/dev/pts";

// Storage directories

/// Storage directory used by libpod (podman, cri-o).
static DIR_STORAGE_LIBPOD: &str = "/var/lib/containers/storage";
/// Storage directory used by docker (aufs driver).
static DIR_STORAGE_DOCKER_AUFS: &str = "/var/lib/docker/aufs";
/// Storage directory used by docker (btrfs driver).
static DIR_STORAGE_DOCKER_BTRFS: &str = "/var/lib/docker/btrfs";
/// Storage directory used by docker (devmapper driver).
static DIR_STORAGE_DOCKER_DEVMAPPER: &str = "/var/lib/docker/devmapper";
/// Storage directory used by docker (overlay driver).
static DIR_STORAGE_DOCKER_OVERLAY: &str = "/var/lib/docker/overlay";
/// Storage directory used by docker (overlay2 driver).
static DIR_STORAGE_DOCKER_OVERLAY2: &str = "/var/lib/docker/overlay2";
/// Storage directory used by docker (vfs driver).
static DIR_STORAGE_DOCKER_VFS: &str = "/var/lib/docker/vfs";
/// Storage directory used by docker (zfs driver).
static DIR_STORAGE_DOCKER_ZFS: &str = "/var/lib/docker/zfs";
/// Storage directory used by containerd.
static DIR_STORAGE_CONTAINERD: &str = "/var/run/container";
/// Storage directory used by CRI containerd.
static DIR_STORAGE_CRI_CONTAINERD: &str = "/run/containerd/io.containerd.runtime.v1.linux";
/// Storage directory used by CRI containerd.
static DIR_STORAGE_CRI_CONTAINERD2: &str = "/run/containerd/io.containerd.runtime.v2.task";

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
static DIR_CGROUP_BLKIO_K8S1: &str = "/sys/fs/cgroup/blkio/kubepods.slice";
/// CPU accounting controller for kubelet.
static DIR_CGROUP_CPU_K8S1: &str = "/sys/fs/cgroup/cpu,cpuacct/kubepods.slice";
/// Cpusets for libpod for kubelet.
static DIR_CGROUP_CPUSET_K8S1: &str = "/sys/fs/cgroup/cpuset/kubepods.slice";
/// Device allowlist controller for kubelet.
static DIR_CGROUP_DEVICES_K8S1: &str = "/sys/fs/cgroup/devices/kubepods.slice";
/// Cgroup freezer for kubelet.
static DIR_CGROUP_FREEZER_K8S1: &str = "/sys/fs/cgroup/freezer/kubepods.slice";
/// HugeTLB controller for kubelet.
static DIR_CGROUP_HUGETLB_K8S1: &str = "/sys/fs/cgroup/hugetlb/kubepods.slice";
/// Memory controller for kubelet.
static DIR_CGROUP_MEMORY_K8S1: &str = "/sys/fs/cgroup/memory/kubepods.slice";
/// Network classifier and priority controller for kubelet.
static DIR_CGROUP_NET_K8S1: &str = "/sys/fs/cgroup/net_cls,net_prio/kubepods.slice";
/// Perf event controller for kubelet.
static DIR_CGROUP_PERF_K8S1: &str = "/sys/fs/cgroup/perf_event/kubepods.slice";
/// Process number controller for kubelet.
static DIR_CGROUP_PIDS_K8S1: &str = "/sys/fs/cgroup/pids/kubepods.slice";
/// Cgroup v1 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_SYSTEMD_K8S1: &str = "/sys/fs/cgroup/systemd/kubepods.slice";
/// Cgroup v2 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_UNIFIED_K8S1: &str = "/sys/fs/cgroup/unified/kubepods.slice";
/// Block I/O controller for kubelet.
static DIR_CGROUP_BLKIO_K8S2: &str = "/sys/fs/cgroup/blkio/kubepods-besteffort";
/// CPU accounting controller for kubelet.
static DIR_CGROUP_CPU_K8S2: &str = "/sys/fs/cgroup/cpu,cpuacct/kubepods-besteffort";
/// Cpusets for libpod for kubelet.
static DIR_CGROUP_CPUSET_K8S2: &str = "/sys/fs/cgroup/cpuset/kubepods-besteffort";
/// Device allowlist controller for kubelet.
static DIR_CGROUP_DEVICES_K8S2: &str = "/sys/fs/cgroup/devices/kubepods-besteffort";
/// Cgroup freezer for kubelet.
static DIR_CGROUP_FREEZER_K8S2: &str = "/sys/fs/cgroup/freezer/kubepods-besteffort";
/// HugeTLB controller for kubelet.
static DIR_CGROUP_HUGETLB_K8S2: &str = "/sys/fs/cgroup/hugetlb/kubepods-besteffort";
/// Memory controller for kubelet.
static DIR_CGROUP_MEMORY_K8S2: &str = "/sys/fs/cgroup/memory/kubepods-besteffort";
/// Network classifier and priority controller for kubelet.
static DIR_CGROUP_NET_K8S2: &str = "/sys/fs/cgroup/net_cls,net_prio/kubepods-besteffort";
/// Perf event controller for kubelet.
static DIR_CGROUP_PERF_K8S2: &str = "/sys/fs/cgroup/perf_event/kubepods-besteffort";
/// Process number controller for kubelet.
static DIR_CGROUP_PIDS_K8S2: &str = "/sys/fs/cgroup/pids/kubepods-besteffort";
/// Cgroup v1 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_SYSTEMD_K8S2: &str = "/sys/fs/cgroup/systemd/kubepods-besteffort";
/// Cgroup v2 hierarchy (used by systemd) for kubelet.
static DIR_CGROUP_UNIFIED_K8S2: &str = "/sys/fs/cgroup/unified/kubepods-besteffort";
/// Block I/O controller for containerd.
static DIR_CGROUP_BLKIO_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/blkio/system.slice/containerd.service";
/// CPU accounting controller for containerd.
static DIR_CGROUP_CPU_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/cpu,cpuacct/system.slice/containerd.service";
/// Cpusets for libpod for containerd.
static DIR_CGROUP_CPUSET_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/cpuset/system.slice/containerd.service";
/// Device allowlist controller for containerd.
static DIR_CGROUP_DEVICES_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/devices/system.slice/containerd.service";
/// Cgroup freezer for containerd.
static DIR_CGROUP_FREEZER_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/freezer/system.slice/containerd.service";
/// HugeTLB controller for containerd.
static DIR_CGROUP_HUGETLB_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/hugetlb/system.slice/containerd.service";
/// Memory controller for containerd.
static DIR_CGROUP_MEMORY_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/memory/system.slice/containerd.service";
/// Network classifier and priority controller for containerd.
static DIR_CGROUP_NET_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/net_cls,net_prio/system.slice/containerd.service";
/// Perf event controller for containerd.
static DIR_CGROUP_PERF_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/perf_event/system.slice/containerd.service";
/// Process number controller for containerd.
static DIR_CGROUP_PIDS_CONTAINERD_K8S: &str = "/sys/fs/cgroup/pids/system.slice/containerd.service";
/// Cgroup v1 hierarchy (used by systemd) for containerd.
static DIR_CGROUP_SYSTEMD_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/systemd/system.slice/containerd.service";
/// Cgroup v2 hierarchy (used by systemd) for containerd.
static DIR_CGROUP_UNIFIED_CONTAINERD_K8S: &str =
    "/sys/fs/cgroup/unified/system.slice/containerd.service";
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

/// Cgroup file, i.e. cgroup:[4026531835]
static GROUP: &str = "cgroup:";
/// IPC namespace file, i.e. ipc:[4026531839]
static NS_IPC: &str = "ipc:";
/// Mount namespace file, i.e. mnt:[4026531840]
static NS_MNT: &str = "mnt:";
/// Network namespace file, i.e. net:[4026531992]
static NS_NET: &str = "net:";
/// PID namespace file, i.e. pid:[4026531836]
static NS_PID: &str = "pid:";
/// Pipe
static PIPE: &str = "pipe:";
/// Time namespace file. i.e. time:[4026531834]
static NS_TIME: &str = "time:";
/// User namespace file, i.e. user:[4026531837]
static NS_USER: &str = "user:";
/// UTS namespace file, i.e. uts:[4026531838]
static NS_UTS: &str = "uts:";
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
static DIR_LIB64: &str = "/lib64";
static PAUSE: &str = "/pause";
static DIR_OPT: &str = "/opt";
static DIR_PROC: &str = "/proc";
static DIR_RUN: &str = "/run";
static DIR_CGROUP: &str = "/sys/fs/cgroup";
static DIR_MM: &str = "/sys/kernel/mm";
static DIR_TMP: &str = "/tmp";
static DIR_USR: &str = "/usr";
static DIR_VAR: &str = "/var";
static DIR_K8S_SECRETS: &str = "/var/run/secrets/kubernetes.io";

static DIR_PROC_ACPI: &str = "/proc/acpi";
static DIR_PROC_SYS: &str = "/proc/sys";

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

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

        s.set_default("runtimes", vec![trim_task_comm_len("runc".to_string())])?;
        s.set_default(
            "allowed_paths_mount_restricted",
            vec![
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_DOCKER_AUFS.to_string(),
                DIR_STORAGE_DOCKER_BTRFS.to_string(),
                DIR_STORAGE_DOCKER_DEVMAPPER.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY2.to_string(),
                DIR_STORAGE_DOCKER_VFS.to_string(),
                DIR_STORAGE_DOCKER_ZFS.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD2.to_string(),
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
                DIR_CGROUP_BLKIO_K8S1.to_string(),
                DIR_CGROUP_CPU_K8S1.to_string(),
                DIR_CGROUP_CPUSET_K8S1.to_string(),
                DIR_CGROUP_DEVICES_K8S1.to_string(),
                DIR_CGROUP_FREEZER_K8S1.to_string(),
                DIR_CGROUP_HUGETLB_K8S1.to_string(),
                DIR_CGROUP_MEMORY_K8S1.to_string(),
                DIR_CGROUP_NET_K8S1.to_string(),
                DIR_CGROUP_PERF_K8S1.to_string(),
                DIR_CGROUP_PIDS_K8S1.to_string(),
                DIR_CGROUP_SYSTEMD_K8S1.to_string(),
                DIR_CGROUP_UNIFIED_K8S1.to_string(),
                DIR_CGROUP_BLKIO_K8S2.to_string(),
                DIR_CGROUP_CPU_K8S2.to_string(),
                DIR_CGROUP_CPUSET_K8S2.to_string(),
                DIR_CGROUP_DEVICES_K8S2.to_string(),
                DIR_CGROUP_FREEZER_K8S2.to_string(),
                DIR_CGROUP_HUGETLB_K8S2.to_string(),
                DIR_CGROUP_MEMORY_K8S2.to_string(),
                DIR_CGROUP_NET_K8S2.to_string(),
                DIR_CGROUP_PERF_K8S2.to_string(),
                DIR_CGROUP_PIDS_K8S2.to_string(),
                DIR_CGROUP_SYSTEMD_K8S2.to_string(),
                DIR_CGROUP_UNIFIED_K8S2.to_string(),
                DIR_CGROUP_BLKIO_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_CPU_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_CPUSET_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_DEVICES_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_FREEZER_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_HUGETLB_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_MEMORY_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_NET_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_PERF_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_PIDS_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_SYSTEMD_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_UNIFIED_CONTAINERD_K8S.to_string(),
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
        s.set_default(
            "allowed_paths_mount_baseline",
            vec![
                // Paths used by container runtimes.
                DIR_PTS.to_string(),
                DIR_STORAGE_LIBPOD.to_string(),
                DIR_STORAGE_DOCKER_AUFS.to_string(),
                DIR_STORAGE_DOCKER_BTRFS.to_string(),
                DIR_STORAGE_DOCKER_DEVMAPPER.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY.to_string(),
                DIR_STORAGE_DOCKER_OVERLAY2.to_string(),
                DIR_STORAGE_DOCKER_VFS.to_string(),
                DIR_STORAGE_DOCKER_ZFS.to_string(),
                DIR_STORAGE_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD.to_string(),
                DIR_STORAGE_CRI_CONTAINERD2.to_string(),
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
                DIR_CGROUP_BLKIO_K8S1.to_string(),
                DIR_CGROUP_CPU_K8S1.to_string(),
                DIR_CGROUP_CPUSET_K8S1.to_string(),
                DIR_CGROUP_DEVICES_K8S1.to_string(),
                DIR_CGROUP_FREEZER_K8S1.to_string(),
                DIR_CGROUP_HUGETLB_K8S1.to_string(),
                DIR_CGROUP_MEMORY_K8S1.to_string(),
                DIR_CGROUP_NET_K8S1.to_string(),
                DIR_CGROUP_PERF_K8S1.to_string(),
                DIR_CGROUP_PIDS_K8S1.to_string(),
                DIR_CGROUP_SYSTEMD_K8S1.to_string(),
                DIR_CGROUP_UNIFIED_K8S1.to_string(),
                DIR_CGROUP_BLKIO_K8S2.to_string(),
                DIR_CGROUP_CPU_K8S2.to_string(),
                DIR_CGROUP_CPUSET_K8S2.to_string(),
                DIR_CGROUP_DEVICES_K8S2.to_string(),
                DIR_CGROUP_FREEZER_K8S2.to_string(),
                DIR_CGROUP_HUGETLB_K8S2.to_string(),
                DIR_CGROUP_MEMORY_K8S2.to_string(),
                DIR_CGROUP_NET_K8S2.to_string(),
                DIR_CGROUP_PERF_K8S2.to_string(),
                DIR_CGROUP_PIDS_K8S2.to_string(),
                DIR_CGROUP_SYSTEMD_K8S2.to_string(),
                DIR_CGROUP_UNIFIED_K8S2.to_string(),
                DIR_CGROUP_BLKIO_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_CPU_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_CPUSET_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_DEVICES_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_FREEZER_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_HUGETLB_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_MEMORY_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_NET_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_PERF_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_PIDS_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_SYSTEMD_CONTAINERD_K8S.to_string(),
                DIR_CGROUP_UNIFIED_CONTAINERD_K8S.to_string(),
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
        s.set_default(
            "allowed_paths_access_restricted",
            vec![
                GROUP.to_string(),
                NS_IPC.to_string(),
                NS_MNT.to_string(),
                NS_NET.to_string(),
                NS_PID.to_string(),
                PIPE.to_string(),
                NS_TIME.to_string(),
                NS_USER.to_string(),
                NS_UTS.to_string(),
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
                DIR_LIB64.to_string(),
                PAUSE.to_string(),
                DIR_OPT.to_string(),
                DIR_PROC.to_string(),
                DIR_RUN.to_string(),
                DIR_CGROUP.to_string(),
                DIR_MM.to_string(),
                DIR_TMP.to_string(),
                DIR_USR.to_string(),
                DIR_VAR.to_string(),
            ],
        )?;
        s.set_default(
            "allowed_paths_access_baseline",
            vec![
                GROUP.to_string(),
                NS_IPC.to_string(),
                NS_MNT.to_string(),
                NS_NET.to_string(),
                NS_PID.to_string(),
                PIPE.to_string(),
                NS_TIME.to_string(),
                NS_USER.to_string(),
                NS_UTS.to_string(),
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
                DIR_LIB64.to_string(),
                PAUSE.to_string(),
                DIR_OPT.to_string(),
                DIR_PROC.to_string(),
                DIR_RUN.to_string(),
                DIR_CGROUP.to_string(),
                DIR_MM.to_string(),
                DIR_TMP.to_string(),
                DIR_USR.to_string(),
                DIR_VAR.to_string(),
            ],
        )?;
        s.set_default(
            "denied_paths_access_restricted",
            vec![
                DIR_PROC_ACPI.to_string(),
                DIR_PROC_SYS.to_string(),
                DIR_K8S_SECRETS.to_string(),
            ],
        )?;
        s.set_default(
            "denied_paths_access_baseline",
            vec![DIR_PROC_ACPI.to_string(), DIR_K8S_SECRETS.to_string()],
        )?;

        s.merge(config::File::with_name("/etc/lockc/lockc.toml").required(false))?;
        s.try_into()
    }
}
