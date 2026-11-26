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

- **Studio Desktop App (`rec gui`)**: Clean monochrome Qt6 GUI with Dark and Light mode, live terminal PID manager (1-click in-place recording via silent standby), visual terminal auto-record picker, screen/region recorder, multi-track video editor, and clips library.
- **Recordly-style Multi-Track Studio Editor (`rec edit [clip|project]`)**:
  - **Smart Auto-Zoom & Manual Zoom Regions**: 1-click Auto-Zoom from cursor/motion activity plus draggable zoom regions on the timeline with critically damped spring easing, click-on-canvas focus targeting, and motion blur.
  - **Interactive Multi-Track Timeline**: Drag and resize Zoom regions, Trim Cuts (remove unwanted sections), Speed regions (0.25x to 4x), Overlays, and supplementary Audio tracks, with Undo/Redo and Spacebar preview playback.
  - **Wallpapers, Frames & Aspect Ratios**: 8 built-in studio wallpapers, custom gradients, solid fills, or blurred wallpaper images, padding, corner radius, drop shadows, macOS/minimal window chrome, crop controls, and aspect ratio presets (`16:9`, `9:16`, `1:1`, `4:5`, `4:3`, `21:9`, `Original`).
  - **Cursor Polish**: Smoothed macOS-style vector cursor, highlight ring or dot modes, click bounce & ripple animation, and auto-hide when idle.
  - **Annotations & Dynamic Webcam Bubble**: Timed Text callouts, Blur/Redact boxes, Highlight boxes, Arrows, Image overlays, and a floating rounded webcam bubble that auto-shrinks during zooms.
  - **Projects & Export**: Save and reopen non-destructive `.recproj` project files, and export to MP4 (`720p` to `4K UHD`, 30/60fps, synced AAC audio) or palette-optimized GIF (`rec render-project`).
- **Screen, Region, Audio & Webcam Capture (`rec screen`)**: Record full screen or custom regions on Wayland (GNOME DBus / `wf-recorder`) or X11 (`x11grab`) with Microphone + System Audio, synchronized webcam sidecar, cursor tracking, and live zoom point flagging (`SIGUSR2`).
- **Process-Managed Parallel Terminal Recording (`rec ps`)**: Start, stop, pause, resume, cancel, or drop YouTube chapter markers on any live terminal or SSH session by PID or name from any other terminal without dropping SSH.
- **Smart Auto-Render Pipeline**: Records terminal sessions at `auto`, `720p`, `1080p`, `1440p`, `4k`, `reels` (9:16 cursor-following auto-reframe), or `reels-4k`, at 2x speed in 5-minute chunks, auto-named after the tool you ran, with automatic secret redaction and passive OpenVPN/log skip.

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
sudo apt install ./rec-cli_2.0.0_all.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE

Download the `.rpm` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo dnf install ./rec-cli-2.0.0-1.noarch.rpm
# openSUSE
sudo zypper install ./rec-cli-2.0.0-1.noarch.rpm
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
| `rec gui` | Open the Qt Studio GUI (Dark/Light mode, live PID sessions, Screen Rec, Studio Editor, terminal picker, clips library) |
| `rec edit [clip\|project]` | Open any MP4, `.cast`, or `.recproj` directly in the multi-track Studio Editor |
| `rec screen [flags]` | Open the Screen Recorder tab (or capture via CLI with `--mic`, `--system-audio`, `--webcam`, `--area`) |
| `rec render-project <f.recproj>` | Render a saved `.recproj` project headlessly to MP4 or GIF |
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
