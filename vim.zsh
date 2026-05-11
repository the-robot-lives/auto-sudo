# --- auto-sudo vim shim ---
# Automatically uses sudo when editing files you don't have write access to.
vim() {
  local need_sudo=false
  local files=()

  # Collect file arguments (skip flags: -x, +x)
  for arg in "$@"; do
    [[ "$arg" == -* || "$arg" == +* ]] && continue
    files+=("$arg")
  done

  # No file args → just open vim normally
  if (( ${#files[@]} == 0 )); then
    command vim "$@"
    return
  fi

  for f in "${files[@]}"; do
    if [[ -e "$f" ]]; then
      # File exists: can I write it?
      [[ ! -w "$f" ]] && need_sudo=true && break
    else
      # File doesn't exist: can I create in parent dir?
      local dir="${f:h}"  # zsh dirname
      [[ ! -w "$dir" ]] && need_sudo=true && break
    fi
  done

  if $need_sudo; then
    printf '\033[1;33m⚡ auto-sudo vim\033[0m %s\n' "${files[*]}"
    sudo vim "$@"
  else
    command vim "$@"
  fi
}
