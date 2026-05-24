# altered-state

you broke the domain. altered-state doesn't care: snapshot, diff, fix, done. reset...restart...replay!

<div align=center>
  <img height="150" alt="logo" src="https://github.com/investigato/altered-state/blob/main/assets/explain_ad.png" />
</div>

> functional, opinionated, and not done. use accordingly.

## why

because some things are better when learned together. i wanted one machine that could run a bunch of different ADCS scenarios...just not all at once. needed a way to:

- save the state of each one
- switch between them cleanly

## how

connects to your DC over LDAP (Kerberos/GSSAPI), snapshots the full directory to a compressed binary, and tracks it by name. when you're ready to switch scenarios or reset, it diffs current AD state against the target snapshot and generates the PowerShell to close the gap.

## build

```powershell
cargo build --release
```

build script copies `wwwroot/` and `config.json` next to the binary automatically.

## configuration

copy `config.example.json` to `config.json` in the same directory as the binary:

```json
{
  "domain": "example.com",
  "hostname": "dc01.example.com",
  "never_touch_these_attributes": [  // not kidding, you will screw things up
    "certificatetemplates",
    "cn",
    "objectcategory"
  ]
}
```

| field | description |
| ----- | ----------- |
| `domain` | AD domain FQDN |
| `hostname` | domain controller FQDN used for LDAP bind |
| `never_touch_these_attributes` | excluded from comparison and remediation. leave these alone. |

## commands

### `init` —> start here

snapshot current AD state as the `default` scenario baseline. run this on a clean lab before you touch anything.

```powershell
altered-state.exe init [--overwrite] [--template <path>]
```

### `new` —> name a scenario

captures a new baseline snapshot under a custom name. optionally init from a template (copies hooks, image, exclusions).

```powershell
altered-state.exe new --name <name> [--description <desc>] [--template <path>] [--overwrite]
```

### `activate` —> apply a scenario

diffs current AD state against the scenario snapshot and executes whatever PowerShell is needed to get there.

```powershell
altered-state.exe activate --scenario <name> [--state baseline|current|working|snapshot]
```

### `reset` —> something broke. run this

restores the active scenario to its saved baseline state.

```powershell
altered-state.exe reset --name <name>
```

### `serve` —> web ui for players

spins up a minimal web interface so players can switch and reset scenarios without touching the CLI. they get `activate` and `reset`. nothing else.

```powershell
altered-state.exe serve [--port 5000]
```

### global flags

```powershell
--config <path>    path to config.json (defaults to executable directory)
-v / -vv / -vvv   verbosity
```

## scenario structure

each scenario lives under `scenarios/<name>/`:

```sh
scenarios/
  esc1/
    config.json       # scenario config: hooks, image, exclusions
    baseline.bin      # compressed LDAP snapshot
    activation.ps1    # runs on activation
    cleanup.ps1       # runs on cleanup
    esc1.jpg          # optional scenario image
```

### config.json

```json
{
  "name": "esc1",
  "description": "ADCS ESC1 misconfiguration",
  "image_path": "...",
  "hooks": [
    {
      "hook_type": "Activation",
      "path": "C:\\path\\to\\activation.ps1",
      "arguments": [],
      "continue_on_error": true
    },
    {
      "hook_type": "Cleanup",
      "path": "C:\\path\\to\\cleanup.ps1",
      "arguments": [],
      "continue_on_error": true
    }
  ],
  "exclusions": []
}
```

hook types: `Activation`, `Cleanup`, `PreAction`

## workflow (lab creators)

```powershell
# 1. clean slate
altered-state.exe init

# 2. configure your scenario, then snapshot it, activates automatically
altered-state.exe new --name esc1 --template templates/esc1/config.json

# 3. back to baseline
altered-state.exe reset --name default

# 4. update a scenario, just overwrite it
altered-state.exe new --name esc1 --template templates/esc1/config.json --overwrite
```
