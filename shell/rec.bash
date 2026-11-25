# rec-cli Bash Integration
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
  case "$sub" in
    start|on)
      shift
      if [[ -n "${ASCIINEMA_REC:-}" || -n "${_TERM_REC_ACTIVE:-}" ]]; then
        printf '\033[38;2;247;118;142m● Already recording this terminal!\033[0m Run \033[1mrec stop\033[0m to finish & save, or \033[1mrec cancel\033[0m to discard.\n'
        return 0
      fi
      if ! command -v asciinema >/dev/null 2>&1; then
        echo "Error: asciinema not found in PATH. Run 'rec doctor'." >&2
        return 1
      fi
      local custom_name="${1:-}"
      mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
      local stamp
      stamp="$(date +%Y%m%d-%H%M%S)-$$"
      local cast_file="$TERM_REC_DIR/casts/session-${stamp}.cast"
      local meta_file="${cast_file%.cast}.meta"
      local ctrl_file="/tmp/term-rec-ctrl-${stamp}"
      rm -f "$ctrl_file"
      if [[ -n "$custom_name" ]]; then
        printf 'NAME\t%s\n' "$custom_name" > "$meta_file"
        printf '\033[38;2;247;118;142m● Recording started\033[0m (name: \033[1m%s\033[0m) — run \033[1mrec stop\033[0m when done.\n' "$custom_name"
      else
        printf '\033[38;2;247;118;142m● Recording started\033[0m (auto-named by process) — run \033[1mrec stop\033[0m when done.\n'
      fi
      _TERM_REC_ACTIVE=1 \
      _TERM_REC_CAST="$cast_file" \
      _TERM_REC_META="$meta_file" \
      _TERM_REC_CTRL="$ctrl_file" \
      asciinema rec --quiet --stdin --idle-time-limit 2 "$cast_file"

      local action="save" saved_pwd=""
      if [[ -f "$ctrl_file" ]]; then
        action="$(sed -n '1p' "$ctrl_file")"
        saved_pwd="$(sed -n '2p' "$ctrl_file")"
        rm -f "$ctrl_file"
      fi
      [[ -n "$saved_pwd" && -d "$saved_pwd" ]] && builtin cd "$saved_pwd" || true

      if [[ "$action" == "cancel" ]]; then
        rm -f "$cast_file" "$meta_file"
        printf '\033[38;2;224;175;104m✕ Recording canceled & discarded.\033[0m\n'
      elif [[ -s "$cast_file" ]] && command -v cast2mp4 >/dev/null 2>&1; then
        if command -v setsid >/dev/null 2>&1; then
          setsid -f cast2mp4 --auto --cleanup --speed "${REC_SPEED:-${CAST_SPEED:-2}}" "$cast_file" </dev/null >/dev/null 2>&1
        else
          nohup cast2mp4 --auto --cleanup --speed "${REC_SPEED:-${CAST_SPEED:-2}}" "$cast_file" </dev/null >/dev/null 2>&1 &
          disown 2>/dev/null || true
        fi
        printf '\033[38;2;158;206;106m■ Recording stopped\033[0m — rendering %sx MP4 to \033[38;2;125;207;255m%s\033[0m (run \033[1mrec list\033[0m).\n' "${REC_SPEED:-${CAST_SPEED:-2}}" "$TERM_VIDEO_DIR"
      else
        rm -f "$cast_file" "$meta_file"
        echo "■ Recording stopped (empty session discarded)."
      fi
      ;;
    stop|off)
      if [[ -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" ]]; then
        echo "Not currently recording in this terminal."
        return 0
      fi
      [[ -n "${_TERM_REC_CTRL:-}" ]] && printf 'stop\n%s\n' "$PWD" > "$_TERM_REC_CTRL"
      exit 0
      ;;
    cancel|discard)
      if [[ -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" ]]; then
        echo "Not currently recording in this terminal."
        return 0
      fi
      [[ -n "${_TERM_REC_CTRL:-}" ]] && printf 'cancel\n%s\n' "$PWD" > "$_TERM_REC_CTRL"
      exit 0
      ;;
    *)
      rec-cli "$@"
      ;;
  esac
}

alias recs='rec-cli list'
alias norec='NO_RECORD=1 bash'
alias render-casts='casts2mp4'

# Log executed commands to sidecar .meta via Bash DEBUG trap
if [[ -n "${_TERM_REC_META:-}" ]]; then
  _rec_bash_in_prompt=0
  _rec_bash_preexec() {
    [[ "$_rec_bash_in_prompt" -eq 1 ]] && return 0
    local cmd="${BASH_COMMAND:-}"
    case "$cmd" in
      ""|_rec_*|rec\ *|rec|PROMPT_COMMAND*) return 0 ;;
    esac
    local ts
    ts="$(date +%s.%3N 2>/dev/null || date +%s)"
    printf '%s\t%s\n' "$ts" "$cmd" >> "$_TERM_REC_META" 2>/dev/null || true
    _rec_bash_in_prompt=1
  }
  _rec_bash_precmd() {
    _rec_bash_in_prompt=0
  }
  PROMPT_COMMAND="_rec_bash_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
  trap '_rec_bash_preexec' DEBUG
