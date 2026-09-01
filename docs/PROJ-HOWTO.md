# auto-sudo — How To

Task-oriented guides for the things you'll actually do with auto-sudo. See
[PROJ-ARCH.md](PROJ-ARCH.md) for *why* it's built this way and
[PROJ-LAYOUT.md](PROJ-LAYOUT.md) for *where* things live.

## How to: install auto-sudo and start using it

**Goal:** get wrapped commands (`vim`, `chmod`, `rm`, ...) auto-escalating to `sudo` in your shell.
**Prereqs:** Rust/Cargo toolchain, zsh as your shell.

1. From this directory:
   ```bash
   make install
   ```
   This builds the release binary, installs it to `~/.local/bin/auto-sudo`,
   installs the zsh loader to `~/.local/share/auto-sudo/`, seeds
   `~/.config/auto-sudo/config.yaml` from `config.example.yaml` (only if it
   doesn't already exist), and appends a `source` line to `~/.zshrc`.
2. Load it into your current shell:
   ```bash
   source ~/.zshrc
   ```

**Verify:**
```bash
vim /etc/hosts   # should print a yellow "Auto Sudo vim /etc/hosts" notice, then prompt for your sudo password
vim /tmp/some-new-file.txt   # should open normally, no sudo
```

**Gotchas:**
- `make install` never overwrites an existing `~/.config/auto-sudo/config.yaml` — edit that file directly for rule changes, not `config.example.yaml`.
- Wrappers only take effect in shells that source `~/.zshrc` after install; open a new terminal if `source` alone doesn't pick it up.

## How to: check why a command did (or didn't) get sudo-prefixed

**Goal:** understand which rule fired, without guessing.
**Prereqs:** auto-sudo installed (see above).

1. Run the CLI's decision engine directly with `--explain`:
   ```bash
   auto-sudo decide --explain -- vim /etc/hosts
   ```
   Output on stderr names the matched rule (e.g. `matched rule protected-config-path for vim`), and stdout prints the prefix (`sudo `) that the wrapper would apply.
2. For a command with no match:
   ```bash
   auto-sudo decide --explain -- vim /tmp/scratch.txt
   ```
   prints `no config for command ...` or no rule match, and an empty prefix.

**Verify:** the printed rule name matches the entry you expect in `~/.config/auto-sudo/config.yaml`.

**Gotchas:**
- `decide` never executes the wrapped command — it only prints a prefix. Don't expect side effects from running it directly.
- If the command isn't wrapped at all (not present as a key under `commands:` in your config), `decide` reports "no config for command", even if the binary itself exists.

## How to: add or change which commands escalate to sudo

**Goal:** make a new command (or a new condition on an existing one) trigger `sudo`.
**Prereqs:** installed config at `~/.config/auto-sudo/config.yaml`.

1. Add a `commands:` entry. Minimal permission-gated example:
   ```yaml
   commands:
     tee:
       wrap: true
       allow_pipes: true
       rules:
         - name: tee-protected-path
           args:
             files:
               - position: any
                 skip_prefixes: ["-"]
           when:
             any_file:
               paths: ["/etc/*"]
   ```
   Or force it to always run under sudo:
   ```yaml
   commands:
     systemctl:
       wrap: true
       always_sudo: true
   ```
2. Reload the shell wrappers (new/changed `commands:` keys need regenerated functions):
   ```bash
   source ~/.zshrc
   ```

**Verify:**
```bash
auto-sudo decide --explain -- tee /etc/myfile
```

**Gotchas:**
- Rules are evaluated top-to-bottom per command; the **first** matching rule wins — order matters when a command has several rules.
- `allow_pipes` is `false` by default (both globally under `defaults:` and per-command). A wrapped command used in a pipeline (`... | tee /etc/foo`) won't escalate unless `allow_pipes: true` is set for it.
- See `config.example.yaml` for the full set of file-check predicates (`exists_not_writable`, `owner_is_not_current_user`, `path_prefixes`, `missing_parent_not_writable`, etc.).

### Escalating to a non-root user instead of root

Some rules need `sudo -u someuser`, not full root — e.g. editing files owned by
a service account. Set `action.sudo.mode: user` with `user`/`group` on the rule:

```yaml
commands:
  psql-vim:
    wrap: true
    rules:
      - name: edit-postgres-owned-files-as-postgres
        action:
          sudo:
            mode: user
            user: postgres
            group: postgres
        args:
          files:
            - position: any
              skip_prefixes: ["-", "+"]
        when:
          any_file:
            path_prefixes: ["/var/lib/postgresql/", "/usr/local/var/postgres/"]
```

This renders as `sudo -u postgres -g postgres psql-vim ...` instead of the
default bare `sudo psql-vim ...`. Omit `action.sudo` (or set `mode: root`,
the global default) for ordinary root escalation.

**Verify:**
```bash
auto-sudo decide --explain -- psql-vim /var/lib/postgresql/data/pg_hba.conf
# prefix should be: sudo -u postgres -g postgres
```

**Gotchas:**
- `mode: user` without a `user` field fails decision with `sudo mode user requires action.sudo.user` — always pair `mode: user` with `user:`.

## How to: set up passwordless sudo for the commands you've wrapped
Generate, install, and manage `sudoers.d` entries so escalation doesn't stop to ask for your password every time.
→ *See [howto/passwordless-sudo.md](howto/passwordless-sudo.md)*

## How to: use auto-sudo in bash instead of zsh

**Goal:** get the same auto-escalating wrapper functions in a bash shell.
**Prereqs:** `auto-sudo` binary installed (see the install guide above);
`~/.config/auto-sudo/config.yaml` already configured — `make install`/`auto-sudo.zsh`
only wire up zsh, so bash needs manual sourcing.

1. Generate bash-syntax wrapper functions instead of the zsh ones:
   ```bash
   auto-sudo shell --shell bash > ~/.local/share/auto-sudo/auto-sudo.bash
   ```
2. Source the generated file from `~/.bashrc`:
   ```bash
   echo 'source ~/.local/share/auto-sudo/auto-sudo.bash' >> ~/.bashrc
   source ~/.bashrc
   ```

**Verify:**
```bash
type vim   # should show a shell function, not the plain binary
vim /etc/hosts   # should print the yellow "Auto Sudo vim /etc/hosts" notice and prompt for sudo
```

**Gotchas:**
- There is no bash equivalent of `make install`'s auto-seeding/`~/.bashrc`-append — you regenerate and re-source `auto-sudo.bash` yourself after every `commands:` change in your config.
- `auto-sudo shell --shell zsh` is the default when `--shell` is omitted; always pass `--shell bash` explicitly here.
