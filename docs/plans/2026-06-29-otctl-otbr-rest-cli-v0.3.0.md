# otctl OTBR REST CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `otctl`, a Rust CLI (with a reusable client library underneath) for the OpenThread Border Router REST API v0.3.0.

**Architecture:** A single crate exposes a progenitor-generated typed client (from a vendored, hand-patched OpenAPI spec). An `OtbrClient` facade wraps the generated client and calls its typed methods, normalising results to `serde_json::Value` at its public boundary so rendering stays uniform. A clap-derive binary `otctl` calls the facade and renders human tables by default or raw JSON with `--json`. The async Actions task queue is handled by a `submit_and_wait` poll helper.

**Tech Stack:** Rust (edition 2021), progenitor + progenitor-client, reqwest (async), tokio, serde / serde_json, clap (derive), thiserror, comfy-table; dev: httpmock, assert_cmd, predicates.

## Global Constraints

- Rust edition `2021`; build on stable toolchain.
- Crate name `otctl`; library target `otctl` + binary target `otctl` (`src/bin/otctl.rs`).
- Default base URL `http://127.0.0.1:8081`; override via `--base-url` flag, env fallback `OTBR_URL`.
- Vendored spec is the source of truth: `openapi/ot-rest.yaml`, forked from upstream `openthread/ot-br-posix` commit `ec16e396382b4559e70a2c6fdeecb7d596a5e915` (Swagger OpenThread REST API v0.3.0).
- The facade is the only consumer of generated client methods; the CLI depends only on `facade` and `render`. Generated method names are deterministic from `operationId` (snake_cased); where this plan names one, verify it against the discovery step (Task 2, Step 8) and substitute the actual name if progenitor differs.
- The facade's public methods take and return `serde_json::Value` (or `String`); typed generated request bodies are deserialised from `Value` inside the facade.
- No inline comments explaining rationale; no emojis anywhere; British English in prose.
- Files end with a trailing newline.
- Live-network integration tests run only when `OTBR_TEST_URL` is set; otherwise skipped.

---

### Task 1: Crate scaffold and vendored spec

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/bin/otctl.rs`
- Create: `src/error.rs` (stub)
- Create: `openapi/ot-rest.yaml` (vendored upstream spec, unpatched in this task)
- Create: `docs/spec-sync.md`

**Interfaces:**
- Consumes: nothing.
- Produces: a compiling crate with an empty `error` module and a `main` that prints a placeholder.

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "otctl"
version = "0.1.0"
edition = "2021"
license = "LicenseRef-proprietary"

[lib]
name = "otctl"
path = "src/lib.rs"

[[bin]]
name = "otctl"
path = "src/bin/otctl.rs"

[dependencies]
clap = { version = "4", features = ["derive", "env"] }
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
progenitor = "0.8"
progenitor-client = "0.8"
comfy-table = "7"

[dev-dependencies]
httpmock = "0.7"
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Create stub `src/error.rs`**

```rust
pub struct Placeholder;
```

- [ ] **Step 3: Create `src/lib.rs`**

```rust
pub mod error;
```

- [ ] **Step 4: Create stub `src/bin/otctl.rs`**

```rust
fn main() {
    println!("otctl");
}
```

- [ ] **Step 5: Vendor the upstream spec**

Download `src/rest/openapi.yaml` from upstream commit `ec16e396382b4559e70a2c6fdeecb7d596a5e915` into `openapi/ot-rest.yaml` verbatim. Prepend a single YAML comment line at the very top:

```yaml
# Vendored from openthread/ot-br-posix@ec16e396382b4559e70a2c6fdeecb7d596a5e915 (src/rest/openapi.yaml). Hand-patched for progenitor; see docs/spec-sync.md.
```

- [ ] **Step 6: Create `docs/spec-sync.md`**

```markdown
# Spec sync

`openapi/ot-rest.yaml` is vendored from `openthread/ot-br-posix`.

- Upstream path: `src/rest/openapi.yaml`
- Forked at commit: `ec16e396382b4559e70a2c6fdeecb7d596a5e915`
- API title/version: Swagger OpenThread REST API, v0.3.0

## Local patches (do not lose these when re-syncing)

1. Content type `application/vnd.api+json` rewritten to `application/json` so progenitor models bodies as typed.
2. Added `operationId` to the 22 `/node/*` operations.
3. `anyOf`-of-`$ref` unions changed to `oneOf` so progenitor emits enums.
4. Removed the `SparseFieldset` (`deepObject`) query parameter and its references.
5. Removed the secondary `DatasetTlv` request body from the dataset PUTs (v1 sets datasets via the JSON form only).
6. Removed `format: uuid` from the Actions id field so it generates as a plain `String` (avoids a uuid dependency).

