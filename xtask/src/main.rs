mod bintar;
mod build_ebpf;
mod install;
mod run;

use std::process::exit;

use structopt::StructOpt;
#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Bintar(bintar::Options),
    BuildEbpf(build_ebpf::Options),
    Install(install::Options),
    Run(run::Options),
}

fn main() {
    let opts = Options::from_args();

    use Command::*;
    let ret = match opts.command {
        Bintar(opts) => {
            bintar::BinTar::new(opts).do_bin_tar()
        }
        BuildEbpf(opts) => build_ebpf::build_ebpf(opts),
        Install(opts) => install::Installer::new(opts).do_install(),
        Run(opts) => run::run(opts),
    };

    if let Err(e) = ret {
        eprintln!("{:#}", e);
        exit(1);
    }
}
