# OTBR REST CLI (`otctl`) — Design

Date: 2026-06-29
Status: Approved for planning

## Problem

The OpenThread Border Router (`otbr-agent`, `OTBR_REST=ON`) exposes a REST API on
`:8081`. There is no convenient typed Rust client or command-line tool for
inspecting and operating a running otbr node. We want a CLI for everyday
inspection, network setup, device/diagnostics queries, and the async Actions
task queue, backed by a reusable typed client.

Reference environment: an `otbr` Docker container, REST API reachable at
`http://127.0.0.1:8081` (`curl -s http://127.0.0.1:8081/node` works).

## Source API

Upstream spec: `openthread/ot-br-posix` `src/rest/openapi.yaml`
(forked from commit `ec16e396382b4559e70a2c6fdeecb7d596a5e915`,
"Swagger OpenThread REST API" v0.3.0, OpenAPI 3.0.3).

The API has two coexisting styles:

1. **`/api/*` — JSON:API** (`application/vnd.api+json`, sparse fieldsets,
   async task queue): `Actions`, `Devices`, `Diagnostics`, `/api/node`.
2. **`/node/*` — flat plain-JSON**: `state`, `rloc`, `rloc16`, `ext-address`,
   `network-name`, `leader-data`, `ext-panid`, `num-of-router`, `coprocessor/version`,
   `ba-id`, `dataset/active`, `dataset/pending`, `commissioner/state`,
   `commissioner/joiner`, factory reset (`DELETE /node`).

## Decisions

| Decision | Choice |
|---|---|
| Deliverable | CLI tool with a reusable client library underneath |
| Language | Rust |
| Client generation | progenitor, from a **vendored hand-patched** spec (approach C) |
| Scope (v1) | All four areas: node info/state, datasets/commissioning, devices/diagnostics, Actions |
| Output | Human tables/key-values by default; global `--json` for raw response |
| Async Actions | Block + poll until done by default; `--no-wait` returns the action ID |
| Crate layout | Single crate, `src/lib.rs` (client) + `src/bin/otctl.rs` (CLI) |

## Why the spec must be patched

progenitor cannot consume the upstream spec as-is. Four blockers, all fixed by
hand in the vendored copy:

1. **`application/vnd.api+json` content type** (all `/api/*`): progenitor only
   models `application/json` request/response bodies as typed; otherwise they
   become opaque byte bodies. Fix: rewrite the content-type keys to
   `application/json` in the vendored spec.
2. **22 `/node/*` operations missing `operationId`**: progenitor names methods
   from `operationId`. Fix: add explicit, stable `operationId`s
   (e.g. `getNodeState`, `setNodeState`, `getActiveDataset`, …).
3. **`oneOf` (10) / `anyOf` (5)** in the polymorphic Actions task schemas:
   `anyOf` is poorly supported. Fix: flatten/normalise the worst offenders in
   the vendored spec; if any Actions request body still defeats progenitor,
   hand-model that single type in the facade and submit it as a typed value.
4. **`deepObject` param style** (JSON:API sparse fieldsets, `fields[type]=`):
   limited progenitor support. Fix: drop these query params from the vendored
   spec and pass sparse-fieldset queries via a small manual query-string helper
   in the facade where needed (v1 may simply not expose sparse fieldsets).

The vendored spec is committed and diffable. `docs/spec-sync.md` records the
upstream SHA and the reconciliation procedure.

## Architecture

Single crate `otctl`:

```
otctl/
  Cargo.toml
  openapi/ot-rest.yaml        # vendored, hand-patched spec (source of truth)
  src/
    lib.rs                    # re-exports: generated client + facade
    generated.rs              # progenitor::generate_api!("openapi/ot-rest.yaml") — no build.rs needed
    facade.rs                 # ergonomic wrapper: config, Actions poll helper
    error.rs                  # thiserror Error enum, API-error mapping
    bin/
      otctl.rs                # clap CLI entry
    cli/
      mod.rs                  # command tree, dispatch
      node.rs                 # node info/state + datasets + commissioner
      devices.rs              # devices + diagnostics
      actions.rs              # actions submit/list/get/delete
      render.rs               # human table/key-value rendering + --json passthrough
```

### Layers and boundaries

- **generated** — progenitor output. Raw typed client; one method per
  `operationId`. Not used directly by the CLI.
- **facade** (`OtbrClient`) — the public library surface. Holds base URL +
  `reqwest` client, wraps the generated client, and adds:
  - `OtbrClient::new(base_url)` (default `http://127.0.0.1:8081`)
  - convenience methods returning facade-owned result types
  - `submit_and_wait(task, poll_interval, timeout)` for Actions
  - error normalisation via `error.rs`
- **cli** — clap derive command tree, calls the facade, hands results to
  `render`. No HTTP logic here.

### Command tree (clap derive)

```
otctl [--base-url URL] [--json] <command>

node state [--set <STATE>]      node rloc | rloc16 | ext-address | network-name
node leader-data | ext-panid | num-of-router | coprocessor-version | ba-id
node info                        # /api/node summary
node reset                       # DELETE /node (factory reset; confirm prompt)

dataset active [--set <FILE|->]
dataset pending [--set <FILE|->]

commissioner state [--set <STATE>]
commissioner joiner list
commissioner joiner add <args>
commissioner joiner remove <args>

devices list | get <id> | delete <id> | delete-all
diagnostics list | get <id> | delete-all

actions submit <task-args> [--no-wait]
actions list | get <id> | delete-all
```

Global flags: `--base-url` (env `OTBR_URL` fallback), `--json`.

### Data flow

`otctl node state` → `cli::node` → `facade.get_node_state()` →
generated client GET `/node/state` → facade maps response/error →
`render::emit(value, json_flag)` → stdout.

`otctl actions submit … ` (default) → facade `submit_and_wait` → POST
`/api/actions` → loop GET `/api/actions/{id}` until terminal status or timeout →
render final action item. With `--no-wait`: POST, print the action ID, return.

## Error handling

`error.rs` defines `thiserror`-based `Error`:
- `Http(reqwest::Error)` — transport/connection failures.
- `Api { status, errors }` — non-2xx; parse JSON:API `Errors`/`SimpleError`
  bodies into a structured list, fall back to raw text.
- `Timeout` — Actions poll exceeded the deadline.
- `Parse` — unexpected/unparseable body.

CLI maps `Error` to a clean stderr message and a non-zero exit code
(distinct codes: 1 generic, 2 usage, 3 API error, 4 timeout).

## Testing

- **Unit**: `render` (table + `--json` output), `error.rs` API-error parsing
  from sample JSON:API error bodies, `submit_and_wait` polling logic against a
  mock transport.
- **Spec/codegen**: a test asserts the vendored spec parses and progenitor
  generation succeeds (compile-time guarantee via the macro; an explicit
  `cargo build` in CI covers it).
- **Integration (gated)**: tests hitting a live otbr, enabled only when
  `OTBR_TEST_URL` is set (default skip), so `cargo test` stays green offline.
  Covers `node state`, `dataset active` round-trip read, `actions submit
  --no-wait`.

## Out of scope (v1)

- JSON:API sparse fieldsets / `fields[]` query params (deferred; see blocker 4).
- Authentication (the REST API is unauthenticated on the local interface).
- Publishing to crates.io; this stays a vendored/local crate.
- Shell completions and man pages (nice-to-have, later).

## Open risk

The Actions `anyOf` task schemas are the one place progenitor may still fail
after patching. Mitigation is local: hand-model the offending request type(s)
in the facade. This does not affect the rest of the design.
