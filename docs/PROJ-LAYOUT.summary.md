```
auto-sudo/
├── auto-sudo.zsh           # zsh loader
├── config.example.yaml     # default rules
├── Makefile                # compile/test/install
├── .meta/pointers.yaml     # doc-pointer store
├── rust/                   # Rust CLI crate
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── .gitignore
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── decision.rs
│       ├── shell.rs
│       └── sudoers.rs
├── docs/                   # PROJ-ARCH/SCHEMA/HOWTO/FAQ/LAYOUT/THREAT-MODEL (+ summaries); howto/, faq/ topic pages
├── .gitignore              # .DS_Store, swap files, .env*, .claude/worktrees/
├── CHANGELOG.md
├── CLAUDE.md
├── AGENT.md
├── AGENTS.md
├── merge-notes.md
└── README.md
```
