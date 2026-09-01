# Project Layout

```
auto-sudo/
├── auto-sudo.zsh           # zsh loader — evals `auto-sudo shell --shell zsh`
├── config.example.yaml     # default YAML rules (editors, chmod/chown/chgrp, tee, cp/mv, readers, launchctl)
├── Makefile                # compile/test/install targets (→ ~/.local/bin, ~/.local/share/auto-sudo, ~/.config/auto-sudo)
├── rust/                   # Rust CLI crate (auto-sudo binary)
│   ├── Cargo.toml          #   crate manifest (clap, serde_yaml, sha2, base64, which, dirs)
│   ├── Cargo.lock          #   pinned dependency versions
│   ├── .gitignore          #   ignores /target/
│   └── src/
│       ├── main.rs         #   CLI entrypoint (decide / shell / sudoers subcommands)
│       ├── config.rs       #   config schema (serde) and loading
│       ├── decision.rs     #   sudo decision engine (arg extraction, file checks, prefix render)
│       ├── shell.rs        #   shell wrapper generator (zsh/bash)
│       └── sudoers.rs      #   sudoers snippet manager (print/write/toggle/refresh/check)
├── docs/                   # Documentation
│   ├── PROJ-ARCH.md        #   architecture doc (+ .summary.md)
│   ├── PROJ-LAYOUT.md      #   this file (+ .summary.md)
│   ├── PROJ-HOWTO.md       #   usage howto (+ .summary.md)
│   ├── PROJ-FAQ.md         #   FAQ (+ .summary.md)
│   ├── howto/              #   per-topic howtos (passwordless-sudo.md)
│   └── faq/                #   per-topic FAQ pages (passwordless-sudo.md)
├── .gitignore              # editor swap files, .env, .envrc.local
├── CHANGELOG.md            # release history
├── merge-notes.md          # notes from config refactor merge (data-driven rules)
└── README.md               # Project description, install, and usage
```

## Key Files

| File | Purpose |
|------|---------|
| `rust/src/main.rs` | CLI entrypoint (clap subcommands: `decide`, `shell`, `sudoers`) |
| `rust/src/config.rs` | Config schema and loading |
| `rust/src/decision.rs` | sudo decision engine |
| `rust/src/shell.rs` | shell wrapper generator |
| `rust/src/sudoers.rs` | sudoers snippet manager |
| `auto-sudo.zsh` | Source in `.zshrc` to enable generated wrappers |
| `config.example.yaml` | Starter ruleset (data-driven rules with YAML anchors) |
| `Makefile` | `make install` = build + install binary, loader, default config, zshrc line |
| `docs/PROJ-ARCH.md` | Architecture (decision engine, generators) |
| `docs/PROJ-HOWTO.md` | How to configure and operate |
| `docs/PROJ-FAQ.md` | FAQ; `docs/faq/`, `docs/howto/` for per-topic pages |
| `README.md` | Describes the auto-sudo concept and commands |

## Key Files Requiring Setup

| File | Action |
|------|--------|
| `~/.config/auto-sudo/config.yaml` | Seeded from `config.example.yaml` by `make install`; edit rules here |
| `~/.zshrc` | `make install` appends a source line for the installed loader |

## Usage

Run `make install`, edit `~/.config/auto-sudo/config.yaml`, then source the
installed loader from `.zshrc`. Build artifacts land in `rust/target/`
(gitignored, not documented here).
