# auto-sudo

**Repo:** https://github.com/the-robot-lives/auto-sudo

A Rust CLI plus shell wrapper generator for commands that should transparently use `sudo` only when configured rules say escalation is needed.

## Why

Blanket passwordless sudo is a security hole; typing `sudo` everywhere is friction. auto-sudo sits between the two: a data-driven config decides, per command and per argument, whether a given invocation actually needs root — and generated shell wrappers apply the escalation prefix automatically (with a visible `Auto Sudo <command>` notice before any password prompt).

Part of the Noizu utilities fleet (`Portfolio/Utilities/source/*`); also installed via the monorepo root `make install-utilities`.

## Getting Started

Prerequisites: Rust toolchain (for the CLI), zsh (wrapper generation).

```bash
make install       # installs to ~/.local/bin
source ~/.zshrc    # pick up generated wrappers
```

Configuration lives at `~/.config/auto-sudo/config.yaml` — see `config.example.yaml` (preserves the legacy behavior for `vim`, `chmod`, `chown`, `chgrp`).

## Commands

```bash
auto-sudo decide -- vim /etc/hosts
auto-sudo shell --shell zsh
auto-sudo sudoers print
auto-sudo sudoers write --file /etc/sudoers.d/auto-sudo
auto-sudo sudoers write --append --file /etc/sudoers.d/auto-sudo
auto-sudo sudoers toggle vim-root --off
auto-sudo sudoers toggle vim-root --on
auto-sudo sudoers refresh --file /etc/sudoers.d/auto-sudo
```

## How It Works

- `decide` prints only the prefix a shell wrapper should apply — `sudo `, `sudo -u user `, or an empty string. It never executes the wrapped command.
- Generated wrappers print a yellow `Auto Sudo <command>` notice to stderr before invoking a sudo-prefixed command, so it appears before any sudo password prompt.
- `sudoers write/refresh` renders the config into a scoped `/etc/sudoers.d/` drop-in, so matching commands can run passwordless while everything else keeps normal sudo behavior.

### Rule model

- File rules can select arguments by raw position, `position: any`, `--flag=value`, or `--flag value`.
- File checks include permissions, ownership, group membership, exact/wildcard paths, path prefixes, and path suffixes.
- `always_sudo: true` makes a command's wrapper always run through sudo while still respecting `allow_pipes`:

```yaml
commands:
  systemctl:
    wrap: true
    always_sudo: true
```

- Command-level `sudo:` targets a specific user/group for always-sudo wrappers.

## Development

```bash
make test
make install
```

Repo layout: `rust/` (CLI crate), `auto-sudo.zsh` (wrapper generation), `config.example.yaml`.
