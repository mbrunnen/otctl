use crate::error::{map_progenitor, Error, Result};
use crate::generated::Client;
use progenitor_client::ResponseValue;

pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8081";

pub struct OtbrClient {
    inner: Client,
}

fn into_value<T: serde::Serialize>(r: ResponseValue<T>) -> Result<serde_json::Value> {
    serde_json::to_value(r.into_inner()).map_err(|e| Error::Parse(e.to_string()))
}

#[allow(dead_code)]
fn ok_after<T>(_r: ResponseValue<T>) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "ok": true }))
}

impl OtbrClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            inner: Client::new(base_url.trim_end_matches('/')),
        }
    }

    pub async fn node_state(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_state().await.map_err(map_progenitor)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn node_state_returns_body() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/node/state");
            then.status(200).header("content-type", "application/json").body("\"leader\"");
        });
        let client = OtbrClient::new(&server.base_url());
        let v = client.node_state().await.unwrap();
        m.assert();
        assert_eq!(v, serde_json::json!("leader"));
    }
}
