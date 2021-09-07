use std::{
    env, path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

use chrono::prelude::*;

fn main() -> anyhow::Result<()> {
    let check_lsm = match env::var("LOCKC_CHECK_LSM_SKIP") {
        Ok(_) => false,
        Err(_) => true,
    };
    if check_lsm {
        let sys_lsm_path = path::Path::new("/sys")
            .join("kernel")
            .join("security")
            .join("lsm");
        lockc::check_bpf_lsm_enabled(sys_lsm_path)?;
    }

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

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    while running.load(Ordering::SeqCst) {
        eprint!(".");
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}
