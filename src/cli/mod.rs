pub mod actions;
pub mod devices;
pub mod node_ext;

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
    Dataset {
        #[command(subcommand)]
        what: node_ext::DatasetCmd,
    },
    Commissioner {
        #[command(subcommand)]
        what: node_ext::CommissionerCmd,
    },
    Devices {
        #[command(subcommand)]
        what: devices::DevicesCmd,
    },
    Diagnostics {
        #[command(subcommand)]
        what: devices::DiagnosticsCmd,
    },
    Actions {
        #[command(subcommand)]
        what: actions::ActionsCmd,
    },
}

#[derive(Subcommand)]
pub enum NodeCmd {
    State {
        #[arg(long)]
        set: Option<String>,
    },
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
    Reset {
        #[arg(long)]
        yes: bool,
    },
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
            NodeCmd::Reset { yes } => {
                if !yes {
                    eprintln!(
                        "This will factory-reset the border router and erase all persistent data."
                    );
                    eprint!("Type 'yes' to confirm: ");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .map_err(|e| Error::Parse(e.to_string()))?;
                    if input.trim() != "yes" {
                        return Err(Error::Aborted("reset aborted".into()));
                    }
                }
                client.node_reset().await?
            }
        },
        Command::Dataset { what } => match what {
            node_ext::DatasetCmd::Active { set: Some(f) } => {
                client
                    .set_dataset_active(node_ext::read_json_input(&f)?)
                    .await?
            }
            node_ext::DatasetCmd::Active { set: None } => client.dataset_active().await?,
            node_ext::DatasetCmd::Pending { set: Some(f) } => {
                client
                    .set_dataset_pending(node_ext::read_json_input(&f)?)
                    .await?
            }
            node_ext::DatasetCmd::Pending { set: None } => client.dataset_pending().await?,
        },
        Command::Commissioner { what } => match what {
            node_ext::CommissionerCmd::State { set: Some(s) } => {
                client.set_commissioner_state(&s).await?
            }
            node_ext::CommissionerCmd::State { set: None } => client.commissioner_state().await?,
            node_ext::CommissionerCmd::JoinerList => client.commissioner_joiner().await?,
            node_ext::CommissionerCmd::JoinerAdd { file } => {
                client
                    .add_commissioner_joiner(node_ext::read_json_input(&file)?)
                    .await?
            }
            node_ext::CommissionerCmd::JoinerRemove { value } => {
                client.remove_commissioner_joiner(&value).await?
            }
        },
        Command::Devices { what } => match what {
            devices::DevicesCmd::List { no_refresh } => {
                if !no_refresh {
                    eprintln!(
                        "Refreshing device collection (network scan, up to ~100s); use --no-refresh to skip."
                    );
                    client
                        .refresh_device_collection(
                            std::time::Duration::from_secs(1),
                            std::time::Duration::from_secs(100),
                        )
                        .await?;
                }
                client.devices_list().await?
            }
            devices::DevicesCmd::Get { id } => client.device_get(&id).await?,
            devices::DevicesCmd::Delete { id } => client.device_delete(&id).await?,
            devices::DevicesCmd::DeleteAll => client.devices_delete_all().await?,
        },
        Command::Diagnostics { what } => match what {
            devices::DiagnosticsCmd::List => client.diagnostics_list().await?,
            devices::DiagnosticsCmd::Get { id } => client.diagnostic_get(&id).await?,
            devices::DiagnosticsCmd::DeleteAll => client.diagnostics_delete_all().await?,
        },
        Command::Actions { what } => match what {
            actions::ActionsCmd::Submit { file, no_wait } => {
                let body = node_ext::read_json_input(&file)?;
                if no_wait {
                    let id = client.submit_action(body).await?;
                    serde_json::json!({ "id": id })
                } else {
                    client
                        .submit_and_wait(
                            body,
                            std::time::Duration::from_secs(1),
                            std::time::Duration::from_secs(60),
                        )
                        .await?
                }
            }
            actions::ActionsCmd::List => client.actions_list().await?,
            actions::ActionsCmd::Get { id } => client.get_action(&id).await?,
            actions::ActionsCmd::DeleteAll => client.actions_delete_all().await?,
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
