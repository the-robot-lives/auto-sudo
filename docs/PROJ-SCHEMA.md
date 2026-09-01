# Project Schema

> **No persistence layer.** auto-sudo has **no database and no SQL schema** — no
> migrations, no Liquibase changelogs, no tables. Its data artifacts are files:
> a YAML rule config it consumes, a sudoers.d snippet it generates, and shell
> wrapper functions it emits. Those file formats are the "schema" documented
> below.

## Data Artifacts Overview

| Artifact | Direction | Location | Producer/Consumer |
|----------|-----------|----------|-------------------|
| `config.yaml` | consumed | `~/.config/auto-sudo/config.yaml` | read by all subcommands |
| sudoers.d snippet | generated | `/etc/sudoers.d/auto-sudo` (default) | `sudoers print/write/refresh/toggle/check` |
| shell wrappers | generated | stdout (eval'd by `auto-sudo.zsh`) | `shell --shell zsh\|bash` |
| `.zshrc` source line | append-only | `~/.zshrc` | `make install` |

## config.yaml (rule config)

Loaded by `rust/src/config.rs` (`serde_yaml`), schema mirrors those structs.
Top level:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `version` | int | No | — | config format version (currently `1`; not enforced) |
| `defaults` | map | No | empty | fallback values for all commands |
| `commands` | map name → CommandConfig | No | `{}` | one entry per wrapped command (BTreeMap, sorted) |

### defaults

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `allow_pipes` | bool | `false` | global pipe policy fallback |
| `sudo` | SudoSpec | `{mode: root}` | default sudo target |

### commands.<name> (CommandConfig)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `wrap` | bool | `true` | emit a shell function for this command |
| `allow_pipes` | bool | `defaults.allow_pipes` (`false`) | permit sudo when piped |
| `always_sudo` | bool | `false` | always escalate, ignoring rules |
| `sudo` | SudoSpec | inherited from `defaults.sudo` | escalation target |
| `rules` | list of Rule | `[]` | evaluated top to bottom, **first match wins** |

### SudoSpec

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mode` | `root` \| `user` | `root` | escalation mode |
| `user` | string | — | required when `mode: user` (`decide` errors without it) |
| `group` | string | — | optional `-g` group |

Resolution order: `rule.action.sudo` → `command.sudo` → `defaults.sudo` → default (`root`).

### rules[] (Rule)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | — | human label, shown by `decide --explain` |
| `action` | `{sudo: SudoSpec}` | — | per-rule sudo target override |
| `args` | ArgSpec | `{}` | how to extract file operands from argv |
| `when` | When | required | match condition |

### args.files[] (FileArgSpec)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `position` | `"any"` or 0-based int | — | which argv entries are file operands |
| `flag` | string (e.g. `--file`) | — | extract values from `--flag=value` or `--flag value` |
| `skip_prefixes` | list of strings | `[]` | ignore argv entries starting with these (e.g. `["-"]`) |

### when (When)

| Field | Type | Description |
|-------|------|-------------|
| `always` | bool | match unconditionally (e.g. `chown`) |
| `any_file` | FileChecks | true if **any** extracted file passes all checks |
| `all_files` | FileChecks | true if ≥1 file extracted and **all** pass |

### FileChecks (any_file / all_files body)

All default `false`; empty-string filters (`paths`, `path_prefixes`,
`path_suffixes`) are no-ops. Path filters use `*`/`?` wildcard matching
(hand-rolled in `decision.rs`, not glob).

| Field | Meaning |
|-------|---------|
| `paths` | wildcard-glob path list (e.g. `/etc/*`) |
| `path_prefixes` | raw string prefixes (`/var/log/`) |
| `path_suffixes` | raw string suffixes |
| `exists` / `missing` | existence |
| `exists_not_writable` | file exists and current user cannot write it |
| `missing_parent_not_writable` | file absent and parent dir not writable |
| `missing_parent_not_readable` | file absent and parent dir not readable |
| `current_user_can_read` / `can_write` / `can_execute` | positive access |
| `current_user_cannot_read` / `cannot_write` / `cannot_execute` | negative access (skipped for missing files — see `decision.rs` guard) |
| `owner_is_current_user` / `owner_is_not_current_user` | uid comparison |
| `group_in_current_user_groups` / `group_not_in_current_user_groups` | gid membership |

Config convention: `config.example.yaml` uses **YAML anchors** (`&editor_command`,
`*write_file_command`) to share rule bodies across commands; `brew` ships
`wrap: false`.

## sudoers.d snippet (generated)

Produced by `rust/src/sudoers.rs` (`render`), default target
`/etc/sudoers.d/auto-sudo`, mode written via temp file + `rename` after
`visudo -cf` validation.

Per entry, two lines (plus blank line separator):

```
# AUTO-SUDO ENTRY id=<id> command=<name> path=<abspath> checksum=<sha256:...> [dev=D ino=I mtime=M]
<subject> ALL=(<runas>) NOPASSWD: <checksum> <path>
```

| Element | Format | Source |
|---------|--------|--------|
| `id` | `<sanitized-command>-<target>`; sanitize = non `[A-Za-z0-9_-]` → `-`; target = `root` or sudo user (`user` fallback) | `entry_id()` |
| `subject` | `$USER` env, falling back to `ALL` | `sudoers_subject()` |
| `runas` | `user` or `user:group` (`root` default) | `runas()` |
| `checksum` | `sha256:<base64(SHA-256 of binary)>` — used as a sudoers digest constraint | `checksum()` |
| metadata | optional `dev= ino= mtime=` from binary stat | `entry_for()` |

Entries come from: `always_sudo` commands, one per (command × sudo target) over
all rules (deduped by id), plus `--command` extras (root target).

**Toggle representation**: disabling an entry (`sudoers toggle <id> --off`)
comments out its rule line with `# ` (header line stays); `--on` strips the
prefix. Managed-file header: `# auto-sudo managed sudoers entries` +
validation hint.

**Write semantics**: default replaces the file; `--append` joins with existing
content (`trim_end` + `\n`). Both paths validate the temp file with
`visudo -cf` before the atomic rename.

## Generated shell wrappers

`shell --shell zsh|bash` emits one function per `wrap: true` command whose
name is a valid shell function name (`[A-Za-z_]` start, then
`[A-Za-z0-9_.-]`; others are skipped with a comment):

```zsh
<cmd>() {
  local _auto_sudo_prefix
  local _auto_sudo_pipe_args=()
  [[ ! -t 0 ]] && _auto_sudo_pipe_args+=(--stdin-piped)
  [[ ! -t 1 ]] && _auto_sudo_pipe_args+=(--stdout-piped)
  _auto_sudo_prefix="$(auto-sudo decide "${_auto_sudo_pipe_args[@]}" -- <cmd> "$@")" || return $?
  if [[ -n "$_auto_sudo_prefix" ]]; then
    printf '\033[1;33mAuto Sudo <cmd>\033[0m' >&2
    (( $# > 0 )) && printf ' %s' "$*" >&2
    printf '\n' >&2
    eval "command ${_auto_sudo_prefix}<cmd> \"\$@\""
  else
    command <cmd> "$@"
  fi
}
```

`decide` output contract: **exactly one prefix token stream on stdout** —
`sudo `, `sudo -u <user> [-g <group>] `, or empty; reason goes to stderr with
`--explain`.

## CLI grammar (clap)

```
auto-sudo decide [--config <path>] [--explain] [--stdin-piped] [--stdout-piped] -- <cmd> [args...]
auto-sudo shell [--shell zsh|bash] [--config <path>]
auto-sudo sudoers print  [--config <path>] [--command <cmd>]...
auto-sudo sudoers write  [--config <path>] [--file <path>] [--command <cmd>]... [--append]
auto-sudo sudoers refresh [--config <path>] [--file <path>] [--command <cmd>]... [--append]
auto-sudo sudoers toggle <entry-id> [--file <path>] (--on | --off)
auto-sudo sudoers check  [--file <path>]
```

`--file` defaults to `/etc/sudoers.d/auto-sudo` for all sudoers subcommands.
`refresh` is an alias of `write`.

## State files

None beyond the artifacts above — no caches, no lockfiles, no runtime state.
`rust/target/` and `Cargo.lock` are build/dependency artifacts, not data.
