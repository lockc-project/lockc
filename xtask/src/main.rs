use anyhow::Result;
use structopt::StructOpt;

mod bintar;
mod install;

#[derive(StructOpt)]
pub(crate) struct Options {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Bintar(bintar::Options),
    Install(install::Options),
}

fn main() -> Result<()> {
    let opts = Options::from_args();

    use Command::*;
    match opts.command {
        Bintar(opts) => {
            bintar::BinTar::new(opts).do_bin_tar()?;
        }
        Install(opts) => {
            install::Installer::new(opts).do_install()?;
        }
    };

    Ok(())
}
