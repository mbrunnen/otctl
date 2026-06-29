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
    State { #[arg(long)] set: Option<String> },
    Rloc,
    Rloc16,
    ExtAddress,
    NetworkName,
    LeaderData,
    ExtPanid,
    NumOfRouter,
    CoprocessorVersion,
    BaId,
    Info,
}

pub async fn run(cli: Cli) -> Result<String> {
    let client = OtbrClient::new(&cli.base_url);
    let value = match cli.command {
        Command::Node { what } => match what {
            NodeCmd::State { set: Some(s) } => client.set_node_state(&s).await?,
            NodeCmd::State { set: None } => client.node_state().await?,
            NodeCmd::Rloc => client.node_rloc().await?,
            NodeCmd::Rloc16 => client.node_rloc16().await?,
            NodeCmd::ExtAddress => client.node_ext_address().await?,
            NodeCmd::NetworkName => client.node_network_name().await?,
            NodeCmd::LeaderData => client.node_leader_data().await?,
            NodeCmd::ExtPanid => client.node_ext_panid().await?,
            NodeCmd::NumOfRouter => client.node_num_of_router().await?,
            NodeCmd::CoprocessorVersion => client.node_coprocessor_version().await?,
            NodeCmd::BaId => client.node_ba_id().await?,
            NodeCmd::Info => client.node_info().await?,
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
