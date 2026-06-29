use clap::Subcommand;

#[derive(Subcommand)]
pub enum DatasetCmd {
    Active { #[arg(long)] set: Option<String> },
    Pending { #[arg(long)] set: Option<String> },
}

#[derive(Subcommand)]
pub enum CommissionerCmd {
    State { #[arg(long)] set: Option<String> },
    JoinerList,
    JoinerAdd { #[arg(long)] file: String },
    JoinerRemove { value: String },
}

pub fn read_json_input(spec: &str) -> crate::error::Result<serde_json::Value> {
    let raw = if spec == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).map_err(|e| crate::error::Error::Parse(e.to_string()))?;
        s
    } else {
        std::fs::read_to_string(spec).map_err(|e| crate::error::Error::Parse(e.to_string()))?
    };
    serde_json::from_str(&raw).map_err(|e| crate::error::Error::Parse(e.to_string()))
}
