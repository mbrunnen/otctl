# Spec sync

`openapi/ot-rest.yaml` is vendored from `openthread/ot-br-posix`.

- Upstream path: `src/rest/openapi.yaml`
- Forked at commit: `ec16e396382b4559e70a2c6fdeecb7d596a5e915`
- API title/version: Swagger OpenThread REST API, v0.3.0

## Local patches (do not lose these when re-syncing)

1. Content type `application/vnd.api+json` rewritten to `application/json` so progenitor models bodies as typed (20 occurrences).
2. Added `operationId` to the 22 `/node/*` operations.
3. `anyOf`-of-`$ref` unions changed to `oneOf` so progenitor emits enums (5 schemas).
4. Removed the `SparseFieldset` (`deepObject`) query parameter and its references.
5. Removed the secondary `DatasetTlv` request body from the dataset PUTs (v1 sets datasets via the JSON form only).
6. Removed `format: uuid` from the Actions id field and path parameters so they generate as plain `String` (avoids a uuid dependency).
7. Removed `additionalProperties: false` from all schema definitions (10 occurrences) as progenitor parses it as a boolean rather than a schema object.
8. De-duplicated `application/json` content entries created by patch 1, removing schema-less entries with examples only.
9. Removed `text/plain` content type from GET dataset responses to satisfy progenitor's single response type requirement.
10. Standardised inline error responses (400, 409) to component `$ref`s to prevent progenitor panic from mixed None/Type response types.
11. Removed OPTIONS operations (`optionsActions`, `optionsDevices`, `optionsDiagnostics`) as reqwest has no `.options()` method.
12. Removed `AcceptJsonApi` header parameter and all its `$ref` references (generated enum does not implement `TryFrom<HeaderValue>`).
13. Removed `pattern:` constraints from schemas and parameters (9 occurrences) to avoid requiring the `regress` crate dependency.
14. Removed invalid default values (`default: random` on integer fields, `default: not set` on integer field).
15. Removed the `default:` values from `JoinerData.joinerId` (`"*"`) and `JoinerData.timeout` (`60`). Progenitor materialises schema defaults into every serialised body, so `joinerId: "*"` was sent alongside `eui64`/`discerner` even though the spec declares the three mutually exclusive. Without the defaults the fields generate as `Option` and are omitted when unset; the CLI supplies the `"*"` wildcard explicitly instead.

## Re-sync procedure

1. Fetch the new upstream `openapi.yaml`.
2. Re-apply the patches above.
3. Run `cargo build` and fix any new progenitor incompatibilities.
4. Update the commit SHA in this file and in the spec header comment.
