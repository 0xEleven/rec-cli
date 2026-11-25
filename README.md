<p align="center">
  <img src="assets/logo.svg" alt="rec-cli" width="128">
</p>

<h1 align="center">rec-cli</h1>

<p align="center">
  Terminal clip recorder for Linux (Zsh, Bash, Fish). Up to 4K MP4s, 2x speed, 5-minute chunks, named after what you ran.
</p>

<p align="center">
  <a href="https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml"><img src="https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/0xEleven/rec-cli/releases"><img src="https://github.com/0xEleven/rec-cli/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-bb9af7.svg" alt="License: MIT"></a>
</p>

## What it does

- Desktop GUI (`rec gui` or app launcher) built with Qt6 (`PyQt6`/`PySide6`/`PyQt5`), featuring Dark & Light mode, a visual terminal picker, live PID session controls, instant settings, and a clips library browser.
- Manage parallel recordings like processes (`rec ps`): start, stop, pause, resume, cancel, or drop chapter markers on ANY recording by PID or name from ANY other terminal, even while SSHed into a remote machine, without dropping your SSH session.
- Records terminal sessions into MP4 videos at `auto` (detects your screen resolution automatically), `720p`, `1080p`, `1440p`, `4k`, `reels` (9:16 cursor-following auto-reframe), or `reels-4k`.
- Auto-reframe for TikTok, Instagram Reels, and YouTube Shorts tracks your cursor with smooth critically-damped springs and automatically fits full multi-line commands and wide output (like `nmap` tables) so nothing gets cropped.
- Renders at 2x speed by default so typing looks snappy, and splits long sessions into 5-minute clips.
- Names clips after the tool you actually ran (`nmap_20261125-134012.mp4`, `ffuf_..._part2.mp4`), ignoring `sudo`, `cd`, `ls`, and `clear`.
- Only keeps sessions with both keyboard input and terminal output. Skips passive background panes like `openvpn`, `tail -f`, `watch`, and `htop`.
- Redacts leaked secrets (AWS keys, GitHub tokens, API keys, JWTs, `PASSWORD=...`) before rendering.
- Wraps videos in a clean rounded window frame with a titlebar (toggleable).
- Only auto-records in terminals you allow (`kitty` and `tabby` by default).
- Deletes raw `.cast` files after rendering so disk space is not wasted.

## Install

### All Linux distros (universal script)

Handles `apt`, `pacman`, `dnf`, `yum`, `zypper`, `apk`, `xbps-install`, and `emerge`, installs `agg`, and adds the shell hook:

```bash
git clone https://github.com/0xEleven/rec-cli.git
cd rec-cli
./scripts/install.sh
```

### Debian / Ubuntu / Kali / Mint / Pop!_OS

Download the `.deb` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo apt install ./rec-cli_1.4.0_all.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE

Download the `.rpm` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo dnf install ./rec-cli-1.4.0-1.noarch.rpm
# openSUSE
sudo zypper install ./rec-cli-1.4.0-1.noarch.rpm
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
| `rec gui` | Open the Qt desktop GUI (Dark/Light mode, terminal selector, live PID controls, clips library) |
| `rec ps` | List all live recording sessions with PID, state (`REC`/`PAUSED`/`STANDBY`), name, elapsed time, size, and running command |
| `rec start [name]` | Start recording this terminal, or run `rec start <pid> [name]` to start a fresh clip on another live session |
| `rec stop [pid\|name\|all]` | Stop and render MP4 without closing the tab or dropping SSH |
| `rec pause [pid\|name\|all]` | Pause a recording live (keeps shell/SSH running) |
| `rec resume [pid\|name\|all]` | Resume a paused recording |
| `rec cancel [pid\|name\|all]` | Stop and discard the take without saving |
| `rec mark [pid\|name] <label>` | Add a chapter marker to this or another PID (writes `.chapters.txt` next to the MP4) |
| `rec kill <pid\|name\|all>` | Force-terminate a recording session process |
| `rec status` | Show current tab status, active PIDs, and saved clip count |
| `rec auto on\|off` | Switch between auto-recording new tabs and manual `rec start` |
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

Settings live in `~/.config/rec-cli/config`:

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

## Build packages locally

```bash
make test
make packages
```

## License

[MIT](LICENSE)
