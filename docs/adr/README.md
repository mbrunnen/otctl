# Architecture Decision Records

This directory holds Architecture Decision Records (ADRs) for `otctl`, using
the [MADR](https://adr.github.io/madr/) format.

An ADR captures a decision, its context, and the alternatives considered, so
future contributors understand *why* the project looks the way it does, not
just what it looks like today.

## Adding a new ADR

1. Copy `template.md` to `NNNN-short-title.md`, using the next sequence
   number.
2. Fill in the sections. Delete any optional section that doesn't apply.
3. Set the status to `proposed`, `accepted`, `deprecated`, or `superseded by
   ADR-000X`.
4. Open a PR. Once merged, the ADR is considered accepted (unless the status
   says otherwise).

## Index

| ADR | Title | Status |
| --- | --- | --- |
| [0001](0001-project-language-is-english.md) | Project language is English | accepted |
| [0002](0002-tdd-for-bugfixes.md) | Fix bugs with TDD (RED → GREEN) | accepted |
| [0003](0003-json-output-for-every-command.md) | Every output command has a `--json` output | accepted |
