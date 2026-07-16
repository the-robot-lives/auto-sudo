# How to: set up passwordless sudo for the commands you've wrapped

**Goal:** wrapped commands (`vim /etc/hosts`, etc.) run through `sudo` without an
interactive password prompt every time, by installing a checksum-pinned
`sudoers.d` snippet.
**Prereqs:** auto-sudo installed and configured (see main
[PROJ-HOWTO.md](../PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it));
`sudo` access to write under `/etc/sudoers.d/` and run `visudo`.

## 1. Preview what would be generated

```bash
auto-sudo sudoers print
```

This renders one entry per configured command that either has `always_sudo: true`
or has at least one `rules:` entry — **every one of those commands must resolve
to a real executable on `$PATH`**, because each entry embeds the binary's
resolved path and a SHA-256 checksum:

```
# AUTO-SUDO ENTRY id=vim-root command=vim path=/usr/bin/vim checksum=... dev=... ino=... mtime=...
keith ALL=(root) NOPASSWD: <checksum> /usr/bin/vim
```

Add an extra command not already in your config (without editing YAML):

```bash
auto-sudo sudoers print --command mytool
```

## 2. Write it to a sudoers.d file

```bash
sudo -v   # ensure you have a sudo timestamp active; auto-sudo itself does not prompt
auto-sudo sudoers write --file /etc/sudoers.d/auto-sudo
```

`write` replaces the file's contents. Use `--append` to add to an existing file
instead of replacing it:

```bash
auto-sudo sudoers write --append --file /etc/sudoers.d/auto-sudo
```

## 3. Validate before trusting it

```bash
auto-sudo sudoers check --file /etc/sudoers.d/auto-sudo
```

This shells out to `visudo -cf` against the file. Fix any reported syntax error
before relying on the entries.

## 4. Refresh after binaries change

Checksums are pinned at generation time. After an editor/tool upgrade (new
binary, new checksum), regenerate:

```bash
auto-sudo sudoers refresh --file /etc/sudoers.d/auto-sudo
```

`refresh` takes the same arguments as `write` and fully regenerates the managed
entries with current paths/checksums.

## 5. Temporarily disable one entry without deleting it

```bash
auto-sudo sudoers toggle vim-root --off   # comments the entry out
auto-sudo sudoers toggle vim-root --on    # re-enables it
```

The `vim-root`-style entry id is the `id=` value shown in `sudoers print`
output (or in the managed file's `# AUTO-SUDO ENTRY` comment lines).

**Verify:**
```bash
sudo -n /usr/bin/vim /etc/hosts   # -n = no password prompt; should succeed if the entry is active
```

**Gotchas:**
- `sudoers print`/`write`/`refresh` process **every** command in your config
  that has `always_sudo: true` or any `rules:`, not just the one(s) you
  mention with `--command`. If any of those commands isn't installed (e.g.
  `dscacheutil` in `config.example.yaml` is macOS-only and won't exist on
  Linux), the whole render fails with `failed to locate <command>: cannot find
  binary path`. Fix: comment out or remove commands from your
  `~/.config/auto-sudo/config.yaml` that aren't installed on this host — the
  example file marks the macOS/Linux-specific ones (`dscacheutil`,
  `launchctl`, `systemctl`, `apt`, etc.) clearly for this reason.
- `write` (without `--append`) **replaces** the whole target file — if you hand-maintain
  other entries in `/etc/sudoers.d/auto-sudo`, use `--append` or a separate file.
- These subcommands themselves never call `sudo` — you need write access to
  `/etc/sudoers.d/` (typically via your own interactive `sudo`) to run `write`/`refresh`/`toggle`.
