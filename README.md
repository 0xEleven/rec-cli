<p align="center">
  <img src="assets/logo.svg" alt="rec-cli" width="128">
</p>

<h1 align="center">rec-cli</h1>

<p align="center">
  Fast terminal clip recorder and native Rust desktop app for Linux (Zsh, Bash, Fish).<br>
  Record crisp demos, tutorials, content clips, and proof-of-concepts up to 4K MP4 or 9:16 vertical.
</p>

<p align="center">
  <a href="https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml"><img src="https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/0xEleven/rec-cli/releases"><img src="https://github.com/0xEleven/rec-cli/releases/latest/badge.svg" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-bb9af7.svg" alt="License: MIT"></a>
</p>

## What it does

- **Fast Native Rust Desktop App (`rec gui`)**: Clean, responsive Dark & Light interface built in Rust (`rec-gui`) for everyday users — start a new recorded terminal in 1 click, pause/resume/snapshot/bookmark live sessions, browse your clips with real video thumbnails, export GIFs, rename, trim losslessly, and configure video themes & terminal apps without touching the command line.
- **Live Terminal Recording Control (`rec start`, `rec stop`, `rec save`, `rec ps`)**: Start, stop, pause, resume, snapshot, discard, or add YouTube chapter markers to any live terminal or SSH session without ever closing your tab or dropping SSH.
- **Smart Auto-Render Pipeline**: Exports crisp terminal MP4s at `auto`, `720p`, `1080p`, `1440p`, `4k`, `reels` (9:16 cursor-following vertical), or `reels-4k`, with customizable speed, 5-minute chunking, auto-naming, rounded window frames, and automatic secret/password redaction.
- **Manual-First or Auto-Record**: New terminals start in zero-overhead standby ready for 1-click recording, or flip on `Auto-record` for specific terminal apps (Kitty, Tabby, Alacritty, WezTerm, Ghostty, Foot, Konsole, GNOME Terminal).

## Install

### All Linux distros (universal script)

Handles `apt`, `pacman`, `dnf`, `yum`, `zypper`, `apk`, `xbps-install`, and `emerge`, installs `agg`, builds `rec-gui`, and adds the shell hook:

```bash
git clone https://github.com/0xEleven/rec-cli.git
cd rec-cli
./scripts/install.sh
```

### Debian / Ubuntu / Kali / Mint / Pop!_OS

Download the `.deb` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo apt install ./rec-cli_2.1.0_amd64.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE

Download the `.rpm` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo dnf install ./rec-cli-2.1.0-1.x86_64.rpm
# openSUSE
sudo zypper install ./rec-cli-2.1.0-1.x86_64.rpm
```

### Arch / Manjaro / EndeavourOS

```bash
cd packaging/arch && makepkg -si
```

## Shell setup

If you installed from a `.deb`, `.rpm`, or `PKGBUILD`, add one line to your shell config:

- **Zsh** (`~/.zshrc`): `eval "$(rec-cli init zsh)"`
- **Bash** (`~/.bashrc`): `eval "$(rec-cli init bash)"`
- **Fish** (`~/.config/fish/config.fish`): `rec-cli init fish | source`

Optional Starship `● REC` prompt badge: paste [starship/rec-module.toml](starship/rec-module.toml) into `~/.config/starship.toml` and include `${custom.rec}` in your `format`.

## Commands

| Command | Action |
| --- | --- |
| `rec gui` | Open the native Rust desktop app (live sessions, 1-click record, clips library & settings) |
| `rec ps` | List all live recording sessions with PID, state (`REC`/`PAUSED`/`STANDBY`), name, elapsed time, size, and running command |
| `rec start [--new] [name]` | Start recording this terminal (or `rec start <pid> [--new] [name]`); refuses to silently stop an active clip unless `--new` is passed |
| `rec stop [--bg] [pid\|name\|all]` | Stop and save MP4 right now without closing the tab or dropping SSH (`--bg` renders in background) |
| `rec save [pid\|name] [label]` | Save an instant MP4 snapshot right now without stopping the active recording |
| `rec pause [pid\|name\|all]` | Pause a recording live (keeps shell/SSH running) |
| `rec resume [pid\|name\|all]` | Resume a paused recording |
| `rec cancel [pid\|name\|all]` | Stop and discard the take without saving |
| `rec mark [pid\|name] <label>` | Add a chapter marker to this or another PID (writes `.chapters.txt` next to the MP4) |
| `rec kill <pid\|name\|all>` | Force-terminate a recording session process |
| `rec status` | Show current tab status, active PIDs, and saved clip count |
| `rec auto on\|off` | Toggle auto-recording on new tabs (default: `off` — manual `rec start` / `rec stop`) |
| `rec list` | List saved MP4 clips |
| `rec play [clip\|latest]` | Open a clip in `mpv`, `vlc`, or your default player |
| `rec gif [clip\|latest]` | Convert a clip to a palette-optimized GIF |
| `rec thumb [clip\|latest]` | Save a PNG thumbnail from a clip |
| `rec trim <clip> <start> <end>` | Fast lossless trim (`rec trim latest 00:05 01:20`) |
| `rec reels <file.cast>` | Render a 9:16 TikTok/Reels cursor-following clip (zero crop) |
| `rec stats` | Show clip count, total duration, disk usage, and top tools |
| `rec config auto` | Auto-pick resolution based on this machine's screen |
| `rec config [key val]` | View or change settings |
| `rec open` | Open the clips directory |
| `rec clear [--yes]` | Delete saved clips and raw casts |
| `rec doctor` | Check installed dependencies |

## Configuration

Settings live in `~/.config/rec-cli/config` (and can all be edited in `rec gui`):

```bash
rec config auto                   # auto-detect display resolution (720p/1080p/1440p/4k)
rec config resolution 4k          # auto | 720p | 1080p | 1440p | 4k | reels | reels-4k
rec config reels-copy on          # also export a 9:16 cursor-following Reels clip alongside landscape
rec config speed 2                # playback speed multiplier
rec config max-secs 300           # max clip length in seconds
rec config frame on               # on | off
rec config redact on              # on | off
rec config theme tokyo-night      # tokyo-night | dracula | catppuccin | gruvbox | nord
rec config terminals kitty,tabby  # comma-separated list, or "all"
rec config video-dir "$HOME/Videos/content"
```

## Build & test locally

```bash
make test
make packages
```

## License

[MIT](LICENSE)
