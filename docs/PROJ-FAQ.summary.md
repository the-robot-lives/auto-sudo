# auto-sudo — FAQ Summary

Question index only. Full answers in [PROJ-FAQ.md](PROJ-FAQ.md).

## Motivation
- Why would I use this instead of just typing `sudo` myself?
- Why is behavior defined in a YAML config instead of hardcoded per command?
- Why does `auto-sudo decide` only print a prefix instead of just running the command with the right privileges?

## Fit
- When is auto-sudo the wrong tool?
- Should I use `always_sudo: true` or permission-gated `rules:` for a command I'm adding?
- Is auto-sudo a substitute for `sudo -A`/`sudo-askpass`/security-hardening tools?

## Comparison
- How is this different from just adding shell aliases like `alias vim='sudo vim'`?
- How does `auto-sudo decide` differ from `auto-sudo sudoers write`?
- How does this differ from giving my user `NOPASSWD: ALL` in sudoers?

## Capability
- Can auto-sudo run a wrapped command as a non-root user, or only root?
- Does auto-sudo work with commands used in a pipeline, like `... | tee /etc/foo`?
- Does auto-sudo work in bash, not just zsh?
- Can I preview or install a passwordless sudo entry for a command that isn't in my `config.yaml` yet?
- Can a rule tell the difference between editing a file that doesn't exist yet vs. one I can't read?

## Caveats
- Why toggle a sudoers entry off instead of just removing the command from my config and re-running `refresh`?
- Why does a generated sudoers entry also record the file's device/inode/mtime, not just a SHA-256 checksum?
- Is running `auto-sudo sudoers write` safe to do without reviewing the output first?
- What happens after I upgrade a wrapped binary (e.g. a new `vim` from your package manager)?
- Could a bad rule in my config accidentally escalate something I didn't intend?
- If the config file is malformed, does auto-sudo fall back to no escalation, or does it refuse to run?

## Trust
- Does `make install` ever overwrite my existing config or sudoers file?
- Does auto-sudo log or transmit anything about the commands I run?
