# rec-cli Zsh Integration (v1.3.0 — process-managed via rec-pty)
# Add to ~/.zshrc:  eval "$(rec-cli init zsh)"

_rec_load_config() {
  local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/config"
  [[ -f "$cfg" ]] && source "$cfg"
  export TERM_REC_DIR="${REC_DIR:-${TERM_REC_DIR:-$HOME/Videos/terminal-recordings}}"
  export TERM_VIDEO_DIR="${REC_VIDEO_DIR:-${TERM_VIDEO_DIR:-$HOME/Videos/content}}"
}
_rec_load_config

_rec_detect_terminal() {
  if [[ -n "$KITTY_WINDOW_ID" || -n "$KITTY_PID" || "$TERM" == "xterm-kitty" ]]; then
    echo "kitty"
  elif [[ "$TERM_PROGRAM" == "Tabby" ]]; then
    echo "tabby"
  elif [[ -n "$ALACRITTY_WINDOW_ID" || -n "$ALACRITTY_SOCKET" || "$TERM" == "alacritty" ]]; then
    echo "alacritty"
  elif [[ "$TERM_PROGRAM" == "WezTerm" || -n "$WEZTERM_PANE" ]]; then
    echo "wezterm"
  elif [[ "$TERM_PROGRAM" == "ghostty" || -n "$GHOSTTY_RESOURCES_DIR" ]]; then
    echo "ghostty"
  elif [[ "$TERM" == "foot" || "$TERM" == "foot-extra" ]]; then
    echo "foot"
  elif [[ -n "$KONSOLE_VERSION" ]]; then
    echo "konsole"
  elif [[ -n "$GNOME_TERMINAL_SCREEN" || "$VTE_VERSION" != "" ]]; then
    echo "gnome-terminal"
  elif [[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" && "$TERM" == xterm* ]]; then
    echo "xterm"
  fi
}

_rec_is_allowed_terminal() {
  local allowed="${REC_TERMINALS:-kitty,tabby}"
  [[ ",${allowed:l}," == *",all,"* ]] && return 0
  local cur
  cur="$(_rec_detect_terminal)"
  [[ -n "$cur" && ",${allowed:l}," == *",${cur},"* ]]
}

rec() {
  _rec_load_config
  local sub="${1:-help}"
  if [[ "$sub" == "start" || "$sub" == "on" ]]; then
    shift
    # Remote start by PID (`rec start <pid> [name]`) or toggle live standby in this tab
    if [[ "${1:-}" =~ ^[0-9]+$ || -n "${REC_PTY_PID:-}" ]]; then
      rec-cli start "$@"
    else
      rec-pty ${1:+--name "$1"} -- "${SHELL:-zsh}"
    fi
    return $?
  fi
  rec-cli "$@"
}

alias recs='rec-cli list'
alias norec='NO_RECORD=1 zsh'
alias render-casts='casts2mp4'

# Log executed commands to the live recording's .meta symlink for process-based naming
if [[ -n "${_TERM_REC_META:-}" ]]; then
  zmodload -F zsh/datetime b:strftime p:EPOCHREALTIME 2>/dev/null || true
  _rec_preexec() {
    case "$1" in
      rec\ *|rec|rec-cli\ *|rec-cli) return 0 ;;
    esac
    if [[ -L "$_TERM_REC_META" || -f "$_TERM_REC_META" ]]; then
      local ts="${EPOCHREALTIME:-$(date +%s)}"
      printf '%s\t%s\n' "$ts" "$1" >> "$_TERM_REC_META" 2>/dev/null || true
    fi
  }
  autoload -Uz add-zsh-hook
  add-zsh-hook preexec _rec_preexec
fi

# Wrap interactive desktop terminals in rec-pty:
# - Allowed terminals (with auto-record ON) start recording immediately
# - All other terminals (or auto-record OFF) start in zero-overhead standby so rec-gui / rec start <pid> can record them in-place
if [[ -n "$(_rec_detect_terminal)" ]] \
  && [[ $- == *i* ]] \
  && [[ -t 0 && -t 1 ]] \
  && [[ "$TERM" != "dumb" ]] \
  && [[ -z "${REC_PTY_PID:-}" && -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" && -z "${NO_RECORD:-}" && -z "${CI:-}" && -z "${VSCODE_INJECTION:-}" ]] \
  && [[ "$(ps -o comm= -p $PPID 2>/dev/null)" != "ttyd" ]] \
  && command -v rec-pty >/dev/null 2>&1; then
  mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
  command -v casts2mp4 >/dev/null 2>&1 && casts2mp4 --bg
  if _rec_is_allowed_terminal \
    && [[ ! -f "${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/auto-disabled" ]] \
    && [[ ! -f "$HOME/.config/term-rec/auto-disabled" ]]; then
    exec rec-pty -- "${SHELL:-zsh}"
  else
    exec rec-pty --standby -- "${SHELL:-zsh}"
  fi
fi
