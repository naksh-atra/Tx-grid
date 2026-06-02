use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "tmux-taskgrid",
    version = "0.1.0",
    about = "A tmux task grid for managing long-running commands and AI agents",
    long_about = None
)]
pub struct Args {
    /// Print task summary as JSON and exit (non-interactive mode).
    #[arg(long)]
    pub check: bool,

    /// Print machine-readable JSON output and exit.
    #[arg(long)]
    pub json: bool,

    /// Enable verbose debug logging.
    #[arg(long)]
    pub debug: bool,

    /// Run environment diagnostics.
    #[arg(long)]
    pub doctor: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
