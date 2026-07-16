## auto-sudo - Architecture Summary

- **What**: Rust CLI plus generated zsh/bash wrappers for YAML-configured sudo decisions.
- **Config**: Reads `~/.config/auto-sudo/config.yaml` by default.
- **Decision flow**: Shell wrapper detects pipe context, calls `auto-sudo decide`, prints a yellow notice when sudo will be used, then prefixes the real command with `sudo`, `sudo -u user`, or nothing.
- **Rules**: Match always, positional file args, `--flag=value`, `--flag value`, paths, ownership, group, and permission checks.
- **sudoers**: `auto-sudo sudoers` prints, writes, toggles, refreshes, and validates managed NOPASSWD entries.
- **Install**: local `make install` → binary to `~/.local/bin`, loader to `~/.local/share/auto-sudo`, config seeded to `~/.config/auto-sudo/config.yaml`, source line appended to `~/.zshrc`.
- **Ecosystem**: standalone Rust utility in `utilities/shell/`; not part of `make install-utilities`, no `k8-lib` or `.infra-config.yaml` dependency, but follows the `~/.local/bin` install convention.