## Re-sync procedure

1. Fetch the new upstream `openapi.yaml`.
2. Re-apply the patches above.
3. Run `cargo build` and fix any new progenitor incompatibilities.
4. Update the commit SHA in this file and in the spec header comment.
```

- [ ] **Step 7: Verify the crate builds**

Run: `cargo build`
Expected: compiles successfully (warning about unused `Placeholder` is acceptable).

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml src openapi docs/spec-sync.md
git commit -m "Scaffold otctl crate and vendor OTBR REST spec"
```

---

### Task 2: Patch vendored spec and wire progenitor codegen

**Files:**
- Modify: `openapi/ot-rest.yaml`
- Create: `src/generated.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: vendored spec from Task 1.
- Produces: module `otctl::generated` with a `Client` (from `progenitor::generate_api!`) exposing one async builder method per `operationId`, and generated request/response types under `otctl::generated::types`.

- [ ] **Step 1: Rewrite the JSON:API content type**

In `openapi/ot-rest.yaml`, replace every occurrence of `application/vnd.api+json:` with `application/json:`.

- [ ] **Step 2: Add `operationId` to the 22 `/node/*` operations**

Add these exact `operationId` values under each method:

```
GET    /node                       -> getNode
DELETE /node                       -> resetNode
GET    /node/ba-id                 -> getNodeBaId
GET    /node/rloc                  -> getNodeRloc
GET    /node/rloc16                -> getNodeRloc16
GET    /node/ext-address           -> getNodeExtAddress
GET    /node/state                 -> getNodeState
PUT    /node/state                 -> setNodeState
GET    /node/network-name          -> getNodeNetworkName
GET    /node/leader-data           -> getNodeLeaderData
GET    /node/ext-panid             -> getNodeExtPanId
GET    /node/num-of-router         -> getNodeNumOfRouter
GET    /node/dataset/active        -> getActiveDataset
PUT    /node/dataset/active        -> setActiveDataset
GET    /node/dataset/pending       -> getPendingDataset
PUT    /node/dataset/pending       -> setPendingDataset
GET    /node/commissioner/state    -> getCommissionerState
PUT    /node/commissioner/state    -> setCommissionerState
GET    /node/commissioner/joiner   -> getCommissionerJoiner
POST   /node/commissioner/joiner   -> addCommissionerJoiner
DELETE /node/commissioner/joiner   -> removeCommissionerJoiner
GET    /node/coprocessor/version   -> getCoprocessorVersion
```

- [ ] **Step 3: Convert `anyOf`-of-refs unions to `oneOf`**

Change the key `anyOf:` to `oneOf:` (item lists unchanged) in: `Actions_getNetworkDiagnosticTask`, `Actions_resetNetworkDiagCounterTask`, `ActionsClient` (under `data.items`), `ActionsCollection`, `ActionItem` (under `data`).

- [ ] **Step 4: Remove the `deepObject` sparse-fieldset parameter**

Delete the `SparseFieldset` parameter from `components.parameters`. Remove every `- $ref: "#/components/parameters/SparseFieldset"` line from operation `parameters` lists. Leave `AcceptJsonApi` intact.

- [ ] **Step 5: Trim multi-body and uuid wrinkles**

- In `PUT /node/dataset/active` and `PUT /node/dataset/pending` request bodies, keep only the `application/json` content (schemas `ActiveDataset` / `PendingDataset`); delete the other media-type entry whose schema is `DatasetTlv`.
- In schema `ActionsServer` (and any other place the Actions `id` is declared), remove the `format: uuid` line so `id` is a plain `string`.

- [ ] **Step 6: Verify the patched spec still parses as YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('openapi/ot-rest.yaml')); print('ok')"`
Expected: prints `ok`.

- [ ] **Step 7: Wire progenitor codegen in `src/generated.rs`**

```rust
#![allow(clippy::all)]
#![allow(dead_code)]
progenitor::generate_api!("openapi/ot-rest.yaml");
```

Update `src/lib.rs`:

```rust
pub mod error;
pub mod generated;
```

- [ ] **Step 8: Build, then record the generated method and type names**

Run: `cargo build`
Expected: compiles. If progenitor rejects a remaining `oneOf` body, capture the failing schema name and hand-model only that request type as a plain `serde` struct above the macro in `src/generated.rs`, then rebuild.

Then discover the real names:
Run: `cargo doc --no-deps` and open `target/doc/otctl/generated/struct.Client.html` (and `.../generated/types/index.html`).
Record: the builder method names (expected snake_case of the operationIds, e.g. `get_node_state`, `set_active_dataset`, `invoke_action`, `get_action`, `list_devices`, `get_device`), the path-param setter names (expected `action_id`, `device_id`, `diagnostics_id`), and the body type names (expected `types::ActionsClient`, `types::ActiveDataset`, `types::PendingDataset`, `types::JoinerData`). Later tasks assume these; substitute the actual names where they differ.

- [ ] **Step 9: Commit**

```bash
git add openapi/ot-rest.yaml src/generated.rs src/lib.rs
git commit -m "Patch vendored spec for progenitor and generate typed client"
```

---

### Task 3: Error type, API-error parsing, and progenitor error mapping

**Files:**
- Modify: `src/error.rs`
- Test: inline `#[cfg(test)]` in `src/error.rs`

**Interfaces:**
- Consumes: `generated` (for the `progenitor_client::Error` mapping).
- Produces:
  - `pub enum Error { Http(reqwest::Error), Api { status: u16, errors: Vec<ApiError> }, Timeout, Parse(String) }`
  - `pub struct ApiError { pub title: String, pub detail: Option<String> }`
  - `pub type Result<T> = std::result::Result<T, Error>;`
  - `pub fn parse_api_errors(status: u16, body: &str) -> Error`
  - `pub fn map_progenitor<E: serde::Serialize>(e: progenitor_client::Error<E>) -> Error`

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jsonapi_error_body() {
        let body = r#"{"errors":[{"title":"Bad Request","detail":"bad channel"}]}"#;
        match parse_api_errors(400, body) {
            Error::Api { status, errors } => {
                assert_eq!(status, 400);
                assert_eq!(errors[0].title, "Bad Request");
                assert_eq!(errors[0].detail.as_deref(), Some("bad channel"));
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_on_non_jsonapi_body() {
        match parse_api_errors(500, "internal failure") {
            Error::Api { status, errors } => {
                assert_eq!(status, 500);
                assert_eq!(errors[0].title, "internal failure");
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib error::`
Expected: FAIL (types/functions not defined).

- [ ] **Step 3: Write the implementation**

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Http(reqwest::Error),
    #[error("API error ({status}): {}", .errors.first().map(|e| e.title.clone()).unwrap_or_default())]
    Api { status: u16, errors: Vec<ApiError> },
    #[error("timed out waiting for action to complete")]
    Timeout,
    #[error("failed to parse response: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
struct ErrorEnvelope {
    errors: Vec<ApiError>,
}

pub fn parse_api_errors(status: u16, body: &str) -> Error {
    match serde_json::from_str::<ErrorEnvelope>(body) {
        Ok(env) if !env.errors.is_empty() => Error::Api { status, errors: env.errors },
        _ => Error::Api {
            status,
            errors: vec![ApiError { title: body.to_string(), detail: None }],
        },
    }
}

pub fn map_progenitor<E: serde::Serialize>(e: progenitor_client::Error<E>) -> Error {
    use progenitor_client::Error as Pe;
    match e {
        Pe::CommunicationError(e) => Error::Http(e),
        Pe::ErrorResponse(rv) => {
            let status = rv.status().as_u16();
            let body = serde_json::to_string(rv.as_ref()).unwrap_or_default();
            parse_api_errors(status, &body)
        }
        other => Error::Parse(other.to_string()),
    }
}
```

Delete the `Placeholder` stub. If the discovery in Task 2 showed a different `progenitor_client::Error` variant shape, adjust the `match`; the `other => Error::Parse(...)` arm keeps it total.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib error::`
Expected: PASS (both).

- [ ] **Step 5: Commit**

```bash
git add src/error.rs
git commit -m "Add Error type, API-error parsing, and progenitor error mapping"
```

---

### Task 4: OtbrClient facade — construction and node_state

**Files:**
- Create: `src/facade.rs`
- Modify: `src/lib.rs`
- Test: inline `#[cfg(test)]` in `src/facade.rs`

**Interfaces:**
- Consumes: `generated::Client`, `error::{map_progenitor, Error, Result}`.
- Produces:
  - `pub struct OtbrClient { inner: generated::Client }`
  - `pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8081";`
  - `pub fn new(base_url: &str) -> OtbrClient`
  - `pub async fn node_state(&self) -> Result<serde_json::Value>`
  - private `fn into_value<T: serde::Serialize>(r: progenitor_client::ResponseValue<T>) -> Result<serde_json::Value>`

- [ ] **Step 1: Write the failing test (httpmock-backed)**

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib facade::`
Expected: FAIL (module not found).

- [ ] **Step 3: Write the implementation**

```rust
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

impl OtbrClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            inner: Client::new(base_url.trim_end_matches('/')),
        }
    }

    pub async fn node_state(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_state().send().await.map_err(map_progenitor)?)
    }
}
```

Add `pub mod facade;` to `src/lib.rs`. If `get_node_state` is not the generated name (per Task 2 discovery), substitute the actual one.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib facade::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/facade.rs src/lib.rs
git commit -m "Add OtbrClient facade over generated client with node_state"
```

---

### Task 5: Actions submit_and_wait poll helper

**Files:**
- Modify: `src/facade.rs`
- Test: inline `#[cfg(test)]` in `src/facade.rs`

**Interfaces:**
- Consumes: `OtbrClient`, generated `invoke_action` / `get_action`, `into_value`, `error::{Error, Result, map_progenitor}`, `generated::types::ActionsClient`.
- Produces:
  - `pub async fn submit_action(&self, body: serde_json::Value) -> Result<String>` — deserialise `body` into `types::ActionsClient`, POST via `invoke_action`, return `data[0].id`.
  - `pub async fn get_action(&self, id: &str) -> Result<serde_json::Value>`
  - `pub async fn submit_and_wait(&self, body: serde_json::Value, poll: Duration, timeout: Duration) -> Result<serde_json::Value>` — submit then poll `get_action` until `data.attributes.status` is `completed`/`stopped`/`failed`, else `Error::Timeout`.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn submit_and_wait_polls_until_completed() {
    use httpmock::prelude::*;
    let server = MockServer::start();
    let post = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(201).header("content-type", "application/json").body(
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib facade::submit_and_wait`
Expected: FAIL (method not defined).

- [ ] **Step 3: Write the implementation**

```rust
use crate::generated::types;
use std::time::{Duration, Instant};

impl OtbrClient {
    pub async fn submit_action(&self, body: serde_json::Value) -> Result<String> {
        let typed: types::ActionsClient =
            serde_json::from_value(body).map_err(|e| Error::Parse(e.to_string()))?;
        let resp = self.inner.invoke_action().body(typed).send().await.map_err(map_progenitor)?;
        let v = serde_json::to_value(resp.into_inner()).map_err(|e| Error::Parse(e.to_string()))?;
        v["data"][0]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Parse("missing action id in response".into()))
    }

    pub async fn get_action(&self, id: &str) -> Result<serde_json::Value> {
        into_value(self.inner.get_action().action_id(id).send().await.map_err(map_progenitor)?)
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
```

If `body(typed)` does not type-check because the generated builder expects a borrowed or differently-named body, adjust per the Task 2 discovery (e.g. `.body(&typed)`); the `types::ActionsClient` deserialise step is unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib facade::`
Expected: PASS (all facade tests).

- [ ] **Step 5: Commit**

```bash
git add src/facade.rs
git commit -m "Add Actions submit/get and submit_and_wait poll helper"
```

---

### Task 6: Render module — tables and --json

**Files:**
- Create: `src/render.rs`
- Modify: `src/lib.rs`
- Test: inline `#[cfg(test)]` in `src/render.rs`

**Interfaces:**
- Consumes: `serde_json::Value`.
- Produces: `pub fn to_output(value: &serde_json::Value, json: bool) -> String`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_flag_emits_pretty_json() {
        let v = serde_json::json!({"state":"leader"});
        assert!(to_output(&v, true).contains("\"state\": \"leader\""));
    }

    #[test]
    fn scalar_renders_bare() {
        assert_eq!(to_output(&serde_json::json!("leader"), false), "leader");
    }

    #[test]
    fn object_renders_key_values() {
        let v = serde_json::json!({"rloc16":"0x4400","state":"leader"});
        let out = to_output(&v, false);
        assert!(out.contains("rloc16"));
        assert!(out.contains("0x4400"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib render::`
Expected: FAIL (module not found).

- [ ] **Step 3: Write the implementation**

```rust
use comfy_table::Table;
use serde_json::Value;

pub fn to_output(value: &Value, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    }
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Object(map) => {
            let mut table = Table::new();
            table.set_header(vec!["field", "value"]);
            for (k, v) in map {
                table.add_row(vec![k.clone(), scalarise(v)]);
            }
            table.to_string()
        }
        Value::Array(items) => items.iter().map(scalarise).collect::<Vec<_>>().join("\n"),
    }
}

fn scalarise(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
```

Add `pub mod render;` to `src/lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib render::`
Expected: PASS (all three).

- [ ] **Step 5: Commit**

```bash
git add src/render.rs src/lib.rs
git commit -m "Add render module for tables and --json output"
```

---

### Task 7: CLI scaffold — clap, global flags, dispatch

**Files:**
- Create: `src/cli/mod.rs`
- Modify: `src/lib.rs`
- Rewrite: `src/bin/otctl.rs`
- Test: `tests/cli_smoke.rs`

**Interfaces:**
- Consumes: `facade::{OtbrClient, DEFAULT_BASE_URL}`, `render::to_output`, `error::{Error, Result}`.
- Produces:
  - `pub struct Cli` (clap `Parser`) with `--base-url` (env `OTBR_URL`, default `DEFAULT_BASE_URL`, global), `--json` (global), and a `Command` subcommand enum (only `Node { State }` here).
  - `pub async fn run(cli: Cli) -> Result<String>`
  - `pub fn exit_code(err: &Error) -> i32` (1 generic, 3 API error, 4 timeout).

- [ ] **Step 1: Write the failing smoke test**

```rust
use assert_cmd::Command;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicates::str::contains("--base-url"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke`
Expected: FAIL.

- [ ] **Step 3: Write `src/cli/mod.rs`**

```rust
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
```

Add `pub mod cli;` to `src/lib.rs`.

- [ ] **Step 4: Rewrite `src/bin/otctl.rs`**

```rust
use clap::Parser;
use otctl::cli::{exit_code, run, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(output) => println!("{output}"),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(exit_code(&err));
        }
    }
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test cli_smoke`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/cli src/bin/otctl.rs src/lib.rs tests/cli_smoke.rs
git commit -m "Add clap CLI scaffold with global flags and node state dispatch"
```

---

### Task 8: Node info and state commands

**Files:**
- Modify: `src/facade.rs`
- Modify: `src/cli/mod.rs`
- Test: `tests/cli_node.rs`

**Interfaces:**
- Consumes: generated client builder methods (per Task 2 discovery), `into_value`, `map_progenitor`.
- Produces facade methods (all `async fn(&self) -> Result<serde_json::Value>` unless noted): `node_rloc`, `node_rloc16`, `node_ext_address`, `node_network_name`, `node_leader_data`, `node_ext_panid`, `node_num_of_router`, `node_coprocessor_version`, `node_ba_id`, `node_info`, and `set_node_state(&self, state: &str)`. Extends `NodeCmd` with matching subcommands.

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn node_rloc16_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/rloc16");
        then.status(200).header("content-type", "application/json").body("17408");
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "node", "rloc16"]);
    cmd.assert().success().stdout(predicates::str::contains("17408"));
    m.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_node`
Expected: FAIL (no `rloc16` subcommand).

- [ ] **Step 3: Add facade methods**

```rust
impl OtbrClient {
    pub async fn node_rloc(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_rloc().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_rloc16(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_rloc16().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_ext_address(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ext_address().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_network_name(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_network_name().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_leader_data(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_leader_data().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_ext_panid(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ext_pan_id().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_num_of_router(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_num_of_router().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_coprocessor_version(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_coprocessor_version().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_ba_id(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_ba_id().send().await.map_err(map_progenitor)?)
    }
    pub async fn node_info(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_node_information().send().await.map_err(map_progenitor)?)
    }
    pub async fn set_node_state(&self, state: &str) -> Result<serde_json::Value> {
        into_value(
            self.inner.set_node_state().body(state.to_string()).send().await.map_err(map_progenitor)?,
        )
    }
}
```

If `get_node_information` requires the `AcceptJsonApi` header (the generated builder exposes a required setter), pass `.accept("application/json")` per the Task 2 discovery. If a builder name differs, substitute it.

- [ ] **Step 4: Extend `NodeCmd` and dispatch**

```rust
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
```

In `run`, match each arm to the matching facade method; for `State { set }`, call `set_node_state(&s)` when `set` is `Some(s)`, else `node_state`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test cli_node`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/facade.rs src/cli tests/cli_node.rs
git commit -m "Add node info and state commands"
```

---

### Task 9: Dataset and commissioner commands

**Files:**
- Modify: `src/facade.rs`
- Modify: `src/cli/mod.rs`
- Create: `src/cli/node_ext.rs`
- Test: `tests/cli_dataset.rs`

**Interfaces:**
- Consumes: generated client builders + `types::{ActiveDataset, PendingDataset, JoinerData}`.
- Produces facade methods: `dataset_active`, `dataset_pending` (GET); `set_dataset_active(body: Value)`, `set_dataset_pending(body: Value)` (deserialise into the typed dataset, PUT); `commissioner_state` (GET); `set_commissioner_state(state: &str)` (PUT string); `commissioner_joiner` (GET); `add_commissioner_joiner(body: Value)` (deserialise into `types::JoinerData`, POST); `remove_commissioner_joiner(spec: &str)` (DELETE string body). Adds `Command::Dataset` and `Command::Commissioner`; `--set <FILE|->` reads JSON from a file or stdin.

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn dataset_active_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/dataset/active");
        then.status(200).header("content-type", "application/json")
            .body(r#"{"NetworkName":"OpenThread"}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "dataset", "active"]);
    cmd.assert().success().stdout(predicates::str::contains("OpenThread"));
    m.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_dataset`
Expected: FAIL.

- [ ] **Step 3: Add facade methods**

```rust
impl OtbrClient {
    pub async fn dataset_active(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_active_dataset().send().await.map_err(map_progenitor)?)
    }
    pub async fn dataset_pending(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_pending_dataset().send().await.map_err(map_progenitor)?)
    }
    pub async fn set_dataset_active(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let typed: types::ActiveDataset =
            serde_json::from_value(body).map_err(|e| Error::Parse(e.to_string()))?;
        into_value(self.inner.set_active_dataset().body(typed).send().await.map_err(map_progenitor)?)
    }
    pub async fn set_dataset_pending(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let typed: types::PendingDataset =
            serde_json::from_value(body).map_err(|e| Error::Parse(e.to_string()))?;
        into_value(self.inner.set_pending_dataset().body(typed).send().await.map_err(map_progenitor)?)
    }
    pub async fn commissioner_state(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_commissioner_state().send().await.map_err(map_progenitor)?)
    }
    pub async fn set_commissioner_state(&self, state: &str) -> Result<serde_json::Value> {
        into_value(
            self.inner.set_commissioner_state().body(state.to_string()).send().await.map_err(map_progenitor)?,
        )
    }
    pub async fn commissioner_joiner(&self) -> Result<serde_json::Value> {
        into_value(self.inner.get_commissioner_joiner().send().await.map_err(map_progenitor)?)
    }
    pub async fn add_commissioner_joiner(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        let typed: types::JoinerData =
            serde_json::from_value(body).map_err(|e| Error::Parse(e.to_string()))?;
        into_value(self.inner.add_commissioner_joiner().body(typed).send().await.map_err(map_progenitor)?)
    }
    pub async fn remove_commissioner_joiner(&self, spec: &str) -> Result<serde_json::Value> {
        into_value(
            self.inner.remove_commissioner_joiner().body(spec.to_string()).send().await.map_err(map_progenitor)?,
        )
    }
}
```

- [ ] **Step 4: Add subcommands in `src/cli/node_ext.rs` and wire into `Command`**

```rust
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
```

Add `mod node_ext;` to `src/cli/mod.rs`; add `Dataset { #[command(subcommand)] what: node_ext::DatasetCmd }` and `Commissioner { #[command(subcommand)] what: node_ext::CommissionerCmd }` to `Command`. Dispatch: dataset/commissioner `--set` arms call `read_json_input` then the typed `set_*`; `State { set }` for commissioner calls `set_commissioner_state(&s)` (the body is a bare string, passed directly, not JSON-parsed); `JoinerRemove { value }` passes `value` to `remove_commissioner_joiner`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test cli_dataset`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/facade.rs src/cli tests/cli_dataset.rs
git commit -m "Add dataset and commissioner commands"
```

---

### Task 10: Devices and diagnostics commands

**Files:**
- Modify: `src/facade.rs`
- Modify: `src/cli/mod.rs`
- Create: `src/cli/devices.rs`
- Test: `tests/cli_devices.rs`

**Interfaces:**
- Consumes: generated builders `list_devices`, `get_device`, `delete_device`, `delete_devices`, `list_diagnostics`, `get_diagnostic`, `delete_diagnostics`.
- Produces facade methods: `devices_list`, `device_get(id)`, `device_delete(id)`, `devices_delete_all`, `diagnostics_list`, `diagnostic_get(id)`, `diagnostics_delete_all`. Adds `Command::Devices` and `Command::Diagnostics`.

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn devices_list_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/api/devices");
        then.status(200).header("content-type", "application/json").body(r#"{"data":[]}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "--json", "devices", "list"]);
    cmd.assert().success().stdout(predicates::str::contains("data"));
    m.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_devices`
Expected: FAIL.

- [ ] **Step 3: Add facade methods**

```rust
impl OtbrClient {
    pub async fn devices_list(&self) -> Result<serde_json::Value> {
        into_value(self.inner.list_devices().send().await.map_err(map_progenitor)?)
    }
    pub async fn device_get(&self, id: &str) -> Result<serde_json::Value> {
        into_value(self.inner.get_device().device_id(id).send().await.map_err(map_progenitor)?)
    }
    pub async fn device_delete(&self, id: &str) -> Result<serde_json::Value> {
        into_value(self.inner.delete_device().device_id(id).send().await.map_err(map_progenitor)?)
    }
    pub async fn devices_delete_all(&self) -> Result<serde_json::Value> {
        into_value(self.inner.delete_devices().send().await.map_err(map_progenitor)?)
    }
    pub async fn diagnostics_list(&self) -> Result<serde_json::Value> {
        into_value(self.inner.list_diagnostics().send().await.map_err(map_progenitor)?)
    }
    pub async fn diagnostic_get(&self, id: &str) -> Result<serde_json::Value> {
        into_value(self.inner.get_diagnostic().diagnostics_id(id).send().await.map_err(map_progenitor)?)
    }
    pub async fn diagnostics_delete_all(&self) -> Result<serde_json::Value> {
        into_value(self.inner.delete_diagnostics().send().await.map_err(map_progenitor)?)
    }
}
```

- [ ] **Step 4: Add subcommands in `src/cli/devices.rs` and wire into `Command`**

```rust
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DevicesCmd {
    List,
    Get { id: String },
    Delete { id: String },
    DeleteAll,
}

#[derive(Subcommand)]
pub enum DiagnosticsCmd {
    List,
    Get { id: String },
    DeleteAll,
}
```

Add `mod devices;`, add `Devices { #[command(subcommand)] what: devices::DevicesCmd }` and `Diagnostics { #[command(subcommand)] what: devices::DiagnosticsCmd }` to `Command`; dispatch each arm to the matching facade method.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test cli_devices`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/facade.rs src/cli tests/cli_devices.rs
git commit -m "Add devices and diagnostics commands"
```

---

### Task 11: Actions commands with --no-wait

**Files:**
- Modify: `src/facade.rs`
- Modify: `src/cli/mod.rs`
- Create: `src/cli/actions.rs`
- Test: `tests/cli_actions.rs`

**Interfaces:**
- Consumes: `submit_action`, `submit_and_wait`, generated `list_actions`, `get_action`, `delete_actions`.
- Produces facade methods: `actions_list`, `actions_delete_all` (and the existing `action_get`/`submit_action`/`submit_and_wait`). Adds `Command::Actions` with `Submit { file, no_wait }`, `List`, `Get { id }`, `DeleteAll`. Submit reads a JSON task body from a file or stdin; default polls via `submit_and_wait` (1s poll, 60s timeout), `--no-wait` prints the submitted action id.

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn actions_submit_no_wait_prints_id() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(201).header("content-type", "application/json").body(
            r#"{"data":[{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"pending"}}]}"#,
        );
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "actions", "submit", "--no-wait", "--file", "-"]);
    cmd.write_stdin(r#"{"data":[]}"#);
    cmd.assert().success().stdout(predicates::str::contains("abc"));
    m.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_actions`
Expected: FAIL.

- [ ] **Step 3: Add facade methods**

```rust
impl OtbrClient {
    pub async fn actions_list(&self) -> Result<serde_json::Value> {
        into_value(self.inner.list_actions().send().await.map_err(map_progenitor)?)
    }
    pub async fn actions_delete_all(&self) -> Result<serde_json::Value> {
        into_value(self.inner.delete_actions().send().await.map_err(map_progenitor)?)
    }
}
```

- [ ] **Step 4: Add `src/cli/actions.rs` and wire into `Command`**

```rust
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
```

Add `mod actions;` and `Actions { #[command(subcommand)] what: actions::ActionsCmd }` to `Command`. Dispatch `Submit`:

```rust
ActionsCmd::Submit { file, no_wait } => {
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
```

Dispatch `List`/`Get { id }`/`DeleteAll` to `actions_list` / `action_get(&id)` / `actions_delete_all`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test cli_actions`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/facade.rs src/cli tests/cli_actions.rs
git commit -m "Add actions commands with --no-wait"
```

---

### Task 12: Gated live integration tests

**Files:**
- Create: `tests/live.rs`

**Interfaces:**
- Consumes: the built `otctl` binary.
- Produces: tests that run only when `OTBR_TEST_URL` is set.

- [ ] **Step 1: Write the gated tests**

```rust
use assert_cmd::Command;

fn live_url() -> Option<String> {
    std::env::var("OTBR_TEST_URL").ok()
}

#[test]
fn live_node_state() {
    let Some(url) = live_url() else {
        eprintln!("skipping: OTBR_TEST_URL not set");
        return;
    };
    Command::cargo_bin("otctl").unwrap()
        .args(["--base-url", &url, "node", "state"])
        .assert().success();
}

#[test]
fn live_dataset_active_json() {
    let Some(url) = live_url() else {
        eprintln!("skipping: OTBR_TEST_URL not set");
        return;
    };
    Command::cargo_bin("otctl").unwrap()
        .args(["--base-url", &url, "--json", "dataset", "active"])
        .assert().success().stdout(predicates::str::contains("{"));
}
```

- [ ] **Step 2: Verify they pass offline (skip path)**

Run: `cargo test --test live`
Expected: PASS (both print "skipping" and return).

- [ ] **Step 3: Verify against a live otbr (manual)**

Run: `OTBR_TEST_URL=http://127.0.0.1:8081 cargo test --test live -- --nocapture`
Expected: PASS against the running `otbr` container (`docker logs --tail 30 otbr` to confirm it is up; `curl -s http://127.0.0.1:8081/node` as a sanity check). If the container is down, this is the only step expected to fail; note it and move on.

- [ ] **Step 4: Commit**

```bash
git add tests/live.rs
git commit -m "Add gated live integration tests"
```

---

### Task 13: Full build, lint, and README

**Files:**
- Create: `README.md`

**Interfaces:**
- Consumes: the whole crate.
- Produces: usage docs and a clean final build.

- [ ] **Step 1: Write `README.md`**

Document: what `otctl` is, the `--base-url`/`OTBR_URL`/`--json` globals, one example per command group (`node state`, `dataset active`, `commissioner joiner add --file j.json`, `devices list`, `actions submit --file task.json`), and the `docker logs --tail 30 otbr` / `curl -s http://127.0.0.1:8081/node` reference commands for sanity-checking a running node.

- [ ] **Step 2: Run the full test suite**

Run: `cargo test`
Expected: PASS (all unit + integration tests; live tests skip).

- [ ] **Step 3: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: no warnings, formatting clean. Fix any issues and re-run.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "Add README and finalise build"
```

---

## Self-Review

**Spec coverage:**
- Vendored hand-patched spec + fixes → Tasks 1, 2 (+ `docs/spec-sync.md`). ✓
- progenitor codegen, facade uses generated typed methods → Tasks 2, 4, 5, 8-11. ✓
- Node info/state → Tasks 7, 8. ✓
- Datasets + commissioning → Task 9. ✓
- Devices + diagnostics → Task 10. ✓
- Actions block+poll with `--no-wait` → Tasks 5, 11. ✓
- Human tables + `--json` → Task 6. ✓
- `--base-url` default `127.0.0.1:8081` + `OTBR_URL` → Task 7. ✓
- thiserror error mapping (incl. progenitor errors) + exit codes → Tasks 3, 7. ✓
- Gated live integration tests → Task 12. ✓
- Single crate lib + bin → Task 1. ✓

**Generated-name risk:** generated builder/type/param names are assumed from the `operationId` and spec parameter/schema names and confirmed in Task 2 Step 8; every facade task instructs substituting the discovered name if progenitor differs. The `body(typed)` builder shape and any required `accept` header are the two spots most likely to need adjustment, flagged in Tasks 5 and 8.

**Placeholder scan:** no TBD/TODO; every code step contains complete code. ✓

**Type consistency:** facade public methods return `Result<serde_json::Value>` (or `Result<String>` for `submit_action`); `run` returns `Result<String>`; `to_output(&Value, bool)`; `into_value`, `map_progenitor`, `parse_api_errors`, `exit_code` used consistently across tasks. ✓
