use crate::error::{map_progenitor, parse_api_errors, Error, Result};
use crate::generated::{types, Client};
use progenitor_client::ResponseValue;
use std::time::{Duration, Instant};

pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8081";

pub struct OtbrClient {
    inner: Client,
}

fn into_value<T: serde::Serialize>(r: ResponseValue<T>) -> Result<serde_json::Value> {
    serde_json::to_value(r.into_inner()).map_err(|e| Error::Parse(e.to_string()))
}

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

    pub async fn node_rloc(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_rloc().await.map_err(map_progenitor)?)
    }

    pub async fn node_rloc16(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_rloc16().await.map_err(map_progenitor)?)
    }

    pub async fn node_ext_address(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ext_address().await.map_err(map_progenitor)?)
    }

    pub async fn node_network_name(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_network_name().await.map_err(map_progenitor)?)
    }

    pub async fn node_leader_data(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_leader_data().await.map_err(map_progenitor)?)
    }

    pub async fn node_ext_panid(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ext_pan_id().await.map_err(map_progenitor)?)
    }

    pub async fn node_num_of_router(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_num_of_router().await.map_err(map_progenitor)?)
    }

    pub async fn node_coprocessor_version(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_coprocessor_version().await.map_err(map_progenitor)?)
    }

    pub async fn node_ba_id(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ba_id().await.map_err(map_progenitor)?)
    }

    pub async fn node_info(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_information().await.map_err(map_progenitor)?)
    }

    pub async fn set_node_state(&self, state: &str) -> Result<serde_json::Value> {
        ok_after(self.inner.set_node_state(state).await.map_err(map_progenitor)?)
    }

    pub async fn submit_action(&self, body: serde_json::Value) -> Result<String> {
        let typed: types::ActionsClient =
            serde_json::from_value(body).map_err(|e| Error::Parse(e.to_string()))?;
        let resp = self.inner.invoke_action(&typed).await.map_err(map_progenitor)?;
        let v = serde_json::to_value(resp.into_inner()).map_err(|e| Error::Parse(e.to_string()))?;
        v["data"][0]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Parse("missing action id in response".into()))
    }

    pub async fn get_action(&self, id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/actions/{}", self.inner.baseurl(), id);
        let resp = self
            .inner
            .client()
            .get(&url)
            .send()
            .await
            .map_err(Error::Http)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(parse_api_errors(status, &body));
        }
        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| Error::Parse(e.to_string()))
    }

    pub async fn submit_and_wait(
        &self,
        body: serde_json::Value,
        poll: Duration,
        timeout: Duration,
    ) -> Result<serde_json::Value> {
        let id = self.submit_action(body).await?;
        let deadline = Instant::now() + timeout;
        loop {
            let action = self.get_action(&id).await?;
            let status = action["data"]["attributes"]["status"].as_str().unwrap_or("");
            if matches!(status, "completed" | "stopped" | "failed") {
                return Ok(action);
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout);
            }
            tokio::time::sleep(poll).await;
        }
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

    #[tokio::test]
    async fn submit_and_wait_polls_until_completed() {
        let server = MockServer::start();
        let post = server.mock(|when, then| {
            when.method(POST).path("/api/actions");
            then.status(200).header("content-type", "application/json").body(
                r#"{"data":[{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"pending"}}]}"#,
            );
        });
        let get = server.mock(|when, then| {
            when.method(GET).path("/api/actions/abc");
            then.status(200).header("content-type", "application/json").body(
                r#"{"data":{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"completed"}}}"#,
            );
        });
        let client = OtbrClient::new(&server.base_url());
        let v = client
            .submit_and_wait(
                serde_json::json!({"data":[]}),
                std::time::Duration::from_millis(1),
                std::time::Duration::from_secs(5),
            )
            .await
            .unwrap();
        post.assert();
        get.assert_hits(1);
        assert_eq!(v["data"]["attributes"]["status"], "completed");
    }

    #[tokio::test]
    async fn get_action_surfaces_non_2xx_as_api_error() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/api/actions/missing");
            then.status(404)
                .header("content-type", "application/json")
                .body(r#"{"errors":[{"title":"not found"}]}"#);
        });
        let client = OtbrClient::new(&server.base_url());
        let err = client.get_action("missing").await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 404, .. }));
    }

    #[tokio::test]
    async fn submit_and_wait_returns_timeout_when_action_stays_pending() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/api/actions");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data":[{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"pending"}}]}"#);
        });
        server.mock(|when, then| {
            when.method(GET).path("/api/actions/abc");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data":{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"pending"}}}"#);
        });
        let client = OtbrClient::new(&server.base_url());
        let err = client
            .submit_and_wait(
                serde_json::json!({"data":[]}),
                std::time::Duration::from_millis(1),
                std::time::Duration::from_millis(20),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Timeout));
    }
}
