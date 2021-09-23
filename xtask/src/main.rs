use anyhow::Result;
use structopt::StructOpt;

mod install;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Install(install::Options),
}

fn main() -> Result<()> {
    let opts = Options::from_args();

    use Command::*;
    match opts.command {
        Install(opts) => {
            install::Installer::new(opts).do_install()?;
        }
    };

    Ok(())
}
