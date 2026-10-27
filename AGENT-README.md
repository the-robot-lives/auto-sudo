# AGENT-README

> Orientation for AI coding agents working in this repo. Human docs: README.md.

## Type

rust-cli + scripts

## Purpose

Data-driven sudo wrapper generator: a Rust CLI decides, per command and per argument, whether an invocation actually needs root; generated zsh wrappers apply the escalation prefix automatically only when configured rules say so.

## Stack

Rust crate in `rust/` (clap-style subcommands: `decide`, `shell`, `sudoers`), zsh wrapper generation (`auto-sudo.zsh`), YAML rule config (`config.example.yaml`), Makefile.

## Dev mode

```bash
make install       # builds the CLI and installs to ~/.local/bin
source ~/.zshrc    # pick up generated wrappers
make test
```

## Ports / URLs

- none (library/CLI)

## Tests

```bash
make test
```

## Gotchas

- Live config sits at `~/.config/auto-sudo/config.yaml`; `config.example.yaml` preserves legacy behavior for `vim`, `chmod`, `chown`, `chgrp`.
- `decide` prints only the prefix (`sudo `, `sudo -u user `, or empty) — it never executes the wrapped command.
- `sudoers write/refresh` renders a scoped `/etc/sudoers.d/auto-sudo` drop-in — writing it changes host-wide passwordless sudo; treat as sensitive.
- Generated wrappers print a yellow `Auto Sudo <command>` notice to stderr before any sudo password prompt.
