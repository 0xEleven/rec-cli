# rec-cli Bash Integration (v2.0.0 — process-managed via rec-pty)
# Add to ~/.bashrc:  eval "$(rec-cli init bash)"

_rec_load_config() {
  local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/config"
  # shellcheck disable=SC1090
  [[ -f "$cfg" ]] && source "$cfg"
  export TERM_REC_DIR="${REC_DIR:-${TERM_REC_DIR:-$HOME/Videos/terminal-recordings}}"
  export TERM_VIDEO_DIR="${REC_VIDEO_DIR:-${TERM_VIDEO_DIR:-$HOME/Videos/content}}"
}
_rec_load_config

# Clear inherited rec-pty env vars if this shell is on a different PTY or the parent rec-pty exited
_rec_sanitize_env() {
  if [[ -n "${REC_PTY_PID:-}" ]]; then
    if ! kill -0 "$REC_PTY_PID" 2>/dev/null; then
      unset REC_PTY_PID REC_PTY_TTY REC_RUNTIME_DIR _TERM_REC_ACTIVE _TERM_REC_META _TERM_REC_MARKER
    elif [[ -n "${REC_PTY_TTY:-}" && -t 0 ]]; then
      local cur_tty
      cur_tty="$(tty 2>/dev/null || true)"
      if [[ -n "$cur_tty" && "$cur_tty" != "not a tty" && "$cur_tty" != "$REC_PTY_TTY" ]]; then
        unset REC_PTY_PID REC_PTY_TTY REC_RUNTIME_DIR _TERM_REC_ACTIVE _TERM_REC_META _TERM_REC_MARKER
      fi
    fi
  fi
}
_rec_sanitize_env

_rec_detect_terminal() {
  if [[ -n "${KITTY_WINDOW_ID:-}" || -n "${KITTY_PID:-}" || "${TERM:-}" == "xterm-kitty" ]]; then
    echo "kitty"
  elif [[ "${TERM_PROGRAM:-}" == "Tabby" || -n "${TABBY_CONFIG_DIRECTORY:-}" ]]; then
    echo "tabby"
  elif [[ -n "${ALACRITTY_WINDOW_ID:-}" || -n "${ALACRITTY_SOCKET:-}" || "${TERM:-}" == "alacritty" ]]; then
    echo "alacritty"
  elif [[ "${TERM_PROGRAM:-}" == "WezTerm" || -n "${WEZTERM_PANE:-}" ]]; then
    echo "wezterm"
  elif [[ "${TERM_PROGRAM:-}" == "ghostty" || -n "${GHOSTTY_RESOURCES_DIR:-}" ]]; then
    echo "ghostty"
  elif [[ "${TERM:-}" == "foot" || "${TERM:-}" == "foot-extra" ]]; then
    echo "foot"
  elif [[ -n "${KONSOLE_VERSION:-}" ]]; then
    echo "konsole"
  elif [[ -n "${GNOME_TERMINAL_SCREEN:-}" || -n "${VTE_VERSION:-}" ]]; then
    echo "gnome-terminal"
  elif [[ "$(ps -o comm= -p "$PPID" 2>/dev/null)" == "tabby" ]]; then
    echo "tabby"
  elif [[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" && "${TERM:-}" == xterm* ]]; then
    echo "xterm"
  fi
}

_rec_is_allowed_terminal() {
  local allowed="${REC_TERMINALS:-kitty,tabby}"
  local allowed_lc="${allowed,,}"
  [[ ",${allowed_lc}," == *",all,"* ]] && return 0
  local cur
  cur="$(_rec_detect_terminal)"
  [[ -n "$cur" && ",${allowed_lc}," == *",${cur},"* ]]
}

_rec_is_auto_enabled() {
  [[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/auto-disabled" ]] && return 1
  [[ -f "$HOME/.config/term-rec/auto-disabled" ]] && return 1
  local auto_lc="${REC_AUTO_RECORD:-off}"
  auto_lc="${auto_lc,,}"
  case "$auto_lc" in
    on|enable|enabled|1|true|yes) return 0 ;;
    *) return 1 ;;
  esac
}

rec() {
  _rec_load_config
  _rec_sanitize_env
  local sub="${1:-help}"
  if [[ "$sub" == "start" || "$sub" == "on" ]]; then
    shift
    local first_pos="" arg
    for arg in "$@"; do
      [[ "$arg" == "--new" || "$arg" == "--restart" || "$arg" == "-n" ]] && continue
      first_pos="$arg"
      break
    done
    if [[ "$first_pos" =~ ^[0-9]+$ || -n "${REC_PTY_PID:-}" ]]; then
      rec-cli start "$@"
    else
      rec-pty ${first_pos:+--name "$first_pos"} -- "${SHELL:-bash}"
    fi
    return $?
  fi
  rec-cli "$@"
}

alias recs='rec-cli list'
alias norec='NO_RECORD=1 bash'
alias render-casts='casts2mp4'

# Log executed commands to the live recording's .meta symlink via Bash DEBUG trap
if [[ -n "${_TERM_REC_META:-}" ]]; then
  _rec_bash_in_prompt=0
  _rec_bash_preexec() {
    [[ "$_rec_bash_in_prompt" -eq 1 ]] && return 0
    local cmd="${BASH_COMMAND:-}"
    case "$cmd" in
      ""|_rec_*|rec\ *|rec|rec-cli\ *|rec-cli|PROMPT_COMMAND*) return 0 ;;
    esac
    if [[ -L "$_TERM_REC_META" || -f "$_TERM_REC_META" ]]; then
      local ts
      ts="$(date +%s.%3N 2>/dev/null || date +%s)"
      printf '%s\t%s\n' "$ts" "$cmd" >> "$_TERM_REC_META" 2>/dev/null || true
    fi
    _rec_bash_in_prompt=1
  }
  _rec_bash_precmd() {
    _rec_bash_in_prompt=0
  }
  PROMPT_COMMAND="_rec_bash_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
  trap '_rec_bash_preexec' DEBUG
fi

# Wrap interactive desktop terminals in rec-pty (manual standby by default; auto-record only if enabled via `rec auto on`)
if [[ -n "$(_rec_detect_terminal)" ]] \
  && [[ $- == *i* ]] \
  && [[ -t 0 && -t 1 ]] \
  && [[ "${TERM:-}" != "dumb" ]] \
  && [[ -z "${REC_PTY_PID:-}" && -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" && -z "${NO_RECORD:-}" && -z "${CI:-}" && -z "${VSCODE_INJECTION:-}" ]] \
  && [[ "$(ps -o comm= -p "$PPID" 2>/dev/null)" != "ttyd" ]] \
  && command -v rec-pty >/dev/null 2>&1; then
  mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
  if _rec_is_allowed_terminal && _rec_is_auto_enabled; then
    exec rec-pty --auto -- "${SHELL:-bash}"
  else
    exec rec-pty --standby -- "${SHELL:-bash}"
  fi
fi
