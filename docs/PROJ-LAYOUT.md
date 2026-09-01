# Project Layout

```
auto-sudo/
├── auto-sudo.zsh           # zsh loader — evals `auto-sudo shell --shell zsh`
├── config.example.yaml     # default YAML rules (vim/chmod/chown/chgrp)
├── Makefile                # compile/test/install targets (→ ~/.local/bin, ~/.config/auto-sudo)
├── rust/                   # Rust CLI crate (auto-sudo binary)
│   ├── Cargo.toml          #   crate manifest (clap, serde_yaml, sha2, which)
│   ├── Cargo.lock          #   pinned dependency versions
│   ├── .gitignore          #   ignores /target/
│   └── src/
│       ├── main.rs         #   CLI entrypoint (decide / shell / sudoers subcommands)
│       ├── config.rs       #   config schema and loading
│       ├── decision.rs     #   sudo decision engine (rules, file checks)
│       ├── shell.rs        #   shell wrapper generator (zsh/bash)
│       └── sudoers.rs      #   sudoers snippet manager (print/write/toggle/refresh)
├── docs/                   # Documentation
│   ├── PROJ-ARCH.md        #   architecture doc (+ .summary.md)
│   └── PROJ-LAYOUT.md      #   this file (+ .summary.md)
├── .gitignore              # editor swap files, .env, .envrc.local
└── README.md               # Project description, install, and usage
```

## Key Files

| File | Purpose |
|------|---------|
| `rust/src/main.rs` | CLI entrypoint |
| `rust/src/config.rs` | Config schema and loading |
| `rust/src/decision.rs` | sudo decision engine |
| `rust/src/shell.rs` | shell wrapper generator |
| `rust/src/sudoers.rs` | sudoers snippet manager |
| `auto-sudo.zsh` | Source in `.zshrc` to enable generated wrappers |
| `config.example.yaml` | Starter rules for vim/chmod/chown/chgrp |
| `Makefile` | `make install` = build + install binary, loader, default config |
| `README.md` | Describes the auto-sudo concept |

## Key Files Requiring Setup

| File | Action |
|------|--------|
| `~/.config/auto-sudo/config.yaml` | Seeded from `config.example.yaml` by `make install`; edit rules here |
| `~/.zshrc` | `make install` appends a source line for the installed loader |

## Usage

Run `make install`, edit `~/.config/auto-sudo/config.yaml`, then source the
installed loader from `.zshrc`. Build artifacts land in `rust/target/`
(gitignored, not documented here).
