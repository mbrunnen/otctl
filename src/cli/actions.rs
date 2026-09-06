use clap::Subcommand;

#[derive(Subcommand)]
pub enum ActionsCmd {
    Submit {
        #[arg(long)]
        file: String,
        #[arg(long)]
        no_wait: bool,
        /// How long to wait for the action to reach a terminal status, in seconds
        #[arg(long, default_value_t = 60)]
        timeout: u64,
    },
    List,
    Get {
        id: String,
    },
    DeleteAll,
}
