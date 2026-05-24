use anyhow::Result;

use crate::cli::Cli;

pub fn run() -> Result<()> {
    let cli = Cli::parse_args();
    crate::commands::dispatch(cli)
}
