# FAQ deep-dive: passwordless sudo entries

Full answers for the `sudoers print/write/refresh/toggle` question cluster.
Short answers + links live in [../PROJ-FAQ.md](../PROJ-FAQ.md); the procedure
itself is in [passwordless-sudo.md](passwordless-sudo.md).

### How does `auto-sudo decide` differ from `auto-sudo sudoers write`?

`decide` is the per-invocation runtime check the shell wrapper calls every
time — it only prints a prefix and touches nothing on disk. `sudoers write`
(and `refresh`) are one-time/occasional administrative actions that generate
and install real `/etc/sudoers.d` NOPASSWD entries so the *sudo prompt
itself* stops appearing for wrapped commands. You can use `decide`-driven
escalation with ordinary interactive sudo prompts and never touch
`sudoers write` at all.

### How does this differ from giving my user `NOPASSWD: ALL` in sudoers?

`NOPASSWD: ALL` removes the password prompt for *every* command as root,
unconditionally. auto-sudo's generated sudoers entries are checksum-pinned to
a specific resolved binary path (`checksum=... path=/usr/bin/vim`) — so the
entry only matches that exact file's contents, not "vim, whatever it
resolves to at the time," and only for commands you've explicitly configured
with `always_sudo` or `rules:`.

### Can I preview or install a passwordless sudo entry for a command that isn't in my `config.yaml` yet?

Yes — `auto-sudo sudoers print --command mytool` (and `write`/`refresh` with
the same flag) renders an entry for any resolvable binary, even one you
haven't added to `commands:` in your config. The catch: that entry isn't
wired to any `decide` rule, so `sudo -n mytool ...` runs passwordless but the
shell wrapper still won't auto-prefix `sudo` for `mytool` — you'd type `sudo
mytool` yourself, or add the command to `config.yaml` with `wrap: true` to
get auto-escalation and the passwordless entry together.

### Why toggle a sudoers entry off instead of just removing the command from my config and re-running `refresh`?

Because `toggle --off` is reversible in place: it comments the specific
`AUTO-SUDO ENTRY`-tagged line out in `/etc/sudoers.d/auto-sudo` by its `id=`,
leaving `config.yaml` and every other entry untouched, so `toggle --on`
restores exactly that entry later. Removing the command from config and
refreshing instead regenerates the whole managed block and drops the entry
outright — fine if you're done with it for good, worse if you just want to
suspend it during, say, a security review.

### Why does a generated sudoers entry also record the file's device/inode/mtime, not just a SHA-256 checksum?

It doesn't change what `sudo` enforces — only the `checksum=...` value in the
`NOPASSWD:` line is what `sudo` itself checks against the binary's bytes at
exec time. `dev=/ino=/mtime=` are recorded in the `# AUTO-SUDO ENTRY` comment
purely as metadata, for a human or future tooling to notice "this file's
identity changed" even in the rare case its content checksum didn't (e.g. a
symlink retarget). Nothing in auto-sudo currently reads those fields back to
enforce anything.

### Is running `auto-sudo sudoers write` safe to do without reviewing the output first?

Review with `sudoers print` first — `write` (without `--append`) **replaces**
the entire target file's contents, silently discarding anything you or
another tool hand-maintains there. It also processes *every* configured
command with `always_sudo` or `rules:`, not just ones you care about right
now, and fails outright if any of them doesn't resolve to a real binary on
this host (e.g. leaving macOS-only `dscacheutil` uncommented on Linux).

### What happens after I upgrade a wrapped binary (e.g. a new `vim` from your package manager)?

Nothing automatically — passwordless sudoers entries pin a SHA-256 checksum
of the binary at generation time, so an upgraded binary's checksum no longer
matches and `sudo` will prompt for a password again (fail closed, not fail
open). Run `auto-sudo sudoers refresh` to regenerate entries against the
current binaries.
→ *See [passwordless-sudo.md#4-refresh-after-binaries-change](passwordless-sudo.md).*
