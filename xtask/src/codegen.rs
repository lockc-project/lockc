use std::{fs::File, io::Write, path::PathBuf};

use aya_tool::generate::InputFile;

pub fn generate() -> Result<(), anyhow::Error> {
    let dir = PathBuf::from("lockc-ebpf/src");
    let names: Vec<&str> = vec!["cred", "file", "sock", "sock_common", "task_struct"];
    let bindings = aya_tool::generate(
        InputFile::Btf(PathBuf::from("/sys/kernel/btf/vmlinux")),
        &names,
        &[],
    )?;
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let mut out = File::create(dir.join("vmlinux.rs"))?;
    write!(out, "{}", bindings)?;
    Ok(())
}
