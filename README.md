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

- Records terminal sessions into MP4 videos at `720p`, `1080p`, `1440p`, `4k` (default), or `vertical` (1080x1920 for Shorts/TikTok/Reels).
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
sudo apt install ./rec-cli_1.1.0_all.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE

Download the `.rpm` from [Releases](https://github.com/0xEleven/rec-cli/releases):

```bash
sudo dnf install ./rec-cli-1.1.0-1.noarch.rpm
# openSUSE
sudo zypper install ./rec-cli-1.1.0-1.noarch.rpm
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
| `rec start [name]` | Start recording the current terminal (auto-named by process if no name is given) |
| `rec stop` | Stop recording, keep the tab open in your current directory, and render the MP4 |
| `rec cancel` | Stop recording and throw away the take |
| `rec mark <label>` | Add a chapter marker (writes a `.chapters.txt` file next to the MP4) |
| `rec status` | Show current tab status, auto-record mode, and saved clip count |
| `rec auto on\|off` | Switch between auto-recording new tabs and manual `rec start` |
| `rec list` | List saved MP4 clips |
| `rec play [clip\|latest]` | Open a clip in `mpv`, `vlc`, or your default player |
| `rec gif [clip\|latest]` | Convert a clip to a palette-optimized GIF |
| `rec thumb [clip\|latest]` | Save a PNG thumbnail from a clip |
| `rec trim <clip> <start> <end>` | Fast lossless trim (`rec trim latest 00:05 01:20`) |
| `rec stats` | Show clip count, total duration, disk usage, and top tools |
| `rec config [key val]` | View or change settings |
| `rec open` | Open the clips directory |
| `rec clear [--yes]` | Delete saved clips and raw casts |
| `rec doctor` | Check installed dependencies |

## Configuration

Settings live in `~/.config/rec-cli/config`:

```bash
rec config resolution 4k          # 720p | 1080p | 1440p | 4k | vertical
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
