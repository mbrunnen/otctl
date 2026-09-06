# 0001. Project language is English

* Status: accepted
* Date: 2026-09-06

## Context and Problem Statement

`otctl` is developed by German-speaking contributors, but wraps a widely used,
English-language upstream (OpenThread/OTBR). Code, comments, commit messages,
CLI help text, and documentation need a single, consistent language so
contributors and future maintainers of any background can work with the
codebase without switching between languages mid-file.

## Considered Options

* English throughout
* German throughout
* Mixed, chosen per file or per author

## Decision Outcome

Chosen option: "English throughout". All code, identifiers, comments, commit
messages, CLI output/help text, and documentation (including ADRs) in this
repository are written in English.

### Consequences

* Good, because it matches the upstream OpenThread/OTBR ecosystem and the
  Rust crate ecosystem this project depends on.
* Good, because it keeps the codebase approachable for non-German-speaking
  contributors.
* Bad, because German-speaking contributors write English for a project used
  only internally.
