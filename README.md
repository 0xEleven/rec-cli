# rec-cli

[![CI](https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/0xEleven/rec-cli/actions/workflows/ci.yml)
[![Release](https://github.com/0xEleven/rec-cli/actions/workflows/release.yml/badge.svg)](https://github.com/0xEleven/rec-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-bb9af7.svg)](LICENSE)

**Smart up-to-4K terminal clip recorder CLI for Linux — built for content creators, pentesters, and developers.**

Supports **Zsh, Bash, and Fish** across **all major Linux distributions** (Debian, Ubuntu, Kali, Mint, Pop!_OS, Arch, Manjaro, EndeavourOS, Fedora, RHEL, Rocky, Alma, openSUSE, Alpine, Void, Gentoo).

---

## Features

- **Up to 4K UHD (3840x2160) & Vertical 9:16 (1080x1920)** — native high-DPI font rendering at `720p`, `1080p`, `1440p`, **`4k`** (default), or **`vertical`** (TikTok / YouTube Shorts / Reels) at 60fps H.264.
- **Sleek macOS-Style Window Frame** — rounded terminal card, drop shadow, traffic-light dots, theme-matched gradient backdrop, and centered tool titlebar (`rec config frame on|off`).
- **Automatic Secret Redaction** — scrubs AWS keys, GitHub PATs, API keys (`sk-...`), JWTs, `Bearer` tokens, private keys, and `PASSWORD=`/`SECRET=` leaks *before* rendering (`rec config redact on|off`).
- **Start / Stop / Cancel on demand** — `rec start [name]`, `rec stop`, `rec cancel` without closing your tab or losing `$PWD`.
- **YouTube Chapter Markers** — drop markers mid-session with `rec mark "privesc"` to export a ready-to-paste `.chapters.txt` alongside your MP4.
- **2x speed & 5-minute max clips** — long sessions automatically split into 5-minute chapters (`_part1.mp4`, `_part2.mp4`, ...).
- **Named after the process you ran** — strips `sudo`, `proxychains`, `rlwrap`, `cd`, `ls`, etc., so `sudo nmap -sC -sV 10.10.11.42` saves `nmap_20261125-134012.mp4`.
- **Input + Output only & OpenVPN skip** — discards passive output-only tabs and background daemons (`openvpn`, `wg-quick`, `tail -f`, `watch`, `htop`, `journalctl`) so your HTB VPN tab never wastes storage.
- **Built-in Content Toolkit** — `rec play`, `rec gif`, `rec thumb`, `rec trim`, and `rec stats` right from the terminal.
- **Terminal whitelist** — only auto-records in chosen terminals (**Kitty & Tabby** by default; configurable to `alacritty`, `wezterm`, `ghostty`, `foot`, `konsole`, or `all`).

---

## Installation

### Universal Installer (All Linux Distros)
Auto-detects `apt`, `pacman`, `dnf`, `yum`, `zypper`, `apk`, `xbps-install`, or `emerge`, installs dependencies + `agg`, and configures your shell:

```bash
git clone https://github.com/0xEleven/rec-cli.git
cd rec-cli
./scripts/install.sh
```

### Debian / Ubuntu / Kali / Linux Mint / Pop!_OS (`.deb`)
Download from [Releases](https://github.com/0xEleven/rec-cli/releases):
```bash
sudo apt install ./rec-cli_1.1.0_all.deb
```

### Fedora / RHEL / Rocky / Alma / openSUSE (`.rpm`)
Download from [Releases](https://github.com/0xEleven/rec-cli/releases):
```bash
sudo dnf install ./rec-cli-1.1.0-1.noarch.rpm
# openSUSE:
sudo zypper install ./rec-cli-1.1.0-1.noarch.rpm
```

### Arch Linux / Manjaro / EndeavourOS (`PKGBUILD`)
```bash
cd packaging/arch && makepkg -si
```

---

## Shell Setup

Add **one line** to your shell rc (done automatically by `./scripts/install.sh`):

- **Zsh** (`~/.zshrc`): `eval "$(rec-cli init zsh)"`
- **Bash** (`~/.bashrc`): `eval "$(rec-cli init bash)"`
- **Fish** (`~/.config/fish/config.fish`): `rec-cli init fish | source`

### Optional: Live Red `● REC` Prompt Badge (Starship)
Add `${custom.rec}` to your `format` in `~/.config/starship.toml` and paste [starship/rec-module.toml](starship/rec-module.toml).

---

## Usage

| Command | Description |
| --- | --- |
| `rec start [name]` | Start recording immediately (custom name optional; auto-named by process otherwise) |
| `rec stop` | Stop recording, keep tab open in `$PWD`, and render MP4 in background |
| `rec cancel` | Stop recording and discard the take completely |
| `rec mark <label>` | Drop a YouTube chapter marker (exports `.chapters.txt` alongside MP4) |
| `rec status` | Show live recording state, config & clip stats |
| `rec list` *(or `recs`)* | List saved MP4 clips in `~/Videos/content` |
| `rec play [clip\|latest]` | Preview a clip in `mpv` / `vlc` / default player |
| `rec gif [clip\|latest]` | Export an optimized palette GIF for READMEs / Discord / X |
| `rec thumb [clip\|latest]` | Extract a crisp PNG thumbnail frame |
| `rec trim <clip> <start> <end>` | Fast lossless trim (`rec trim latest 00:05 01:20`) |
| `rec stats` | Show clip count, total runtime, disk usage & top tools |
| `rec auto on\|off` | Toggle auto-recording on new tabs vs. manual-only mode |
| `rec config [key val]` | Configure `resolution`, `speed`, `max-secs`, `frame`, `redact`, `theme`, `terminals`, `video-dir` |
| `rec doctor` | Verify dependencies (`asciinema`, `agg`, `ffmpeg`, `python3`, `python3-pil`, `flock`, `setsid`) |

---

## Configuration Examples

```bash
rec config resolution 4k                  # 720p | 1080p | 1440p | 4k | vertical
rec config speed 2                        # 2x fast-typing playback
rec config max-secs 300                   # 5 minutes max per video
rec config frame on                       # sleek rounded window frame + titlebar
rec config redact on                      # scrub API keys / JWTs / passwords
rec config theme tokyo-night              # tokyo-night | dracula | catppuccin | gruvbox | nord
rec config terminals kitty,tabby          # or alacritty,wezterm,ghostty,all
```

---

## Building Packages Locally

```bash
make test       # run full automated test suite (4K, vertical, redaction, chapters, split)
make packages   # builds dist/.deb, dist/.rpm, dist/.tar.gz
```

## License

MIT — see [LICENSE](LICENSE).
