# Project Layout

```
auto-sudo/
├── auto-sudo.zsh           # zsh loader for generated wrappers
├── config.example.yaml     # default YAML rules
├── rust/                   # Rust CLI crate
│   ├── Cargo.toml
│   └── src/
├── docs/                   # Documentation
└── README.md               # Project description and usage
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
| `README.md` | Describes the auto-sudo concept |

## Usage

Run `make install`, edit `~/.config/auto-sudo/config.yaml`, then source the
installed loader from `.zshrc`.
