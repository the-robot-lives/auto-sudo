# Changelog — utilities/shell/auto-sudo

## [Unreleased]
- Refreshed PROJ-ARCH.md / PROJ-LAYOUT.md (+ summaries) to reflect current module layout

## [m3-missing-file-semantics] — 2026-07-07 — tag: `utilities-shell-auto-sudo/m3-missing-file-semantics`
Milestone summary: fixed a false-positive escalation where operating on a not-yet-created file (e.g. `vim /tmp/new-note.txt`) was misread as a permission-denied case and triggered `sudo`.

### Fixed
- `current_user_can/cannot_*` checks no longer match missing files; a nonexistent path is treated as "absent," not "unreadable," deferring to the `missing_parent_not_readable`/`missing_parent_not_writable` checks instead
### Added
- `missing_parent_not_readable` check alongside the existing writable-parent check

## [m2-decision-engine-hardening] — 2026-06-27 — tag: `utilities-shell-auto-sudo/m2-decision-engine-hardening`
Milestone summary: follow-up pass on the initial Rust rewrite — tightened the decision/sudoers logic and brought docs and example config in line with the new engine.

### Changed
- Expanded `decision.rs` rule matching and `sudoers.rs` handling
- Extended `config.rs` parsing to cover the new rule fields
- Updated README, PROJ-ARCH.md, PROJ-ARCH.summary.md, and config.example.yaml to match

## [m1-initial-rust-cli] — 2026-06-26 — tag: `utilities-shell-auto-sudo/m1-initial-rust-cli`
Milestone summary: replaced the original zsh-only prototype with a Rust CLI (`auto-sudo decide`) — config-driven rules decide whether a wrapped command should be prefixed with `sudo`, backed by a real Cargo crate (config, decision, shell, sudoers modules).

### Added
- Rust crate under `rust/` (`Cargo.toml`, `config.rs`, `decision.rs`, `shell.rs`, `sudoers.rs`)
- `config.example.yaml` documenting rule schema
- PROJ-ARCH.md / PROJ-LAYOUT.md (+ summaries) architecture docs
### Changed
- `auto-sudo.zsh` wrapper reworked to call into the new Rust decision engine
- Makefile and README updated for the Rust build/install flow
