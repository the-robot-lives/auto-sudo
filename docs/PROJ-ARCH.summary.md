## auto-sudo - Architecture Summary

- **What**: Rust CLI plus generated zsh/bash wrappers for YAML-configured sudo decisions; `sudoers` subcommand emits checksum-pinned NOPASSWD entries for passwordless escalation.
- **Config**: Reads `~/.config/auto-sudo/config.yaml` by default.
- **Decision flow (fixed order)**: pipe policy → `always_sudo` → rules (top-to-bottom, first match wins). Wrapper detects pipe context, calls `auto-sudo decide`, prints a yellow notice when sudo will be used, then prefixes the real command with `sudo`, `sudo -u user [-g group]`, or nothing.
- **Rules**: Match always, positional file args, `--flag=value`, `--flag value`; path filters (wildcard/prefix/suffix) gate access, ownership, and group checks. Missing files never pass access checks — only parent-directory checks match absent paths.
- **sudoers**: `auto-sudo sudoers` prints, writes (temp file + `visudo -cf` + atomic rename, `--append` optional), toggles managed entries by id, refreshes, and validates. Generated lines carry a `sha256:<base64>` digest constraint.
- **Install**: local `make install` → binary to `~/.local/bin`, loader to `~/.local/share/auto-sudo`, config seeded to `~/.config/auto-sudo/config.yaml` (never overwritten), source line appended to `~/.zshrc`.
- **Ecosystem**: standalone Rust utility in the monorepo (dual path `Portfolio/Utilities/source/auto-sudo` / `utilities/`); not part of `make install-utilities`, no `k8-lib` or `.infra-config.yaml` dependency, but follows the `~/.local/bin` install convention.
