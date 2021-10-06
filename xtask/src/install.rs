use std::{
    ffi::OsStr,
    fs,
    io::{self, prelude::*},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path,
};

use serde::Serialize;
use structopt::StructOpt;
use tera::{Context, Tera};
use thiserror::Error;

#[derive(Error, Debug)]
enum EscalateIfNotOwnedError {
    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error("could not escalate privileges (sudo)")]
    SudoError,
}

fn mkdir_if_not_exists(p: path::PathBuf) -> Result<(), io::Error> {
    if !p.exists() {
        fs::create_dir_all(p)?;
    }

    Ok(())
}

fn escalate_if_not_owned(p: path::PathBuf) -> Result<(), EscalateIfNotOwnedError> {
    if p.metadata()?.uid() == 0 {
        match sudo::escalate_if_needed() {
            Ok(_) => {}
            Err(_) => {
                // sudo library always returns the std::error::Error. Not
                // really descriptive for users... Replace it with our own
                // error.
                return Err(EscalateIfNotOwnedError::SudoError);
            }
        }
    }
    Ok(())
}

#[derive(StructOpt)]
pub struct Options {
    #[structopt(default_value = "debug", long)]
    profile: String,

    #[structopt(default_value = "/", long)]
    destdir: String,

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

impl Options {
    fn full_destdir(&self) -> path::PathBuf {
        path::PathBuf::from(&self.destdir)
    }

    fn full_prefix(&self) -> path::PathBuf {
        return path::Path::new(&self.destdir).join(&self.prefix);
    }

    fn full_bindir(&self) -> path::PathBuf {
        return path::Path::new(&self.destdir)
            .join(&self.prefix)
            .join(&self.bindir);
    }

    fn full_sysconfdir(&self) -> path::PathBuf {
        return path::Path::new(&self.destdir).join(&self.sysconfdir);
    }

