use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    string::String,
};

use anyhow::{bail, Context, Result};

static CLANG_DEFAULT: &str = "/usr/bin/clang";

static HEADER_COMPILER: &str = "src/bpf/compiler.h";
static HEADER_LIMITS: &str = "src/bpf/limits.h";
static HEADER_MAP_STRUCTS: &str = "src/bpf/map_structs.h";
static HEADER_MAPS: &str = "src/bpf/maps.h";
static HEADER_POLICY: &str = "src/bpf/policy.h";
static HEADER_STRUTILS: &str = "src/bpf/strutils.h";
static MODULE_BPF: &str = "src/bpf/lockc.bpf.c";

static VMLINUX_URL: &str =
    "https://raw.githubusercontent.com/libbpf/libbpf-bootstrap/master/vmlinux/x86/vmlinux_508.h";

/// Downloads vmlinux.h from github (which is generated from 5.8 kernel).
fn download_vmlinux(mut f: fs::File) -> Result<()> {
    let mut res = reqwest::blocking::get(VMLINUX_URL)?;
    io::copy(&mut res, &mut f)?;

    Ok(())
}

/// Tries to generate the vmlinux.h header. If not possible, it gets downloaded
/// (generated from 5.8 kernel).
fn generate_vmlinux<P: AsRef<Path>>(include_path: P) -> Result<()> {
    let vmlinux_path = include_path.as_ref().join("vmlinux.h");
    let mut f = fs::File::create(vmlinux_path)?;
    match Command::new("bpftool")
        .arg("btf")
        .arg("dump")
        .arg("file")
        .arg("/sys/kernel/btf/vmlinux")
        .arg("format")
        .arg("c")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                f.write_all(&output.stdout)?;
            } else {
                download_vmlinux(f)?;
            }
        }
        Err(_) => download_vmlinux(f)?,
    };

    Ok(())
}

/// Extract vendored libbpf headers from libbpf-sys.
fn extract_libbpf_headers<P: AsRef<Path>>(include_path: P) -> Result<()> {
    let dir = include_path.as_ref().join("bpf");
    fs::create_dir_all(&dir)?;
    for (filename, contents) in libbpf_sys::API_HEADERS.iter() {
        let path = dir.as_path().join(filename);
        let mut file = OpenOptions::new().write(true).create(true).open(path)?;
        file.write_all(contents.as_bytes())?;
    }

    Ok(())
}

/// Build eBPF programs with clang and libbpf headers.
fn build_ebpf<P: Clone + AsRef<Path>>(out_path: P, include_path: P) -> Result<()> {
    println!("cargo:rerun-if-changed={}", HEADER_COMPILER);
    println!("cargo:rerun-if-changed={}", HEADER_LIMITS);
    println!("cargo:rerun-if-changed={}", HEADER_MAP_STRUCTS);
    println!("cargo:rerun-if-changed={}", HEADER_MAPS);
    println!("cargo:rerun-if-changed={}", HEADER_POLICY);
    println!("cargo:rerun-if-changed={}", HEADER_STRUTILS);
    println!("cargo:rerun-if-changed={}", MODULE_BPF);

    extract_libbpf_headers(&include_path)?;

    let bpf_dir = Path::new("src").join("bpf");
    let src = bpf_dir.join("lockc.bpf.c");

    let out = out_path.as_ref().join("lockc.bpf.o");

    let clang = match env::var("CLANG") {
        Ok(val) => val,
        Err(_) => String::from(CLANG_DEFAULT),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86",
        "aarch64" => "arm64",
        _ => std::env::consts::ARCH,
    };
    let mut cmd = Command::new(clang);
    cmd.arg(format!("-I{}", include_path.as_ref().to_string_lossy()))
        .arg("-g")
        .arg("-O2")
        .arg("-target")
        .arg("bpf")
        .arg("-c")
        .arg(format!("-D__TARGET_ARCH_{}", arch))
        .arg(src.as_os_str())
        .arg("-o")
        .arg(out);

    let output = cmd.output().context("Failed to execute clang")?;
    if !output.status.success() {
        bail!(
            "Failed to compile eBPF programs\n \
            stdout=\n \
            {}\n \
            stderr=\n \
            {}\n",
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }

    Ok(())
}

/// Generate Rust FFI bindings to structs used in eBPF programs, so they can be
/// reused in Rust code as well.
fn generate_bindings<P: AsRef<Path>>(out_path: P) -> Result<()> {
    println!("cargo:rerun-if-changed={}", HEADER_MAP_STRUCTS);

    let bindings = bindgen::Builder::default()
        .header(HEADER_MAP_STRUCTS)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    bindings.write_to_file(out_path.as_ref().join("bindings.rs"))?;

    Ok(())
}

fn main() -> Result<()> {
    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    let include_path = out_path.join("include");
    fs::create_dir_all(&include_path)?;

    generate_vmlinux(&include_path)?;
    build_ebpf(&out_path, &include_path)?;
    generate_bindings(&out_path)?;

    Ok(())
}
