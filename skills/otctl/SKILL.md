---
name: otctl
description: Use when working with an OpenThread Border Router (OTBR) — Thread network diagnostics, commissioning/joiner management, dataset inspection, or device table lookups — during firmware development for Thread end devices, Matter server/controller development, or any task that needs to query or drive a live border router. Not specific to this repo — applies from any project.
---

# otctl

## Overview

`otctl` is a Rust CLI wrapping the [OTBR REST API](https://openthread.io/reference/border-router/rest-api) v0.3.0. It is installed on PATH (`otctl`, built from this repo) and is the preferred way to query or drive a live OpenThread Border Router — use it directly via Bash instead of raw `curl` or building an MCP server for this.

## When to use

- Debugging Thread end-device firmware against a real border router (network role, RLOC, ext address, device table).
- Commissioning/joining new devices (joiner add/remove, commissioner state).
- Matter server/controller development that needs Thread network state (active/pending dataset, leader data, diagnostics).
- Any ad-hoc "what is the border router seeing right now" question.

## Usage

Every subcommand supports `--json` for machine-readable output; without it, output is a human-readable table.

```bash
otctl --base-url <url> node state --json
otctl --base-url <url> devices list --json
```

Base URL: `--base-url` flag or `OTBR_URL` env var. Default is `http://127.0.0.1:8081`. No auth required.

The user's border router is reachable at `https://otbr.lan.lee-brunnen.de`.

## Commands

- `node` — read/set node properties (`state`, `rloc`, `ext-address`, `network-name`, `leader-data`, `num-of-router`, `info`, `reset --yes`)
- `dataset active|pending [--set <file>]` — read or set the active/pending operational dataset
- `commissioner state|joiner-list|joiner-add --file <file>|joiner-remove <eui-or-pskd>` — commissioner and joiner management
- `devices list [--no-refresh]|get <id>|delete <id>|delete-all` — device table
- `diagnostics list|get <id>|delete-all` — network diagnostics
- `actions submit --file <file> [--no-wait]|list|get <id>|delete-all` — long-running diagnostic/action tasks

## Gotchas

- `devices list` refreshes the device table via a network diagnostics task by default and can take up to ~90s; pass `--no-refresh` for an instant but possibly stale read.
- `actions submit` blocks and polls until the action completes (60s timeout) unless `--no-wait` is passed.
- `node reset` performs a factory reset and requires `--yes` or interactive confirmation — never pass `--yes` without explicit user instruction.
