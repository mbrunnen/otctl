use clap::Subcommand;

#[derive(Subcommand)]
pub enum DevicesCmd {
    List {
        #[arg(long)]
        no_refresh: bool,
    },
    Get {
        id: String,
    },
    Delete {
        id: String,
    },
    DeleteAll,
}

#[derive(Subcommand)]
pub enum DiagnosticsCmd {
    List,
    Get { id: String },
    DeleteAll,
}
