# Threat Model Summary — auto-sudo

- **Scope**: single-user local privilege-escalation helper; no network surface. Trust boundary: user process ↔ root. Jewels: `/etc/sudoers.d/auto-sudo` (grants) and `~/.config/auto-sudo/config.yaml` (policy).
- **Register**: 6 entries — 2 mitigated, 2 partial, 1 open, 3 accepted residuals.
  - T-001 High (Elevation, accepted): NOPASSWD entries pin binary+checksum but not args — granted editors etc. are passwordless-root-shell vectors (e.g. vim `:!sh`). Inherent to design on a single-admin workstation.
  - T-002 Medium (Tampering, partial): config.yaml is user-writable; same-user malware can reshape policy before a user-run `sudoers write`. Control: review `sudoers print` first; managed entries auditable/togglable.
  - T-003 Medium (Elevation, OPEN): `sudoers_subject()` falls back to `ALL` when `$USER` unset — writes from cron/launchd-style envs grant every local user NOPASSWD. Hardening: error or `--subject` instead of widening.
  - T-004 Low (Tampering, accepted): TOCTOU between decide checks and sudo exec.
  - T-005 Low (Spoofing, mitigated): PATH shadowing of decide/targets; sudoers absolute-path + sha256 digest pins the escalated binary.
  - T-006 Low (Info disclosure, accepted): wrapper notice prints full argv to stderr.
- **Key mitigations**: `visudo -cf` + atomic rename on all writes (`write_checked`); sha256 digest constraints on every NOPASSWD line; `shell_word()` quoting blocks injection via config into the wrapper's `eval`; missing-file ≠ unreadable semantics; fail-closed on config errors.
- Details: [THREAT-MODEL.md](THREAT-MODEL.md).
