# 0002. Fix bugs with TDD (RED → GREEN)

* Status: accepted
* Date: 2026-09-06

## Context and Problem Statement

`otctl` talks to an external REST API (the OTBR border router) and is
exercised mostly through integration tests against a mocked server
(`tests/`, `httpmock`). Without a rule for how bugfixes are made, a fix can
land without a regression test, and the same bug can silently reappear during
a later refactor or spec re-sync.

## Considered Options

* Fix first, add a test afterwards if there's time
* Require a failing test before writing the fix (RED → GREEN)
* No formal process, rely on manual testing

## Decision Outcome

Chosen option: "Require a failing test before writing the fix
(RED → GREEN)". Every bugfix starts with a test that reproduces the bug and
fails (RED), then the minimal code change to make it pass (GREEN). The test
is committed together with the fix.

### Consequences

* Good, because every bugfix leaves behind a regression test, preventing the
  same bug from resurfacing.
* Good, because a failing-first test proves the bug is actually reproduced
  and the fix actually addresses it.
* Bad, because it adds upfront effort for bugs that are hard to reproduce in
  the mocked test environment.
