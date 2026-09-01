# merge-notes — auto-sudo (sep-1 branch sweep, 2026-09-01)

Utility repo (`the-robot-lives/auto-sudo`; dual checkout `Portfolio/Utilities/source/auto-sudo` + `utilities/...`, same remote — swept from the source checkout). 

- Base: `origin/mono-repo-dev` @ b202327 (freshest tip by commit date; mono-repo-dev carries the real unmerged content vs a stale `main`).
- `develop` cut from mono-repo-dev b202327 and pushed; **mono-repo-dev retained as historical** — develop IS its continuation; do not open a PR for mono-repo-dev.
- `sep-1` tag created on b202327 (base tip).
- Local `main` left as-is / ff-synced to `origin/main` (was stale); main never pushed. Fast-forward `develop` → `main` when adopting.
- Branches pruned: none (repo held only `main` + `mono-repo-dev`; mono-repo-dev kept per sweep policy).

## Review/merge sequence
1. Review `develop` (== mono-repo-dev b202327) vs `main`; merge `develop` → `main` (FF) when validated. No other pending work; no open PRs.

## Open PRs
None.
