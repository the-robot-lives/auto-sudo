# auto-sudo — How-To Task List

Companion index to [PROJ-HOWTO.md](PROJ-HOWTO.md). One line per guide.

- **Install auto-sudo and start using it** — get wrapped commands (`vim`, `chmod`, `rm`, ...) auto-escalating to `sudo` in your shell.
- **Check why a command did (or didn't) get sudo-prefixed** — understand which rule fired, without guessing, using `auto-sudo decide --explain`.
- **Add or change which commands escalate to sudo** — make a new command (or a new condition on an existing one) trigger `sudo` via `~/.config/auto-sudo/config.yaml`; includes escalating to a non-root user (`mode: user`) instead of root.
- **Set up passwordless sudo for the commands you've wrapped** ([howto/passwordless-sudo.md](howto/passwordless-sudo.md)) — generate, install, and manage checksum-pinned `sudoers.d` entries so escalation doesn't stop to ask for your password every time.
- **Use auto-sudo in bash instead of zsh** — generate bash-syntax wrapper functions with `auto-sudo shell --shell bash` and source them manually, since the install flow only wires up zsh.
