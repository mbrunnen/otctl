pub mod actions;
pub mod devices;
pub mod node_ext;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::facade::{OtbrClient, DEFAULT_BASE_URL};
use crate::render::to_output;
use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{generate, Shell};

#[derive(Parser)]
#[command(name = "otctl", about = "OpenThread Border Router REST CLI")]
pub struct Cli {
    #[arg(long, env = "OTBR_URL", global = true)]
    pub base_url: Option<String>,
    /// Name of a saved border router (see `otctl config list`)
    #[arg(long, env = "OTBR_ROUTER", global = true)]
    pub router: Option<String>,
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
    Config {
        #[command(subcommand)]
        what: ConfigCmd,
    },
    /// Print a shell completion script to stdout
    Completions { shell: Shell },
}

#[derive(Subcommand)]
pub enum ConfigCmd {
    /// Save a border router under a name
    Add { name: String, url: String },
    /// Delete a saved border router
    Remove { name: String },
    /// Show the saved border routers
    List,
    /// Use a saved border router when no --router is given
    Default { name: String },
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

impl Cli {
    /// Parse like `Cli::parse`, but a typed `--base-url` or `--router` overrides the other one's env var.
    pub fn parse_args() -> Cli {
        let mut command = Cli::command();
        let matches = command.get_matches_mut();
        let mut cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
        let url = matches.value_source("base_url");
        let router = matches.value_source("router");
        let typed = [url, router].contains(&Some(ValueSource::CommandLine));
        if typed
            && matches!(
                cli.command,
                Command::Config { .. } | Command::Completions { .. }
            )
        {
            command
                .error(
                    ErrorKind::ArgumentConflict,
                    "--base-url and --router have no effect on config and completions",
                )
                .exit();
        }
        match (url, router) {
            (Some(ValueSource::CommandLine), Some(ValueSource::EnvVariable)) => cli.router = None,
            (Some(ValueSource::EnvVariable), Some(ValueSource::CommandLine)) => cli.base_url = None,
            (Some(_), Some(_)) => command
                .error(
                    ErrorKind::ArgumentConflict,
                    "--base-url (OTBR_URL) and --router (OTBR_ROUTER) cannot be used together",
                )
                .exit(),
            _ => {}
        }
        cli
    }
}

pub async fn run(cli: Cli) -> Result<String> {
    if let Command::Completions { shell } = cli.command {
        return Ok(completions(shell));
    }

    if let Command::Config { what } = cli.command {
        return config(what, cli.json);
    }

    let base_url = match cli.base_url {
        Some(url) => url,
        None => Config::load()?.resolve_url(cli.router, DEFAULT_BASE_URL)?,
    };
    let client = OtbrClient::new(&base_url);
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
            node_ext::CommissionerCmd::JoinerAdd {
                file,
                pskd,
                joiner_id,
                eui64,
                discerner,
                timeout,
            } => {
                let body = match file {
                    Some(f) => node_ext::read_json_input(&f)?,
                    None => node_ext::joiner_body(
                        pskd.as_deref()
                            .expect("clap requires --pskd without --file"),
                        joiner_id.as_deref(),
                        eui64.as_deref(),
                        discerner.as_deref(),
                        timeout,
                    ),
                };
                client.add_commissioner_joiner(body).await?
            }
            node_ext::CommissionerCmd::JoinerRemove { value } => {
                client.remove_commissioner_joiner(&value).await?
            }
            node_ext::CommissionerCmd::Enroll {
                pskd,
                joiner_id,
                eui,
                discerner,
                timeout,
                no_wait,
            } => {
                let body = node_ext::enroll_body(
                    &pskd,
                    joiner_id.as_deref(),
                    eui.as_deref(),
                    discerner.as_deref(),
                    timeout,
                );
                if no_wait {
                    let id = client.submit_action(body).await?;
                    serde_json::json!({ "id": id })
                } else {
                    // The task itself expires after `timeout`; give the border
                    // router a grace period to report the final status.
                    client
                        .submit_and_wait(
                            body,
                            std::time::Duration::from_secs(1),
                            std::time::Duration::from_secs(u64::from(timeout) + 30),
                        )
                        .await?
                }
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
            actions::ActionsCmd::Submit {
                file,
                no_wait,
                timeout,
            } => {
                let body = node_ext::read_json_input(&file)?;
                if no_wait {
                    let id = client.submit_action(body).await?;
                    serde_json::json!({ "id": id })
                } else {
                    client
                        .submit_and_wait(
                            body,
                            std::time::Duration::from_secs(1),
                            std::time::Duration::from_secs(timeout),
                        )
                        .await?
                }
            }
            actions::ActionsCmd::List => client.actions_list().await?,
            actions::ActionsCmd::Get { id } => client.get_action(&id).await?,
            actions::ActionsCmd::DeleteAll => client.actions_delete_all().await?,
        },
        Command::Config { .. } | Command::Completions { .. } => {
            unreachable!("handled above before client construction")
        }
    };
    Ok(to_output(&value, cli.json))
}

fn completions(shell: Shell) -> String {
    let mut buf = Vec::new();
    generate(shell, &mut Cli::command(), "otctl", &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

fn config(what: ConfigCmd, json: bool) -> Result<String> {
    let mut config = Config::load()?;
    let message = match what {
        ConfigCmd::List => {
            let rows: Vec<(&String, &String, bool)> = config
                .routers
                .iter()
                .map(|(name, router)| {
                    let is_default = config.default.as_deref() == Some(name.as_str());
                    (name, &router.url, is_default)
                })
                .collect();
            if json {
                let list = rows
                    .iter()
                    .map(|(name, url, is_default)| {
                        serde_json::json!({"name": name, "url": url, "default": is_default})
                    })
                    .collect();
                return Ok(to_output(&serde_json::Value::Array(list), true));
            }
            if rows.is_empty() {
                return Ok("no routers saved".into());
            }
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["name", "url", "default"]);
            for (name, url, is_default) in rows {
                table.add_row(vec![
                    name.as_str(),
                    url.as_str(),
                    if is_default { "*" } else { "" },
                ]);
            }
            return Ok(table.to_string());
        }
        ConfigCmd::Add { name, url } => {
            config.add(&name, &url)?;
            format!("Added router '{name}'")
        }
        ConfigCmd::Remove { name } => {
            config.remove(&name)?;
            format!("Removed router '{name}'")
        }
        ConfigCmd::Default { name } => {
            config.set_default(&name)?;
            format!("Default router is '{name}'")
        }
    };
    config.save()?;
    Ok(to_output(&serde_json::Value::String(message), json))
}

pub fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Api { .. } => 3,
        Error::Timeout => 4,
        _ => 1,
    }
}
