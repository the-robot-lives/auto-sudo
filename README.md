auto-sudo is a Rust CLI plus shell wrapper generator for commands that should
transparently use `sudo` only when configured rules say escalation is needed.

The default config lives at:

```bash
~/.config/auto-sudo/config.yaml
```

Install:

```bash
make install
source ~/.zshrc
```

Primary commands:

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

`decide` prints only the prefix that a shell wrapper should apply: `sudo `,
`sudo -u user `, or an empty string. It does not execute the wrapped command.
Generated wrappers print a yellow `Auto Sudo <command>` notice to stderr before
invoking a sudo-prefixed command, so it appears before any sudo password prompt.

`config.example.yaml` preserves the old behavior for `vim`, `chmod`, `chown`,
and `chgrp`, but the behavior is now data-driven.

File rules can select arguments by raw position, `position: any`,
`--flag=value`, or `--flag value`. File checks include permissions, ownership,
group membership, exact/wildcard paths, path prefixes, and path suffixes.

Use `always_sudo: true` on a command to make its generated function always run
through sudo while still respecting `allow_pipes`:

```yaml
commands:
  systemctl:
    wrap: true
    always_sudo: true
```

Set command-level `sudo:` when an always-sudo wrapper should run as a specific
user or group.
