# otctl

Command-line interface for the [OpenThread Border Router (OTBR) REST API](https://openthread.io/reference/border-router/rest-api) v0.3.0.

## Installing

```sh
cargo install --path .
```

Installs `otctl` into `~/.cargo/bin`.

### Shell completion

`otctl completions <shell>` writes a completion script to stdout for `bash`,
`zsh`, `fish`, `elvish` or `powershell`. Where it belongs depends on the shell:

```sh
# zsh: any directory on $fpath, then rebuild the completion cache
otctl completions zsh > ~/.zfunc/_otctl
rm -f ~/.zcompdump && exec zsh

# bash
otctl completions bash > ~/.local/share/bash-completion/completions/otctl

# fish
otctl completions fish > ~/.config/fish/completions/otctl.fish
```

For zsh, `~/.zfunc` has to be on `$fpath` before `compinit` runs; add
`fpath+=~/.zfunc` to `~/.zshrc` above the `compinit` call if it is not already
there. Regenerate the script after upgrading `otctl`, as it is a snapshot of
the subcommands and flags of the version that produced it.

## Global flags

| Flag | Env | Default | Description |
|------|-----|---------|-------------|
| `--base-url` | `OTBR_URL` | `http://127.0.0.1:8081` | Base URL of the OTBR REST API |
| `--json` | | false | Output raw JSON instead of a human-readable table |

The base URL is resolved in this order: `--base-url` flag, then `OTBR_URL` env var, then the `base_url` in the config file, then the built-in default.

### config

Save a default base URL so you don't have to pass `--base-url` every time.

```
otctl config init
```

Prompts for a base URL, showing the currently configured value (or the built-in default) as the default you get by pressing Enter. Writes to `~/.config/otctl/config.toml` (or `$XDG_CONFIG_HOME/otctl/config.toml`).

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
otctl commissioner state --set enable
otctl commissioner joiner-list
otctl commissioner joiner-add --pskd J01NME
otctl commissioner joiner-remove <eui64-or-discerner>
otctl commissioner enroll --pskd J01NME --eui fedcba9876543210
```

A joiner is added either from flags or from a `JoinerData` JSON document.
The flag form covers the common cases:

```sh
otctl commissioner joiner-add --pskd J01NME
# ok: true

otctl commissioner joiner-add --pskd J01NME --eui64 0123456789abcdef --timeout 120
otctl commissioner joiner-add --pskd J01NME --discerner 0xabc/12
```

`--pskd` is required. `--joiner-id`, `--eui64` and `--discerner` are mutually
exclusive; without one of them the wildcard joiner id `*` is sent, which admits
any joiner presenting the PSKd. `--timeout` is the joiner expiration in seconds.

For fields the flags do not cover, pass a `JoinerData` object with `--file`
(mutually exclusive with the flags above):

```json
{"pskd": "J01NME", "joinerId": "*"}
```

Pass `-` as the filename to read from stdin:

```sh
echo '{"pskd":"J01NME","joinerId":"*"}' | otctl commissioner joiner-add --file -
```

#### enroll

`joiner-add` only fills the joiner table; petitioning the commissioner and
tearing it down again is left to the caller. `enroll` hands the whole sequence
to the border router as an `addThreadDeviceTask`: it starts the on-mesh
commissioner if needed, adds the joiner, waits for it to join, and stops the
commissioner again when the last such task ends.

```sh
otctl commissioner enroll --pskd J01NME --eui fedcba9876543210
# field       value
# id          f47ac10b-58cc-4372-a567-0e02b2c3d479
# type        addThreadDeviceTask
# attributes  {"status":"completed"}

otctl commissioner enroll --pskd J01NME --timeout 300
otctl commissioner enroll --pskd J01NME --discerner 0xabc/12 --no-wait
```

`--pskd` is required; `--joiner-id`, `--eui` and `--discerner` are mutually
exclusive and default to the wildcard `*`. Note the task attribute is `eui`,
not the `eui64` of the joiner endpoint. `--timeout` (default 900 s) is how long
the router waits for the joiner; the command waits that long plus a grace
period, or returns the action id right away with `--no-wait`. Track a detached
run with `otctl actions get <id>`.

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
otctl actions submit --file task.json --timeout 900
otctl actions submit --file task.json --no-wait
otctl actions list
otctl actions get <id>
otctl actions delete-all
```

`submit` blocks and polls until the action reaches `completed`, `stopped`, or `failed`, unless `--no-wait` is given, in which case it returns the action ID immediately. `--timeout` bounds the wait in seconds (default 60); raise it for tasks whose own `timeout` attribute is longer, otherwise the command gives up while the task keeps running on the router.

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
