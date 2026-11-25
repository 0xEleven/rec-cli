# rec-cli Fish Integration
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

function _rec_is_allowed_terminal
    set -l allowed "kitty,tabby"
    if set -q REC_TERMINALS
        set allowed "$REC_TERMINALS"
    end
    set -l allowed_lc (string lower -- ",$allowed,")
    if string match -q '*,all,*' -- "$allowed_lc"
        return 0
    end

    set -l cur ""
    if set -q KITTY_WINDOW_ID; or set -q KITTY_PID; or test "$TERM" = "xterm-kitty"
        set cur "kitty"
    else if test "$TERM_PROGRAM" = "Tabby"
        set cur "tabby"
    else if set -q ALACRITTY_WINDOW_ID; or set -q ALACRITTY_SOCKET; or test "$TERM" = "alacritty"
        set cur "alacritty"
    else if test "$TERM_PROGRAM" = "WezTerm"; or set -q WEZTERM_PANE
        set cur "wezterm"
    else if test "$TERM_PROGRAM" = "ghostty"; or set -q GHOSTTY_RESOURCES_DIR
        set cur "ghostty"
    else if test "$TERM" = "foot"; or test "$TERM" = "foot-extra"
        set cur "foot"
    else if set -q KONSOLE_VERSION
        set cur "konsole"
    else if set -q GNOME_TERMINAL_SCREEN; or set -q VTE_VERSION
        set cur "gnome-terminal"
    end

    test -n "$cur"; and string match -q "*,{$cur},*" -- "$allowed_lc"
end

function rec
    _rec_load_config
    set -l sub "help"
    if test (count $argv) -ge 1
        set sub $argv[1]
    end

    switch $sub
        case start on
            if set -q ASCIINEMA_REC; or set -q _TERM_REC_ACTIVE
                printf '\033[38;2;247;118;142m● Already recording this terminal!\033[0m Run \033[1mrec stop\033[0m to finish & save, or \033[1mrec cancel\033[0m to discard.\n'
                return 0
            end
            if not type -q asciinema
                echo "Error: asciinema not found in PATH. Run 'rec doctor'." >&2
                return 1
            end
            set -l custom_name ""
            if test (count $argv) -ge 2
                set custom_name $argv[2]
            end
            mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
            set -l stamp (date +%Y%m%d-%H%M%S)-$fish_pid
            set -l cast_file "$TERM_REC_DIR/casts/session-$stamp.cast"
            set -l meta_file "$TERM_REC_DIR/casts/session-$stamp.meta"
            set -l ctrl_file "/tmp/term-rec-ctrl-$stamp"
            rm -f "$ctrl_file"

            if test -n "$custom_name"
                printf 'NAME\t%s\n' "$custom_name" > "$meta_file"
                printf '\033[38;2;247;118;142m● Recording started\033[0m (name: \033[1m%s\033[0m) — run \033[1mrec stop\033[0m when done.\n' "$custom_name"
            else
                printf '\033[38;2;247;118;142m● Recording started\033[0m (auto-named by process) — run \033[1mrec stop\033[0m when done.\n'
            end

            env _TERM_REC_ACTIVE=1 \
                _TERM_REC_CAST="$cast_file" \
                _TERM_REC_META="$meta_file" \
                _TERM_REC_CTRL="$ctrl_file" \
                asciinema rec --quiet --stdin --idle-time-limit 2 "$cast_file"

            set -l action "save"
            set -l saved_pwd ""
            if test -f "$ctrl_file"
                set action (sed -n '1p' "$ctrl_file")
                set saved_pwd (sed -n '2p' "$ctrl_file")
                rm -f "$ctrl_file"
            end
            if test -n "$saved_pwd" -a -d "$saved_pwd"
                builtin cd "$saved_pwd"
            end

            set -l speed 2
            if set -q REC_SPEED
                set speed "$REC_SPEED"
            else if set -q CAST_SPEED
                set speed "$CAST_SPEED"
            end

            if test "$action" = "cancel"
                rm -f "$cast_file" "$meta_file"
                printf '\033[38;2;224;175;104m✕ Recording canceled & discarded.\033[0m\n'
            else if test -s "$cast_file"; and type -q cast2mp4
                if type -q setsid
                    setsid -f cast2mp4 --auto --cleanup --speed "$speed" "$cast_file" </dev/null >/dev/null 2>&1
                else
                    nohup cast2mp4 --auto --cleanup --speed "$speed" "$cast_file" </dev/null >/dev/null 2>&1 &
                end
                printf '\033[38;2;158;206;106m■ Recording stopped\033[0m — rendering %sx MP4 to \033[38;2;125;207;255m%s\033[0m (run \033[1mrec list\033[0m).\n' "$speed" "$TERM_VIDEO_DIR"
            else
                rm -f "$cast_file" "$meta_file"
                echo "■ Recording stopped (empty session discarded)."
            end

        case stop off
            if not set -q ASCIINEMA_REC; and not set -q _TERM_REC_ACTIVE
                echo "Not currently recording in this terminal."
                return 0
            end
            if set -q _TERM_REC_CTRL
                printf 'stop\n%s\n' "$PWD" > "$_TERM_REC_CTRL"
            end
            exit 0

        case cancel discard
            if not set -q ASCIINEMA_REC; and not set -q _TERM_REC_ACTIVE
                echo "Not currently recording in this terminal."
                return 0
            end
            if set -q _TERM_REC_CTRL
                printf 'cancel\n%s\n' "$PWD" > "$_TERM_REC_CTRL"
            end
            exit 0

        case '*'
            rec-cli $argv
    end