    fn full_unitdir(&self) -> path::PathBuf {
        return path::Path::new(&self.destdir)
            .join(&self.prefix)
            .join(&self.unitdir);
    }
}

#[derive(Serialize)]
struct InstallDirs {
    destdir: path::PathBuf,
    prefix: path::PathBuf,
    bindir: path::PathBuf,
    sysconfdir: path::PathBuf,
    unitdir: path::PathBuf,
}

pub struct Installer {
    opts: Options,
    install_dirs: InstallDirs,
}

#[derive(Error, Debug)]
enum InstallBinariesError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    EscalateIfNotOwned(#[from] EscalateIfNotOwnedError),

    #[error("the project is not built (with the requested profile)")]
    NotBuilt,
}

#[derive(Error, Debug)]
enum InstallConfigError {
    #[error(transparent)]
    FS(#[from] fs_extra::error::Error),

    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    EscalateIfNotOwned(#[from] EscalateIfNotOwnedError),
}

#[derive(Error, Debug)]
enum InstallUnitsError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Tera(#[from] tera::Error),

    #[error(transparent)]
    EscalateIfNotOwned(#[from] EscalateIfNotOwnedError),

    #[error("could not determine a file name for a templated file")]
    TemplatedFileName,
}

impl Installer {
    pub fn new(opts: Options) -> Installer {
        Installer {
            install_dirs: InstallDirs {
                destdir: opts.full_destdir(),
                prefix: opts.full_prefix(),
                bindir: opts.full_bindir(),
                sysconfdir: opts.full_sysconfdir(),
                unitdir: opts.full_unitdir(),
            },
            opts,
        }
    }

    fn install_binaries(&self) -> Result<(), InstallBinariesError> {
        let bindir = self.install_dirs.bindir.clone();

        mkdir_if_not_exists(bindir.clone())?;
        escalate_if_not_owned(bindir.clone())?;

        let target_path = path::Path::new("target").join(self.opts.profile.clone());
        if !target_path.exists() {
            return Err(InstallBinariesError::NotBuilt);
        }
        for entry in fs::read_dir(target_path)? {
            let path_cur = entry?.path();
            let metadata = path_cur.metadata()?;

            // Skip directories. They might meet the next if statement (executable
            // bit), but we don't want to install them.
            if metadata.is_dir() {
                continue;
            }

            // If the file is executable.
            if metadata.permissions().mode() & 0o111 != 0 {
                let file_name = path_cur.file_name().unwrap();

                // Skip xtask (which is THIS binary :) )
                if file_name == "xtask" {
                    continue;
                }

                let path_dest = bindir.clone().join(file_name);
                println!(
                    "Installing {} to {}",
                    file_name.to_string_lossy(),
                    path_dest.display()
                );
                fs::copy(path_cur, path_dest)?;
            }
        }
        Ok(())
    }

    fn install_config(&self) -> Result<(), InstallConfigError> {
        let sysconfdir = self.install_dirs.sysconfdir.clone();

        mkdir_if_not_exists(sysconfdir.clone())?;
        escalate_if_not_owned(sysconfdir.clone())?;

        let config_path = path::Path::new("contrib").join("etc");
        if !config_path.exists() {
            return Ok(());
        }

        let mut paths = Vec::new();
        for entry in fs::read_dir(config_path)? {
            let path_cur = entry?.path();
            paths.push(path_cur);
        }

        println!("Installing config files");
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        fs_extra::copy_items(&paths, sysconfdir, &options)?;

        Ok(())
    }

    fn __install_and_template_units(
        &self,
        unit_path: path::PathBuf,
        file_name: &OsStr,
    ) -> Result<(), InstallUnitsError> {
        // Remove ".in" suffix.
        let file_name_new = match file_name.to_str().unwrap().get(..file_name.len() - 3) {
            Some(f) => f,
            None => return Err(InstallUnitsError::TemplatedFileName),
        };
        let path_dest = self.install_dirs.unitdir.clone().join(file_name_new);

        let tera = Tera::new(&unit_path.join("*.in").to_string_lossy())?;
        let content = tera.render(
            &file_name.to_string_lossy(),
            &Context::from_serialize(&self.install_dirs)?,
        )?;

        let mut file_dst = fs::File::create(path_dest.clone())?;
        println!(
            "Templating and installing systemd unit {} to {}",
            file_name.to_string_lossy(),
            path_dest.display()
        );
        file_dst.write_all(content.as_bytes())?;

        Ok(())
    }

    fn __install_units(
        &self,
        path_cur: path::PathBuf,
        file_name: &OsStr,
    ) -> Result<(), InstallUnitsError> {
        let path_dest = self.install_dirs.unitdir.clone().join(file_name);
        println!(
            "Installing systemd unit {} to {}",
            file_name.to_string_lossy(),
            path_dest.display()
        );
        fs::copy(path_cur, path_dest)?;

        Ok(())
    }

    fn install_units(&self) -> Result<(), InstallUnitsError> {
        let unitdir = self.install_dirs.unitdir.clone();

        mkdir_if_not_exists(unitdir.clone())?;
        escalate_if_not_owned(unitdir)?;

        let unit_path = path::Path::new("contrib").join("systemd");
        if !unit_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(unit_path.clone())? {
            let path_cur = entry?.path();
            let metadata = path_cur.metadata()?;

            // No nested directories in systemd units.
            if metadata.is_dir() {
                continue;
            }

            let file_name = path_cur.file_name().unwrap();

            match path_cur.extension() {
                Some(ext) => {
                    if ext == "in" {
                        self.__install_and_template_units(unit_path.clone(), file_name)?;
                    } else {
                        self.__install_units(path_cur.clone(), file_name)?;
                    }
                }
                None => {
                    self.__install_units(path_cur.clone(), file_name)?;
                }
            }
        }

        Ok(())
    }

    pub fn do_install(&self) -> anyhow::Result<()> {
        self.install_binaries()?;
        self.install_config()?;
        self.install_units()?;
        Ok(())
    }
}
