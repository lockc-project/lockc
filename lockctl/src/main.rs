use std::str::FromStr;

use aya::{
    include_bytes_aligned,
    maps::{HashMap, MapRef, MapRefMut},
    Bpf, BpfLoader,
};
use clap::{Parser, Subcommand};
use cli_table::{print_stdout, Cell, Style, Table};
use lockc_common::{Container, ContainerID, ContainerPolicyLevel, Process};

const PATH_BASE: &str = "/sys/fs/bpf/lockc";

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    subcommand: Sub,
}

#[derive(Subcommand)]
enum Sub {
    /// Manage containers and their policies.
    Container {
        #[command(subcommand)]
        container: SubContainer,
    },
    /// Manage containerized processes.
    Process {
        #[command(subcommand)]
        process: SubProcess,
    },
}

#[derive(Subcommand)]
enum SubContainer {
    /// List all containers.
    List,
    ApplyPolicy {
        /// The ID of the container.
        container_id: String,
        /// The policy to apply.
        #[clap(value_enum)]
        policy: ContainerPolicyLevel,
    },
}

#[derive(Subcommand)]
enum SubProcess {
    /// List all processes.
    List,
}

fn load_bpf() -> anyhow::Result<Bpf> {
    #[cfg(debug_assertions)]
    let bpf = BpfLoader::new()
        .map_pin_path(PATH_BASE)
        .load(include_bytes_aligned!(
            "../../target/bpfel-unknown-none/debug/lockc"
        ))?;
    #[cfg(not(debug_assertions))]
    let bpf = BpfLoader::new()
        .map_pin_path(PATH_BASE)
        .load(include_bytes_aligned!(
            "../../target/bpfel-unknown-none/release/lockc"
        ))?;

    Ok(bpf)
}

fn container_list() -> anyhow::Result<()> {
    let bpf = load_bpf()?;

    let containers: HashMap<MapRef, ContainerID, Container> = bpf.map("CONTAINERS")?.try_into()?;
    let mut table = Vec::new();
    for res in containers.iter() {
        let (container_id, container) = res?;
        table.push(vec![
            container_id.as_str()?.to_string().cell(),
            format!("{}", container.policy_level).cell(),
        ]);
    }

    let table = table.table().title(vec![
        "Container ID".cell().bold(true),
        "Policy Level".cell().bold(true),
    ]);

    print_stdout(table)?;

    Ok(())
}

fn container_apply_policy(
    container_id: String,
    policy: ContainerPolicyLevel,
) -> anyhow::Result<()> {
    let bpf = load_bpf()?;

    let mut containers: HashMap<MapRefMut, ContainerID, Container> =
        bpf.map_mut("CONTAINERS")?.try_into()?;

    let key = ContainerID::from_str(&container_id)?;
    if containers.get(&key, 0).is_err() {
        return Err(anyhow::anyhow!("container {} not found", container_id));
    }

    let container = Container {
        policy_level: policy,
    };
    containers.remove(&key)?;
    containers.insert(key, container, 0)?;

    Ok(())
}

fn process_list() -> anyhow::Result<()> {
    let bpf = load_bpf()?;

    let processes: HashMap<MapRef, i32, Process> = bpf.map("PROCESSES")?.try_into()?;
    let containers: HashMap<MapRef, ContainerID, Container> = bpf.map("CONTAINERS")?.try_into()?;
    let mut table = Vec::new();
    for res in processes.iter() {
        let (pid, process) = res?;
        let (stat, running) = match procfs::process::Process::new(pid) {
            Ok(stat) => (Some(stat), true),
            Err(_) => (None, false),
        };
        let exe = match stat {
            Some(stat) => stat.exe()?.to_string_lossy().to_string(),
            None => "-".to_owned(),
        };
        let container = containers.get(&process.container_id, 0)?;
        table.push(vec![
            pid.to_string().cell(),
            format!("{}", running).cell(),
            exe.cell(),
            process.container_id.as_str()?.to_string().cell(),
            format!("{}", container.policy_level).cell(),
        ]);
    }

    let table = table.table().title(vec![
        "PID".cell().bold(true),
        "Running".cell().bold(true),
        "Command".cell().bold(true),
        "Container ID".cell().bold(true),
        "Policy Level".cell().bold(true),
    ]);

    print_stdout(table)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.subcommand {
        Sub::Container { container } => match container {
            SubContainer::List => container_list()?,
            SubContainer::ApplyPolicy {
                container_id,
                policy,
            } => container_apply_policy(container_id, policy)?,
        },
        Sub::Process { process } => match process {
            SubProcess::List => process_list()?,
        },
    }

    Ok(())
}
