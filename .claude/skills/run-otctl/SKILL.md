---
name: run-otctl
description: Build, run, and drive otctl against a mock OTBR REST server. Use when asked to run otctl, test it end-to-end, verify a CLI change works, or exercise its subcommands without a real border router.
---

`otctl` is a Rust CLI (no GUI, no server of its own) that talks outbound to an
OTBR REST API via `--base-url`/`OTBR_URL`. There is no real border router in
this container, so the driver is `.claude/skills/run-otctl/mock_otbr.py` — a
stdlib-only Python stand-in for the OTBR REST API — plus
`.claude/skills/run-otctl/smoke.sh`, which builds the binary, starts the
mock, and runs a representative set of subcommands against it.

All paths below are relative to the repo root.

## Prerequisites

None beyond a Rust toolchain (`cargo`/`rustc`, this container has 1.90.0) and
`python3` (stdlib only, no pip packages) — both already present here.

## Build

```bash
cargo build          # debug build used by smoke.sh -> target/debug/otctl
cargo build --release
```

## Run (agent path)

```bash
bash .claude/skills/run-otctl/smoke.sh
```

This builds `target/debug/otctl`, starts `mock_otbr.py` on `127.0.0.1:8123`
(override with `OTCTL_MOCK_PORT`), waits for its `MOCK-OTBR-READY` marker,
runs `node info`, `node state` (table and `--json`), `dataset active`,
`devices list --no-refresh`, `diagnostics list`, `actions submit` (via
stdin), and `node reset --yes`, then kills the mock server. `set -euo
pipefail` means any failing command aborts the script non-zero — a clean
exit and "== all commands succeeded ==" means the CLI's HTTP layer, JSON
rendering, table rendering, and error paths against a fresh binary all work.

To poke it manually instead of running the whole script:

```bash
python3 .claude/skills/run-otctl/mock_otbr.py 8123 &   # prints MOCK-OTBR-READY
cargo build
./target/debug/otctl --base-url http://127.0.0.1:8123 node info
./target/debug/otctl --base-url http://127.0.0.1:8123 --json devices list --no-refresh
kill %1
```

`mock_otbr.py` covers: `/node` (+ `state`, `rloc`, `rloc16`, `ext-address`,
`network-name`, `leader-data`, `ext-panid`, `num-of-router`, `ba-id`,
`coprocessor/version`, `DELETE /node`), `/node/dataset/{active,pending}`,
`/node/commissioner/state`, `/node/commissioner/joiner`, `/api/devices`,
`/api/diagnostics`, and `/api/actions` (+ `/api/actions/{id}`, which always
reports `status: completed` — good enough to exercise `otctl actions submit`
and the `devices list` refresh-then-read flow without a real poll loop).
It does not model per-device/per-diagnostic bodies beyond echoing the id, or
validate the JSON:API `attributes` schema on `POST /api/actions` — this is a
CLI smoke harness, not a spec-conformance mock.

## Run (human path)

There is no interactive/long-running mode to speak of — every invocation is
one-shot. Point `--base-url` at a real OTBR (`otctl node state`) or use the
mock above.

## Test

```bash
cargo test
```

7 test binaries, all pass in this container. `tests/live.rs` needs a real
OTBR reachable at `OTBR_TEST_URL` and otherwise prints "skipping" and passes
trivially — it is not exercised by the mock driver above.

## Gotchas

- **`POST /api/actions` request bodies are validated against an untagged
  enum** generated from the OpenAPI spec (`types::ActionsClient`) — a task
  `type` with attributes that don't exactly match that task type's schema
  (e.g. `getNetworkDiagnosticTask` with `updateDeviceCollectionTask`-shaped
  attributes) fails client-side with `data did not match any variant of
  untagged enum ActionsClientDataItem`, before any HTTP request is even
  sent. `smoke.sh` and the existing Rust tests sidestep this by submitting
  `{"data":[]}` (a valid, empty task list) rather than a real task body.
- **`devices list` normally blocks up to ~100s** submitting an
  `updateDeviceCollectionTask` and polling until it leaves `pending` — the
  mock always answers `completed` immediately on `GET /api/actions/{id}`,
  so this returns instantly in the driver. Pass `--no-refresh` if you want
  to skip the refresh entirely (also instant, but against a real device it
  returns whatever the border router's device table currently holds,
  possibly empty/stale).
- **`node reset` prompts for interactive confirmation** unless `--yes` is
  passed; it reads a line from stdin and requires the literal text `yes`.
  `smoke.sh` always passes `--yes` — never do this against a real border
  router without the user's explicit go-ahead, since it factory-resets it.
