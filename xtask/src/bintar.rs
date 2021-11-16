use std::{fs::File, path::Path};

use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use scopeguard::guard;
use structopt::StructOpt;
use tempfile::tempdir;

use crate::install;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(default_value = "debug", long)]
    profile: String,

    #[structopt(default_value = "usr/local", long)]
    prefix: String,

    // Directories which belong under prefix.
    #[structopt(default_value = "bin", long)]
    bindir: String,
    #[structopt(default_value = "etc", long)]
    sysconfdir: String,
    #[structopt(default_value = "lib/systemd/system", long)]
    unitdir: String,
}

pub struct BinTar {
    opts: Options,
}

impl BinTar {
    pub fn new(opts: Options) -> BinTar {
        BinTar { opts }
    }

    pub fn do_bin_tar(&self) -> Result<()> {
        let dir = guard(tempdir()?, |d| {
            // Ensure the dir is deleted.
            d.close().unwrap();
        });
        install::Installer::new(install::Options {
            profile: self.opts.profile.clone(),
            destdir: dir.path().to_string_lossy().to_string(),
            prefix: self.opts.prefix.clone(),
            bindir: self.opts.bindir.clone(),
            sysconfdir: self.opts.sysconfdir.clone(),
            unitdir: self.opts.unitdir.clone(),
        })
        .do_install()?;

        let tar_gz_path = Path::new("target")
            .join(self.opts.profile.clone())
            .join("lockc.tar.gz");
        let tar_gz = File::create(tar_gz_path.clone())?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all("", dir.path())?;

        println!("Tarball created: {}", tar_gz_path.display());
        Ok(())
    }
}
