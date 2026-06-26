# auto-sudo shell integration.
#
# This file intentionally contains only a loader. Command wrappers are generated
# from ~/.config/auto-sudo/config.yaml by the Rust CLI.

if command -v auto-sudo >/dev/null 2>&1; then
  eval "$(auto-sudo shell --shell zsh 2>/dev/null)"
fi
