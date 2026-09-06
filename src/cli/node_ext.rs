use clap::Subcommand;

#[derive(Subcommand)]
pub enum DatasetCmd {
    Active {
        #[arg(long)]
        set: Option<String>,
    },
    Pending {
        #[arg(long)]
        set: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CommissionerCmd {
    State {
        #[arg(long)]
        set: Option<String>,
    },
    JoinerList,
    JoinerAdd {
        #[arg(long, conflicts_with_all = ["pskd", "joiner_id", "eui64", "discerner", "timeout"])]
        file: Option<String>,
        #[arg(long, required_unless_present = "file")]
        pskd: Option<String>,
        #[arg(long, conflicts_with_all = ["eui64", "discerner"])]
        joiner_id: Option<String>,
        #[arg(long, conflicts_with = "discerner")]
        eui64: Option<String>,
        #[arg(long)]
        discerner: Option<String>,
        /// Joiner expiration time in seconds
        #[arg(long)]
        timeout: Option<u32>,
    },
    JoinerRemove {
        value: String,
    },
    /// Enrol a joiner via an addThreadDeviceTask: the border router petitions
    /// the commissioner, adds the joiner and stops the commissioner again.
    Enroll {
        #[arg(long)]
        pskd: String,
        #[arg(long, conflicts_with_all = ["eui", "discerner"])]
        joiner_id: Option<String>,
        #[arg(long, conflicts_with = "discerner")]
        eui: Option<String>,
        #[arg(long)]
        discerner: Option<String>,
        /// How long the border router waits for the joiner, in seconds
        #[arg(long, default_value_t = 900)]
        timeout: u32,
        /// Print the action id instead of waiting for the joiner
        #[arg(long)]
        no_wait: bool,
    },
}

pub fn read_json_input(spec: &str) -> crate::error::Result<serde_json::Value> {
    let raw = if spec == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| crate::error::Error::Parse(e.to_string()))?;
        s
    } else {
        std::fs::read_to_string(spec).map_err(|e| crate::error::Error::Parse(e.to_string()))?
    };
    serde_json::from_str(&raw).map_err(|e| crate::error::Error::Parse(e.to_string()))
}

/// Builds a `JoinerData` body from the joiner flags. Without an explicit
/// target the wildcard joiner id is sent, matching the API default.
pub fn joiner_body(
    pskd: &str,
    joiner_id: Option<&str>,
    eui64: Option<&str>,
    discerner: Option<&str>,
    timeout: Option<u32>,
) -> serde_json::Value {
    let mut body = serde_json::Map::new();
    body.insert("pskd".into(), pskd.into());
    match (joiner_id, eui64, discerner) {
        (_, Some(v), _) => body.insert("eui64".into(), v.into()),
        (_, _, Some(v)) => body.insert("discerner".into(), v.into()),
        (id, _, _) => body.insert("joinerId".into(), id.unwrap_or("*").into()),
    };
    if let Some(t) = timeout {
        body.insert("timeout".into(), t.into());
    }
    serde_json::Value::Object(body)
}

/// Builds an `addThreadDeviceTask` submission. The task attributes name the
/// EUI-64 field `eui`, unlike the `eui64` of the joiner endpoint.
pub fn enroll_body(
    pskd: &str,
    joiner_id: Option<&str>,
    eui: Option<&str>,
    discerner: Option<&str>,
    timeout: u32,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert("pskd".into(), pskd.into());
    match (joiner_id, eui, discerner) {
        (_, Some(v), _) => attributes.insert("eui".into(), v.into()),
        (_, _, Some(v)) => attributes.insert("discerner".into(), v.into()),
        (id, _, _) => attributes.insert("joinerId".into(), id.unwrap_or("*").into()),
    };
    attributes.insert("timeout".into(), timeout.into());
    serde_json::json!({
        "data": [{ "type": "addThreadDeviceTask", "attributes": attributes }]
    })
}
