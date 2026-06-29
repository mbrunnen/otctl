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
