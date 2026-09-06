# 0003. Every output command has a `--json` output

* Status: accepted
* Date: 2026-09-06

## Context and Problem Statement

`otctl` is both a tool for interactive use (human-readable tables) and a
building block for scripts and other programs that need to consume its
output reliably (e.g. CI, other CLIs, automation). A CLI that only prints
human-formatted tables forces scripts to scrape text, which is brittle
against formatting changes.

## Considered Options

* Human-readable output only
* JSON output only
* Human-readable output by default, with an opt-in `--json` flag on every
  command that produces output

## Decision Outcome

Chosen option: "Human-readable output by default, with an opt-in `--json`
flag on every command that produces output". The `--json` flag is defined
once, globally, on the top-level `Cli` (`src/cli/mod.rs`), so it applies
uniformly to every subcommand without needing to be repeated per command.
Rendering is centralised in `src/render.rs::to_output`, which switches
between a table/scalar rendering and `serde_json::to_string_pretty` based on
the flag.

### Consequences

* Good, because scripts get a stable, parseable output format without text
  scraping.
* Good, because the flag is defined once and automatically covers new
  subcommands, so it can't be forgotten on a new command.
* Bad, because command implementations must keep producing a `serde_json::Value`
  internally (rather than formatting strings directly) so both output modes
  stay available.
