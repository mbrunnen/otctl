# otctl

Command-line interface for the [OpenThread Border Router (OTBR) REST API](https://openthread.io/reference/border-router/rest-api) v0.3.0.

## Global flags

| Flag | Env | Default | Description |
|------|-----|---------|-------------|
| `--base-url` | `OTBR_URL` | `http://127.0.0.1:8081` | Base URL of the OTBR REST API |
| `--json` | | false | Output raw JSON instead of a human-readable table |

## Commands

### node

Read or set node properties. The `reset` subcommand performs a factory reset (DELETE /node); it requires `--yes` or interactive confirmation.

```
otctl node state
otctl node state --set leader
otctl node rloc
otctl node rloc16
otctl node ext-address
otctl node network-name
otctl node leader-data
otctl node ext-panid
otctl node num-of-router
otctl node coprocessor-version
otctl node ba-id
otctl node info
otctl node reset --yes
```

Example:

```sh
otctl node state
# leader
```

### dataset

Read or set the active or pending dataset.

```
otctl dataset active
otctl dataset active --set dataset.json
otctl dataset pending
otctl dataset pending --set dataset.json
```

Example:

```sh
otctl dataset active
# NetworkName   Channel  PanId
# OpenThread    15       0x4321
```

### commissioner

Manage the commissioner and joiner list.

```
otctl commissioner state
otctl commissioner state --set active
otctl commissioner joiner-list
otctl commissioner joiner-add --file joiner.json
otctl commissioner joiner-remove <eui-or-pskd>
```

Example:

```sh
otctl commissioner joiner-add --file joiner.json
# ok: true
```

Where `joiner.json` contains a `JoinerData` object, e.g.:

```json
{"eui": "*", "pskd": "J01NME"}
```

Pass `-` as the filename to read from stdin:

```sh
echo '{"eui":"*","pskd":"J01NME"}' | otctl commissioner joiner-add --file -
```

### devices

Manage the device table.

```
otctl devices list
otctl devices list --no-refresh
otctl devices get <id>
otctl devices delete <id>
otctl devices delete-all
```

`list` first refreshes the collection by submitting an `updateDeviceCollectionTask` and waiting for it to finish (network diagnostics; may take up to ~90 s), so the table reflects the live network. Pass `--no-refresh` to read the collection as-is, which is instant but may be stale or empty until a refresh has run.

Example:

```sh
otctl devices list
# ExtAddress         Rloc16  Role
# 0x1122334455667788  0x4400  router
```

### diagnostics

Read or clear network diagnostics.

```
otctl diagnostics list
otctl diagnostics get <id>
otctl diagnostics delete-all
```

### actions

Submit and track long-running actions.

```
otctl actions submit --file task.json
otctl actions submit --file task.json --no-wait
otctl actions list
otctl actions get <id>
otctl actions delete-all
```

`submit` blocks and polls until the action reaches `completed`, `stopped`, or `failed` (60 s timeout), unless `--no-wait` is given, in which case it returns the action ID immediately.

Example:

```sh
otctl actions submit --file task.json
# field       value
# id          f47ac10b-58cc-4372-a567-0e02b2c3d479
# type        getNetworkDiagnosticTask
# attributes  {"status":"completed"}

otctl actions submit --file task.json --json
# full JSON action item

otctl actions submit --file task.json --no-wait
# field  value
# id     f47ac10b-58cc-4372-a567-0e02b2c3d479
```

Pass `-` as the filename to read from stdin:

```sh
echo '{"data":[]}' | otctl actions submit --file -
```

## Sanity-check a running OTBR node

```sh
docker logs --tail 30 otbr
curl -s http://127.0.0.1:8081/node
```
