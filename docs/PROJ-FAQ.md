# auto-sudo — FAQ

Anticipated why/when/compared-to-what questions. For *how*, see
[PROJ-HOWTO.md](PROJ-HOWTO.md); for *why built this way*, see
[PROJ-ARCH.md](PROJ-ARCH.md).

## Motivation

### Why would I use this instead of just typing `sudo` myself?

Because the friction of remembering `sudo` (or forgetting it and retyping the
whole command) is exactly what auto-sudo removes, at the cost of trusting a
rules file to make that call correctly. You type `vim /etc/hosts` normally;
the wrapper inspects the arguments and decides whether root is actually
needed before you hit the permission-denied error. The trade-off: you're now
depending on the rule set matching reality — a badly written rule can either
nag you with unnecessary sudo prompts or, worse, silently *not* escalate when
it should have.
→ *See [PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it](PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it).*

### Why is behavior defined in a YAML config instead of hardcoded per command?

So that adding or tuning a command doesn't require touching Rust code or the
zsh wrapper — see [PROJ-ARCH.md](PROJ-ARCH.md#key-design-decisions) for the
"config-driven behavior" rationale. The honest cost: YAML rule matching is
less expressive than arbitrary code, so exotic argument-parsing needs (a
command with unusual flag syntax) may not be representable without adding a
new selector to `decision.rs` itself.

### Why does `auto-sudo decide` only print a prefix instead of just running the command with the right privileges?

Because keeping execution in the shell wrapper — not the Rust binary — means
`decide` never needs privilege itself and can't become a privilege-escalation
vector on its own. See "CLI decides, shell executes" in
[PROJ-ARCH.md](PROJ-ARCH.md#key-design-decisions). The cost is an extra
process spawn per wrapped invocation; in practice this is not perceptible
next to interactive command latency.

## Fit

### When is auto-sudo the wrong tool?

When your workflow is non-interactive or not zsh-based. auto-sudo's install
path wires wrapper functions into zsh via `auto-sudo.zsh`; scripts, cron
jobs, and CI steps that call `vim`/`chmod`/etc. directly (not through an
interactive zsh session) never invoke the wrapper and get no auto-escalation.
For those, call `sudo` explicitly or invoke `auto-sudo decide` yourself in
the script.
→ *See [PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it](PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it).*

### Should I use `always_sudo: true` or permission-gated `rules:` for a command I'm adding?

Use `always_sudo` only for commands that are administrative by nature
regardless of target (`systemctl`, `launchctl`) — anything where "sometimes
you don't need root" isn't a real case. Use `rules:` with file-permission
predicates for commands that operate on user-supplied paths (editors, `cp`,
`rm`, `chmod`) where root is only needed some of the time. Getting this wrong
in the `always_sudo` direction means sudo prompts for operations that never
needed root.
→ *See [PROJ-HOWTO.md#how-to-add-or-change-which-commands-escalate-to-sudo](PROJ-HOWTO.md#how-to-add-or-change-which-commands-escalate-to-sudo).*

### Is auto-sudo a substitute for `sudo -A`/`sudo-askpass`/security-hardening tools?

No — it's a convenience layer over ordinary `sudo`, not a replacement for
`sudo`'s own auth/policy model. It decides *whether* to prepend `sudo`; it
does not change how `sudo` authenticates, and passwordless entries it
generates are still standard `sudoers.d` NOPASSWD lines that `sudo` itself
enforces.

## Comparison

### How is this different from just adding shell aliases like `alias vim='sudo vim'`?

A blanket alias escalates every invocation, including ones on files you can
already write — you'd sudo to edit your own dotfiles. auto-sudo's rules
inspect the actual arguments (path, existing permissions, ownership) per
invocation, so `vim ~/notes.txt` runs unprivileged while `vim /etc/hosts`
escalates, from the same wrapper.

### How does `auto-sudo decide` differ from `auto-sudo sudoers write`?

`decide` is the per-invocation runtime check the shell wrapper calls every
time and only prints a prefix; `sudoers write`/`refresh` are occasional admin
actions that install real `/etc/sudoers.d` entries so the sudo prompt itself
stops appearing. You can use one without the other.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

### How does this differ from giving my user `NOPASSWD: ALL` in sudoers?

`NOPASSWD: ALL` removes the prompt for every command unconditionally; auto-sudo's
entries are checksum-pinned to one resolved binary path and only cover
commands you've explicitly configured with `always_sudo` or `rules:`.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

## Capability

### Can auto-sudo run a wrapped command as a non-root user, or only root?

Yes — a command's rule (or command-level `sudo:`) can set `mode: user` with
a specific `user`/`group` instead of the root default; see the commented
`psql-vim` example in `config.example.yaml`. Root (`mode: root`, the global
default) is what most commands use in practice.

### Does auto-sudo work with commands used in a pipeline, like `... | tee /etc/foo`?

Only if `allow_pipes: true` is set for that command — globally it's `false`
by default. `tee` in `config.example.yaml` ships with `allow_pipes: true`
already; most other wrapped commands (editors, `cp`, `mv`, `rm`) don't,
because piping into destructive/privileged operations is exactly the case
the default is guarding against.
→ *See [PROJ-HOWTO.md#how-to-add-or-change-which-commands-escalate-to-sudo](PROJ-HOWTO.md#how-to-add-or-change-which-commands-escalate-to-sudo).*

### Does auto-sudo work in bash, not just zsh?

The Rust CLI's `shell` subcommand can generate bash wrappers (`auto-sudo
shell --shell bash`), but the shipped install flow (`make install`,
`auto-sudo.zsh`, the `~/.zshrc` append) only wires up zsh end to end. Bash
support exists at the wrapper-generation layer; you'd need to source the
generated bash output yourself.

### Can I preview or install a passwordless sudo entry for a command that isn't in my `config.yaml` yet?

Yes — `auto-sudo sudoers print --command mytool` works for any resolvable
binary, config or not. The catch: it isn't wired to a `decide` rule, so the
shell wrapper still won't auto-prefix `sudo` for it.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

### Can a rule tell the difference between editing a file that doesn't exist yet vs. one I can't read?

Yes, and this is a case auto-sudo got wrong once and fixed: a missing path is
treated as "absent," matched by `missing_parent_not_writable`/
`missing_parent_not_readable`, not lumped in with `current_user_cannot_read`
or `exists_not_writable`. See the `m3-missing-file-semantics` entry in
[CHANGELOG.md](../CHANGELOG.md) for the false-positive this replaced.

## Caveats

### Why toggle a sudoers entry off instead of just removing the command from my config and re-running `refresh`?

`toggle --off` comments out just that entry's line by `id=`, leaving config
and every other entry untouched, so `toggle --on` restores it exactly.
Removing-and-refreshing instead regenerates the whole managed block.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

### Why does a generated sudoers entry also record the file's device/inode/mtime, not just a SHA-256 checksum?

Only the `checksum=...` value is what `sudo` itself enforces at exec time;
`dev=/ino=/mtime=` are recorded in the comment as metadata for humans/tooling
to notice the file's identity changed, not enforced by anything today.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

### Is running `auto-sudo sudoers write` safe to do without reviewing the output first?

Review with `sudoers print` first — `write` (without `--append`) **replaces**
the entire target file and processes every configured command with
`always_sudo`/`rules:`, failing outright if any doesn't resolve to a binary
on this host.
→ *Full discussion: [faq/passwordless-sudo.md](faq/passwordless-sudo.md).*

### What happens after I upgrade a wrapped binary (e.g. a new `vim` from your package manager)?

Nothing automatically — the pinned checksum no longer matches, so `sudo`
prompts for a password again (fail closed). Run `auto-sudo sudoers refresh`
to regenerate entries against current binaries.
→ *See [howto/passwordless-sudo.md#4-refresh-after-binaries-change](howto/passwordless-sudo.md).*

### Could a bad rule in my config accidentally escalate something I didn't intend?

Yes — this is the primary security surface, since `~/.config/auto-sudo/config.yaml`
is what decides root access for wrapped commands. Rules are matched top to
bottom and the first match wins per [PROJ-ARCH.md](PROJ-ARCH.md#key-design-decisions),
so a too-broad `paths:` wildcard or an `always_sudo: true` added carelessly
takes effect immediately on next `source ~/.zshrc`. There's no dry-run mode
beyond `auto-sudo decide --explain`, so test new rules with `--explain`
before trusting them interactively.

### If the config file is malformed, does auto-sudo fall back to no escalation, or does it refuse to run?

It fails closed: wrapper generation and `decide` calls return non-zero on a
config parse error rather than silently proceeding without sudo (see
[PROJ-ARCH.md](PROJ-ARCH.md#key-design-decisions)) — so a broken config
blocks the wrapped command entirely rather than quietly under-escalating.

## Trust

### Does `make install` ever overwrite my existing config or sudoers file?

`make install` never overwrites an existing `~/.config/auto-sudo/config.yaml`
— it seeds one from `config.example.yaml` only if none exists yet. It does
not touch `/etc/sudoers.d/` at all; that only happens when you explicitly run
`auto-sudo sudoers write` or `refresh`, and `write` itself replaces (not
merges) its target file unless you pass `--append`.
→ *See [PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it](PROJ-HOWTO.md#how-to-install-auto-sudo-and-start-using-it).*

### Does auto-sudo log or transmit anything about the commands I run?

No — `decide` and the generated wrappers are local-only: they read your
config, print a prefix or a stderr notice, and invoke the command in your
own shell. Nothing in this project reads `.infra-config.yaml`, calls out to
the network, or shares state with the rest of the Noizu Infra monorepo (see
"Ecosystem Fit" in [PROJ-ARCH.md](PROJ-ARCH.md#ecosystem-fit)).
