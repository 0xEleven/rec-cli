# rec-cli Bash Integration (v1.3.0 — process-managed via rec-pty)
# Add to ~/.bashrc:  eval "$(rec-cli init bash)"

_rec_load_config() {
  local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/config"
  # shellcheck disable=SC1090
  [[ -f "$cfg" ]] && source "$cfg"
  export TERM_REC_DIR="${REC_DIR:-${TERM_REC_DIR:-$HOME/Videos/terminal-recordings}}"
  export TERM_VIDEO_DIR="${REC_VIDEO_DIR:-${TERM_VIDEO_DIR:-$HOME/Videos/content}}"
}
_rec_load_config

_rec_is_allowed_terminal() {
  local allowed="${REC_TERMINALS:-kitty,tabby}"
  local allowed_lc="${allowed,,}"
  [[ ",${allowed_lc}," == *",all,"* ]] && return 0

  local cur=""
  if [[ -n "${KITTY_WINDOW_ID:-}" || -n "${KITTY_PID:-}" || "${TERM:-}" == "xterm-kitty" ]]; then
    cur="kitty"
  elif [[ "${TERM_PROGRAM:-}" == "Tabby" ]]; then
    cur="tabby"
  elif [[ -n "${ALACRITTY_WINDOW_ID:-}" || -n "${ALACRITTY_SOCKET:-}" || "${TERM:-}" == "alacritty" ]]; then
    cur="alacritty"
  elif [[ "${TERM_PROGRAM:-}" == "WezTerm" || -n "${WEZTERM_PANE:-}" ]]; then
    cur="wezterm"
  elif [[ "${TERM_PROGRAM:-}" == "ghostty" || -n "${GHOSTTY_RESOURCES_DIR:-}" ]]; then
    cur="ghostty"
  elif [[ "${TERM:-}" == "foot" || "${TERM:-}" == "foot-extra" ]]; then
    cur="foot"
  elif [[ -n "${KONSOLE_VERSION:-}" ]]; then
    cur="konsole"
  elif [[ -n "${GNOME_TERMINAL_SCREEN:-}" || -n "${VTE_VERSION:-}" ]]; then
    cur="gnome-terminal"
  fi

  [[ -n "$cur" && ",${allowed_lc}," == *",${cur},"* ]]
}

rec() {
  _rec_load_config
  local sub="${1:-help}"
  if [[ "$sub" == "start" || "$sub" == "on" ]]; then
    shift
    if [[ "${1:-}" =~ ^[0-9]+$ || -n "${REC_PTY_PID:-}" ]]; then
      rec-cli start "$@"
    else
      rec-pty ${1:+--name "$1"} -- "${SHELL:-bash}"
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

# Auto-wrap allowed terminals with rec-pty
if _rec_is_allowed_terminal \
  && [[ ! -f "${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/auto-disabled" ]] \
  && [[ ! -f "$HOME/.config/term-rec/auto-disabled" ]] \
  && [[ $- == *i* ]] \
  && [[ -t 0 && -t 1 ]] \
  && [[ "${TERM:-}" != "dumb" ]] \
  && [[ -z "${REC_PTY_PID:-}" && -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" && -z "${NO_RECORD:-}" && -z "${CI:-}" && -z "${VSCODE_INJECTION:-}" ]] \
  && [[ "$(ps -o comm= -p "$PPID" 2>/dev/null)" != "ttyd" ]] \
  && command -v rec-pty >/dev/null 2>&1; then
  mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
  command -v casts2mp4 >/dev/null 2>&1 && casts2mp4 --bg
  exec rec-pty -- "${SHELL:-bash}"
fi