end

alias recs 'rec-cli list'
alias norec 'env NO_RECORD=1 fish'
alias render-casts 'casts2mp4'

# Log executed commands to sidecar .meta via Fish preexec event
if set -q _TERM_REC_META
    function _rec_fish_preexec --on-event fish_preexec
        set -l cmd "$argv[1]"
        if string match -qr '^rec(\s|$)' -- "$cmd"
            return 0
        end
        set -l ts (date +%s.%3N 2>/dev/null; or date +%s)
        printf '%s\t%s\n' "$ts" "$cmd" >> "$_TERM_REC_META" 2>/dev/null
    end
end

# Auto-record on allowed terminals unless disabled via `rec auto off`
set -l _rec_auto_off "$HOME/.config/rec-cli/auto-disabled"
if set -q XDG_CONFIG_HOME
    set _rec_auto_off "$XDG_CONFIG_HOME/rec-cli/auto-disabled"
end

if status is-interactive
    and _rec_is_allowed_terminal
    and not test -f "$_rec_auto_off"
    and not test -f "$HOME/.config/term-rec/auto-disabled"
    and test "$TERM" != "dumb"
    and not set -q ASCIINEMA_REC
    and not set -q _TERM_REC_ACTIVE
    and not set -q NO_RECORD
    and not set -q CI
    and not set -q VSCODE_INJECTION
    and type -q asciinema
    mkdir -p "$TERM_REC_DIR/casts" "$TERM_VIDEO_DIR"
    if type -q casts2mp4
        casts2mp4 --bg
    end
    set -l _rec_stamp (date +%Y%m%d-%H%M%S)-$fish_pid
    set -l _rec_cast "$TERM_REC_DIR/casts/session-$_rec_stamp.cast"
    set -l _rec_meta "$TERM_REC_DIR/casts/session-$_rec_stamp.meta"
    set -l _rec_ctrl "/tmp/term-rec-ctrl-$_rec_stamp"
    rm -f "$_rec_ctrl"

    env _TERM_REC_ACTIVE=1 \
        _TERM_REC_CAST="$_rec_cast" \
        _TERM_REC_META="$_rec_meta" \
        _TERM_REC_CTRL="$_rec_ctrl" \
        asciinema rec --quiet --stdin --idle-time-limit 2 "$_rec_cast"

    set -l _rec_action "close"
    set -l _rec_pwd ""
    if test -f "$_rec_ctrl"
        set _rec_action (sed -n '1p' "$_rec_ctrl")
        set _rec_pwd (sed -n '2p' "$_rec_ctrl")
        rm -f "$_rec_ctrl"
    end

    set -l _rec_speed 2
    if set -q REC_SPEED
        set _rec_speed "$REC_SPEED"
    else if set -q CAST_SPEED
        set _rec_speed "$CAST_SPEED"
    end

    if test "$_rec_action" = "cancel"
        rm -f "$_rec_cast" "$_rec_meta"
    else if test -s "$_rec_cast"; and type -q cast2mp4
        if type -q setsid
            setsid -f cast2mp4 --auto --cleanup --speed "$_rec_speed" "$_rec_cast" </dev/null >/dev/null 2>&1
        else
            nohup cast2mp4 --auto --cleanup --speed "$_rec_speed" "$_rec_cast" </dev/null >/dev/null 2>&1 &
        end
    else
        rm -f "$_rec_cast" "$_rec_meta"
    end

    if test "$_rec_action" = "stop" -o "$_rec_action" = "cancel"
        if test -n "$_rec_pwd" -a -d "$_rec_pwd"
            builtin cd "$_rec_pwd"
        end
        if test "$_rec_action" = "cancel"
            printf '\033[38;2;224;175;104m✕ Recording canceled & discarded.\033[0m\n'
        else
            printf '\033[38;2;158;206;106m■ Recording stopped\033[0m — rendering %sx MP4 to \033[38;2;125;207;255m%s\033[0m (tab kept open).\n' "$_rec_speed" "$TERM_VIDEO_DIR"
        end
    else
        exit 0
    end
end
