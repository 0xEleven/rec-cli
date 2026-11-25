# rec-cli

[![CI](https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml)
[![Release](https://github.com/0xEleven/rec-cli/actions/workflows/release.yml/badge.svg)](https://github.com/0xEleven/rec-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-bb9af7.svg)](LICENSE)

**Smart terminal clip recorder CLI for Linux — built for content creators, pentesters, and developers.**

Supports **Zsh, Bash, and Fish** across **all major Linux distributions** (Debian, Ubuntu, Kali, Mint, Pop!_OS, Arch, Manjaro, EndeavourOS, Fedora, RHEL, Rocky, Alma, openSUSE, Alpine, Void, Gentoo).

---

## Why `rec-cli`?

- **Start / Stop / Cancel on demand** — `rec start`, `rec stop`, `rec cancel` without closing your tab or losing your working directory.
- **2x speed 1080p60 MP4s by default** — renders crisp H.264 videos (Tokyo Night, Dracula, Catppuccin, Gruvbox, or Nord themes) ready to drop straight into YouTube, X/Twitter, TikTok, or Discord.
- **5-minute max clips** — long sessions automatically split into clean 5-minute chapters (`_part1.mp4`, `_part2.mp4`, ...).
- **Named after the process you ran** — strips `sudo`, `proxychains`, `rlwrap`, `cd`, `ls`, etc., so running `sudo nmap -sC -sV 10.10.11.42` saves `nmap_20261125-134012.mp4` (and if part 2 switches to `ffuf`, part 2 is named `ffuf_..._part2.mp4`).
- **Input + Output only (zero wasted storage)** — skips passive/output-only panes and automatically discards noisy background daemons like `openvpn`, `wg-quick`, `tail -f`, `watch`, `htop`, and `journalctl` so your HTB VPN tab never bloats your disk.
- **Terminal whitelist** — only auto-records in your chosen terminals (**Kitty & Tabby** by default; easily configurable to `alacritty`, `wezterm`, `ghostty`, `foot`, `konsole`, or `all`).
- **Zero duplicate bloat** — raw `.cast` files are automatically cleaned up once their MP4 finishes rendering.

---

## Installation

### Universal Installer (All Linux Distros)
Auto-detects `apt`, `pacman`, `dnf`, `yum`, `zypper`, `apk`, `xbps-install`, or `emerge`, installs dependencies + `agg`, and configures your shell:

```bash
git clone https://github.com/0xEleven/rec-cli.git
cd rec-cli
./scripts/install.sh
```

Options:
- `./scripts/install.sh --shell zsh` (or `bash`, `fish`, `all`)
- `./scripts/install.sh --system` (installs to `/usr/local`)
- `./scripts/install.sh --no-deps` (skip package manager installs)

### Debian / Ubuntu / Kali / Linux Mint / Pop!_OS (`.deb`)
Download the latest `.deb` from [Releases](https://github.com/0xEleven/rec-cli/releases):
```bash
sudo apt install ./rec-cli_1.0.0_all.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE (`.rpm`)
Download the latest `.rpm` from [Releases](https://github.com/0xEleven/rec-cli/releases):
```bash
sudo dnf install ./rec-cli-1.0.0-1.noarch.rpm
# or on openSUSE:
sudo zypper install ./rec-cli-1.0.0-1.noarch.rpm
```

### Arch Linux / Manjaro / EndeavourOS (`PKGBUILD`)
```bash
cd packaging/arch
makepkg -si
```

---

## Shell Setup

Add **one line** to your shell config (done automatically by `./scripts/install.sh`):

- **Zsh** (`~/.zshrc`):
  ```zsh
  eval "$(rec-cli init zsh)"
  ```
- **Bash** (`~/.bashrc`):
  ```bash
  eval "$(rec-cli init bash)"
  ```
- **Fish** (`~/.config/fish/config.fish`):
  ```fish
  rec-cli init fish | source
  ```

### Optional: Live Red `● REC` Prompt Badge (Starship)
Add `${custom.rec}` to your `format` in `~/.config/starship.toml` and paste [starship/rec-module.toml](starship/rec-module.toml).

---

## Usage

| Command | Description |
| --- | --- |
| `rec start [name]` | Start recording immediately (pass an optional custom name, or omit to auto-name by process) |
| `rec stop` | Stop recording, keep tab open in your current directory, and render 2x MP4 in background |
| `rec cancel` | Stop recording and discard the take completely (saves nothing) |
| `rec status` | Show live recording status, auto-record mode, active panes, and saved clip stats |
| `rec auto on\|off` | Toggle auto-recording on new tabs vs. manual-only (`rec start`) mode |
| `rec config` | View or change settings (`terminals`, `speed`, `max-secs`, `video-dir`, `theme`) |
| `rec list` *(or `recs`)* | List saved MP4 clips in `~/Videos/content` |
| `rec render [speed]` | Convert any pending finished `.cast` files immediately |
| `rec open` | Open `~/Videos/content` in your file manager |
| `rec clear [--yes]` | Clear saved MP4 clips and raw casts |
| `rec doctor` | Verify all dependencies (`asciinema`, `agg`, `ffmpeg`, `python3`, `flock`, `setsid`) |

---

## Configuration

Settings are stored in `~/.config/rec-cli/config` and can be updated anytime via `rec config`:

```bash
rec config terminals kitty,tabby          # or kitty,tabby,alacritty,wezterm,ghostty,all
rec config speed 2                        # default 2x speed
rec config max-secs 300                   # 5 minutes max per video
rec config video-dir "$HOME/Videos/content"
rec config theme tokyo-night              # tokyo-night | dracula | catppuccin | gruvbox | nord
```

---

## Building Packages Locally

```bash
make test       # run automated test suite
make deb        # builds dist/rec-cli_1.0.0_all.deb
make rpm        # builds dist/rec-cli-1.0.0-1.noarch.rpm
make tarball    # builds dist/rec-cli-1.0.0.tar.gz
```

## License

MIT — see [LICENSE](LICENSE).
