use clap::Subcommand;

#[derive(Subcommand)]
pub enum ActionsCmd {
    Submit {
        #[arg(long)]
        file: String,
        #[arg(long)]
        no_wait: bool,
    },
    List,
    Get { id: String },
    DeleteAll,
}
