use crate::error::{Error, Result};
use crate::facade::{OtbrClient, DEFAULT_BASE_URL};
use crate::render::to_output;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "otctl", about = "OpenThread Border Router REST CLI")]
pub struct Cli {
    #[arg(long, env = "OTBR_URL", default_value = DEFAULT_BASE_URL, global = true)]
    pub base_url: String,
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Node {
        #[command(subcommand)]
        what: NodeCmd,
    },
}

#[derive(Subcommand)]
pub enum NodeCmd {
    State,
}

pub async fn run(cli: Cli) -> Result<String> {
    let client = OtbrClient::new(&cli.base_url);
    let value = match cli.command {
        Command::Node { what } => match what {
            NodeCmd::State => client.node_state().await?,
        },
    };
    Ok(to_output(&value, cli.json))
}

pub fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Api { .. } => 3,
        Error::Timeout => 4,
        _ => 1,
    }
}
