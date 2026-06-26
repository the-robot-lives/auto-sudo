## auto-sudo - Architecture Summary

- **What**: Rust CLI plus generated zsh/bash wrappers for YAML-configured sudo decisions.
- **Config**: Reads `~/.config/auto-sudo/config.yaml` by default.
- **Decision flow**: Shell wrapper detects pipe context, calls `auto-sudo decide`, then prefixes the real command with `sudo`, `sudo -u user`, or nothing.
- **Rules**: Match always, positional file args, `--flag=value`, `--flag value`, paths, ownership, group, and permission checks.
- **sudoers**: `auto-sudo sudoers` prints, writes, toggles, refreshes, and validates managed NOPASSWD entries.
