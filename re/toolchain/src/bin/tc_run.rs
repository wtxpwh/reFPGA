use clap::Parser;
use prjcombine_re_toolchain::Toolchain;

use std::{error::Error, ffi::OsString, path::PathBuf};

#[derive(Debug, Parser)]
#[command(name = "tc_run", about = "Run a command within a toolchain context.")]
struct Args {
    tc: PathBuf,
    #[arg(long)]
    cwd: Option<PathBuf>,
    cmd: String,
    args: Vec<OsString>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let toolchain = Toolchain::from_file(&args.tc)?;
    let mut cmd = toolchain.command(&args.cmd);
    if let Some(cwd) = args.cwd {
        cmd.current_dir(cwd);
    }
    cmd.args(args.args);
    let status = cmd.status()?;
    if !status.success() {
        println!("OOPSIE {status}");
    }
    Ok(())
}