fi

# Auto-record on allowed terminals unless disabled via `rec auto off`
if _rec_is_allowed_terminal \
  && [[ ! -f "${XDG_CONFIG_HOME:-$HOME/.config}/rec-cli/auto-disabled" ]] \
  && [[ ! -f "$HOME/.config/term-rec/auto-disabled" ]] \
  && [[ $- == *i* ]] \
  && [[ -t 0 && -t 1 ]] \
  && [[ "${TERM:-}" != "dumb" ]] \
  && [[ -z "${ASCIINEMA_REC:-}" && -z "${_TERM_REC_ACTIVE:-}" && -z "${NO_RECORD:-}" && -z "${CI:-}" && -z "${VSCODE_INJECTION:-}" ]] \
  && [[ "$(ps -o comm= -p "$PPID" 2>/dev/null)" != "ttyd" ]] \
  && command -v asciinema >/dev/null 2>&1; then
  mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
  command -v casts2mp4 >/dev/null 2>&1 && casts2mp4 --bg
  _rec_stamp="$(date +%Y%m%d-%H%M%S)-$$"
  _rec_cast="$TERM_REC_DIR/casts/session-${_rec_stamp}.cast"
  _rec_meta="${_rec_cast%.cast}.meta"
  _rec_ctrl="/tmp/term-rec-ctrl-${_rec_stamp}"
  rm -f "$_rec_ctrl"
  _rec_finalize() {
    trap - EXIT HUP TERM INT
    local _act="${1:-close}"
    if [[ "$_act" == "close" && -f "$_rec_ctrl" ]]; then
      _act="$(sed -n '1p' "$_rec_ctrl")"
    fi
    rm -f "$_rec_ctrl"
    if [[ "$_act" == "cancel" ]]; then
      rm -f "$_rec_cast" "$_rec_meta"
    elif [[ -s "$_rec_cast" ]] && command -v cast2mp4 >/dev/null 2>&1; then
      if command -v setsid >/dev/null 2>&1; then
        setsid -f cast2mp4 --auto --cleanup --speed "${REC_SPEED:-${CAST_SPEED:-2}}" "$_rec_cast" </dev/null >/dev/null 2>&1
      else
        nohup cast2mp4 --auto --cleanup --speed "${REC_SPEED:-${CAST_SPEED:-2}}" "$_rec_cast" </dev/null >/dev/null 2>&1 &
        disown 2>/dev/null || true
      fi
    else
      rm -f "$_rec_cast" "$_rec_meta"
    fi
  }
  trap '_rec_finalize close' EXIT HUP TERM
  _TERM_REC_ACTIVE=1 \
  _TERM_REC_CAST="$_rec_cast" \
  _TERM_REC_META="$_rec_meta" \
  _TERM_REC_CTRL="$_rec_ctrl" \
  asciinema rec --quiet --stdin --idle-time-limit 2 "$_rec_cast"

  _rec_action="close"
  _rec_pwd=""
  if [[ -f "$_rec_ctrl" ]]; then
    _rec_action="$(sed -n '1p' "$_rec_ctrl")"
    _rec_pwd="$(sed -n '2p' "$_rec_ctrl")"
  fi
  _rec_finalize "$_rec_action"

  if [[ "$_rec_action" == "stop" || "$_rec_action" == "cancel" ]]; then
    [[ -n "$_rec_pwd" && -d "$_rec_pwd" ]] && builtin cd "$_rec_pwd" || true
    if [[ "$_rec_action" == "cancel" ]]; then
      printf '\033[38;2;224;175;104m✕ Recording canceled & discarded.\033[0m\n'
    else
      printf '\033[38;2;158;206;106m■ Recording stopped\033[0m — rendering %sx MP4 to \033[38;2;125;207;255m%s\033[0m (tab kept open).\n' "${REC_SPEED:-${CAST_SPEED:-2}}" "$TERM_VIDEO_DIR"
    fi
  else
    exit 0
  fi
fi
