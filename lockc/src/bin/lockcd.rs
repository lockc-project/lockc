use std::path;

use chrono::prelude::*;

fn main() -> anyhow::Result<()> {
    let sys_lsm_path = path::Path::new("/sys")
        .join("kernel")
        .join("security")
        .join("lsm");
    lockc::check_bpf_lsm_enabled(sys_lsm_path)?;

    let now = Utc::now();
    let dirname = now.format("%s").to_string();
    let path_base = std::path::Path::new("/sys")
        .join("fs")
        .join("bpf")
        .join("lockc");

    std::fs::create_dir_all(&path_base)?;

    let path_base_ts = path_base.join(&dirname);

    lockc::load_programs(path_base_ts)?;
    lockc::cleanup(path_base, &dirname)?;

    Ok(())
}
