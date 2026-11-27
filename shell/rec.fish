# rec-cli Fish Integration (v2.0.0 — process-managed via rec-pty)
# Add to ~/.config/fish/config.fish:  rec-cli init fish | source

function _rec_load_config
    set -l cfg "$HOME/.config/rec-cli/config"
    if set -q XDG_CONFIG_HOME
        set cfg "$XDG_CONFIG_HOME/rec-cli/config"
    end
    if test -f "$cfg"
        for line in (cat "$cfg")
            if string match -qr '^[A-Za-z_][A-Za-z0-9_]*=' -- "$line"
                set -l kv (string split -m 1 '=' -- "$line")
                set -l val (string trim -c "\"'" -- "$kv[2]")
                set -gx $kv[1] "$val"
            end
        end
    end
    if not set -q TERM_REC_DIR
        if set -q REC_DIR
            set -gx TERM_REC_DIR "$REC_DIR"
        else
            set -gx TERM_REC_DIR "$HOME/Videos/terminal-recordings"
        end
    end
    if not set -q TERM_VIDEO_DIR
        if set -q REC_VIDEO_DIR
            set -gx TERM_VIDEO_DIR "$REC_VIDEO_DIR"
        else
            set -gx TERM_VIDEO_DIR "$HOME/Videos/content"
        end
    end
end
_rec_load_config

function _rec_sanitize_env
    if set -q REC_PTY_PID; and test -n "$REC_PTY_PID"
        if not kill -0 "$REC_PTY_PID" 2>/dev/null
            set -e REC_PTY_PID REC_PTY_TTY REC_RUNTIME_DIR _TERM_REC_ACTIVE _TERM_REC_META _TERM_REC_MARKER
        else if set -q REC_PTY_TTY; and test -n "$REC_PTY_TTY"; and test -t 0
            set -l cur_tty (tty 2>/dev/null)
            if test -n "$cur_tty" -a "$cur_tty" != "not a tty" -a "$cur_tty" != "$REC_PTY_TTY"
                set -e REC_PTY_PID REC_PTY_TTY REC_RUNTIME_DIR _TERM_REC_ACTIVE _TERM_REC_META _TERM_REC_MARKER
            end
        end
    end
end
_rec_sanitize_env

function _rec_detect_terminal
    if set -q KITTY_WINDOW_ID; or set -q KITTY_PID; or test "$TERM" = "xterm-kitty"
        echo "kitty"
    else if test "$TERM_PROGRAM" = "Tabby"; or set -q TABBY_CONFIG_DIRECTORY
        echo "tabby"
    else if set -q ALACRITTY_WINDOW_ID; or set -q ALACRITTY_SOCKET; or test "$TERM" = "alacritty"
        echo "alacritty"
    else if test "$TERM_PROGRAM" = "WezTerm"; or set -q WEZTERM_PANE
        echo "wezterm"
    else if test "$TERM_PROGRAM" = "ghostty"; or set -q GHOSTTY_RESOURCES_DIR
        echo "ghostty"
    else if test "$TERM" = "foot"; or test "$TERM" = "foot-extra"
        echo "foot"
    else if set -q KONSOLE_VERSION
        echo "konsole"
    else if set -q GNOME_TERMINAL_SCREEN; or set -q VTE_VERSION
        echo "gnome-terminal"
    else if begin; set -q DISPLAY; or set -q WAYLAND_DISPLAY; end; and string match -q 'xterm*' -- "$TERM"
        echo "xterm"
    end
end

function _rec_is_allowed_terminal
    set -l allowed "kitty,tabby"
    if set -q REC_TERMINALS
        set allowed "$REC_TERMINALS"
    end
    set -l allowed_lc (string lower -- ",$allowed,")
    if string match -q '*,all,*' -- "$allowed_lc"
        return 0
    end
    set -l cur (_rec_detect_terminal)
    test -n "$cur"; and string match -q "*,{$cur},*" -- "$allowed_lc"
end

function _rec_is_auto_enabled
    set -l auto_off "$HOME/.config/rec-cli/auto-disabled"
    if set -q XDG_CONFIG_HOME
        set auto_off "$XDG_CONFIG_HOME/rec-cli/auto-disabled"
    end
    if test -f "$auto_off" -o -f "$HOME/.config/term-rec/auto-disabled"
        return 1
    end
    if not set -q REC_AUTO_RECORD
        return 1
    end
    set -l auto_lc (string lower -- "$REC_AUTO_RECORD")
    switch "$auto_lc"
        case on enable enabled 1 true yes
            return 0
        case '*'
            return 1
    end
end

function rec
    _rec_load_config
    _rec_sanitize_env
    set -l sub "help"
    if test (count $argv) -ge 1
        set sub $argv[1]
    end
    if test "$sub" = "start" -o "$sub" = "on"
        set -l rest $argv[2..]
        set -l first_pos ""
        for a in $rest
            if test "$a" = "--new" -o "$a" = "--restart" -o "$a" = "-n"
                continue
            end
            set first_pos "$a"
            break
        end
        if test -n "$first_pos"; and string match -qr '^[0-9]+$' -- "$first_pos"
            rec-cli start $rest
        else if set -q REC_PTY_PID
            rec-cli start $rest
        else if test -n "$first_pos"
            rec-pty --name "$first_pos" -- fish
        else
            rec-pty -- fish
        end
        return $status
    end
    rec-cli $argv
end

alias recs 'rec-cli list'
alias norec 'env NO_RECORD=1 fish'
alias render-casts 'casts2mp4'

# Log executed commands to the live recording's .meta symlink via Fish preexec event
if set -q _TERM_REC_META
    function _rec_fish_preexec --on-event fish_preexec
        set -l cmd "$argv[1]"
        if string match -qr '^(rec|rec-cli)(\s|$)' -- "$cmd"
            return 0
        end
        if test -L "$_TERM_REC_META" -o -f "$_TERM_REC_META"
            set -l ts (date +%s.%3N 2>/dev/null; or date +%s)
            printf '%s\t%s\n' "$ts" "$cmd" >> "$_TERM_REC_META" 2>/dev/null
        end
    end
end

# Wrap interactive desktop terminals in rec-pty (manual standby by default; auto-record only if enabled via `rec auto on`)
if status is-interactive
    and test -n "(_rec_detect_terminal)"
    and test "$TERM" != "dumb"
    and not set -q REC_PTY_PID
    and not set -q ASCIINEMA_REC
    and not set -q _TERM_REC_ACTIVE
    and not set -q NO_RECORD
    and not set -q CI
    and not set -q VSCODE_INJECTION
    and type -q rec-pty
    mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
    if _rec_is_allowed_terminal
        and _rec_is_auto_enabled
        exec rec-pty --auto -- fish
    else
        exec rec-pty --standby -- fish
    end
end
