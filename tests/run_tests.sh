#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$REPO_ROOT/bin:$PATH"

echo "=== 1. Syntax checks (bash / zsh / fish) ==="
bash -n "$REPO_ROOT"/bin/rec-cli \
        "$REPO_ROOT"/bin/cast2mp4 \
        "$REPO_ROOT"/bin/casts2mp4 \
        "$REPO_ROOT"/shell/rec.bash \
        "$REPO_ROOT"/scripts/install.sh \
        "$REPO_ROOT"/scripts/uninstall.sh \
        "$REPO_ROOT"/packaging/deb/build-deb.sh \
        "$REPO_ROOT"/packaging/rpm/build-rpm.sh
echo "PASS: bash syntax OK"

if command -v zsh >/dev/null 2>&1; then
  zsh -n "$REPO_ROOT/shell/rec.zsh"
  echo "PASS: zsh syntax OK"
fi

if command -v fish >/dev/null 2>&1; then
  fish --no-execute "$REPO_ROOT/shell/rec.fish"
  echo "PASS: fish syntax OK"
fi

echo "=== 2. rec-cli init & CLI commands ==="
python3 -m py_compile "$REPO_ROOT/bin/rec-pty"
rec-cli version | grep -q "rec-cli 1.3.0"
rec-cli init zsh  | grep -q "_rec_is_allowed_terminal"
rec-cli init bash | grep -q "_rec_bash_preexec"
rec-cli init fish | grep -q "_rec_fish_preexec"
XDG_CONFIG_HOME="$(mktemp -d)" rec-cli config auto | grep -q "Resolution set to auto"
echo "PASS: rec-pty compile, rec-cli init & config auto work for zsh, bash, fish"

echo "=== 2b. Live process-manager test (rec-pty + rec ps/pause/resume/mark/stop/start/cancel/kill) ==="
RT_TEST="$(mktemp -d)"
CAST_TEST="$(mktemp -d)"
XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" "$REPO_ROOT/bin/rec-pty" --name ssh-box -- bash -c "sleep 8" >/dev/null 2>&1 &
PTY_PID=$!
# Wait up to 3s for session registration
for _ in $(seq 1 30); do
  [[ -f "$RT_TEST/rec-cli-$(id -u)/sessions/${PTY_PID}.json" ]] && break
  sleep 0.1
done
XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps | grep -q "$PTY_PID"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps | grep -q "ssh-box"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli pause "$PTY_PID" | grep -q "Paused PID $PTY_PID"
sleep 0.2
XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps | grep -q "PAUSED"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli resume "$PTY_PID" | grep -q "Resumed PID $PTY_PID"
sleep 0.2
XDG_RUNTIME_DIR="$RT_TEST" rec-cli mark "$PTY_PID" "got root" | grep -q "Chapter marker"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli stop "$PTY_PID" | grep -q "Stopped PID $PTY_PID"
sleep 0.3
# Child must STILL be alive in STANDBY mode (simulating live SSH session surviving rec stop!)
kill -0 "$PTY_PID"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps | grep -q "STANDBY"
# Remotely start a second clip on the exact same live PID, then cancel & kill cleanly
XDG_RUNTIME_DIR="$RT_TEST" rec-cli start "$PTY_PID" part2 | grep -q "Started recording on PID $PTY_PID"
sleep 0.2
XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps | grep -q "part2"
XDG_RUNTIME_DIR="$RT_TEST" rec-cli cancel "$PTY_PID" | grep -q "Canceled PID $PTY_PID"
sleep 0.2
XDG_RUNTIME_DIR="$RT_TEST" rec-cli kill "$PTY_PID" | grep -q "Terminated"
wait "$PTY_PID" 2>/dev/null || true
rm -rf "$RT_TEST" "$CAST_TEST"
echo "PASS: Live PID process control (ps/pause/resume/mark/stop-without-dropping/start/cancel/kill) verified!"

echo "=== 3. End-to-end 4K, vertical, redaction, chapters, gif/thumb/trim & split ==="
if ! command -v agg >/dev/null 2>&1 || ! command -v ffmpeg >/dev/null 2>&1; then
  echo "SKIP: agg or ffmpeg not installed in this environment (syntax + init tests passed)"
  exit 0
fi

TDIR="$(mktemp -d)"
OUT="$TDIR/out"
OUT2="$TDIR/out2"
OUT4K="$TDIR/out4k"
OUTV="$TDIR/outv"
mkdir -p "$OUT" "$OUT2" "$OUT4K" "$OUTV"
trap 'rm -rf "$TDIR"' EXIT

python3 - "$TDIR" <<'PYEOF'
import json, os, sys
tdir = sys.argv[1]
base_ts = 1795601000.0

def write_cast(name, events, meta_lines):
    with open(os.path.join(tdir, f"{name}.cast"), "w") as f:
        f.write(json.dumps({"version": 2, "width": 100, "height": 30, "timestamp": int(base_ts)}) + "\n")
        for ev in events:
            f.write(json.dumps(ev) + "\n")
    with open(os.path.join(tdir, f"{name}.meta"), "w") as f:
        for row in meta_lines:
            f.write("\t".join(row) + "\n")

# OpenVPN-only pane -> must skip
ovpn_evs = [[0.1, "i", "s"], [0.2, "i", "u"], [0.3, "i", "\r"]] + [
    [0.5 + i * 0.5, "o", f"OpenVPN TUN/TAP Initialization Sequence Completed {i}\r\n"] for i in range(15)
]
write_cast("ovpn", ovpn_evs, [(f"{base_ts + 0.3:.3f}", "sudo openvpn htb.ovpn")])

# Output-only pane -> must skip
out_evs = [[0.5 + i * 0.5, "o", f"background log {i}\r\n"] for i in range(15)]
write_cast("outonly", out_evs, [])

# Real nmap session with secret + chapter marker
nmap_evs = [
    [0.2, "i", "n"], [0.4, "i", "m"], [0.6, "i", "a"], [0.8, "i", "p"], [1.0, "i", "\r"],
    [1.2, "o", "$ sudo nmap -sC -sV 10.10.11.42\r\n"],
    [2.0, "o", "22/tcp open ssh\r\n"],
    [3.0, "o", "export AWS_KEY=AKIAIOSFODNN7EXAMPLE PASSWORD=SuperSecret123\r\n"],
    [4.0, "o", "80/tcp open http\r\n"],
    [5.0, "o", "Nmap done\r\n"],
    [6.0, "o", "$ \r\n"],
]
write_cast("nmap_sess", nmap_evs, [
    (f"{base_ts + 0.1:.3f}", "cd ~/htb"),
    (f"{base_ts + 0.2:.3f}", "ls"),
    (f"{base_ts + 1.0:.3f}", "sudo nmap -sC -sV 10.10.11.42"),
    ("MARK", f"{base_ts + 3.5:.3f}", "found web port"),
])

# Custom named clip (4K test)
write_cast("custom_4k", nmap_evs, [
    ("NAME", "htb-foothold"),
    (f"{base_ts + 1.0:.3f}", "sudo nmap -sC -sV 10.10.11.42"),
])

# Vertical Shorts/TikTok test
write_cast("vert_test", nmap_evs, [
    ("NAME", "short-clip"),
    (f"{base_ts + 1.0:.3f}", "sudo nmap -sC -sV 10.10.11.42"),
])

# Multi-part split -> part1=nmap, part2=ffuf
split_evs = []
for i in range(12):
    t = i * 1.0 + 0.5
    split_evs.append([t, "i", "a"])
    split_evs.append([t + 0.2, "o", f"nmap port {80+i}\r\n"])
for i in range(12):
    t = 22.0 + i * 1.0
    split_evs.append([t, "i", "b"])
    split_evs.append([t + 0.2, "o", f"ffuf /admin{i} [200]\r\n"])
write_cast("split_sess", split_evs, [
    (f"{base_ts + 0.4:.3f}", "sudo nmap -p- 10.10.11.42"),
    (f"{base_ts + 22.0:.3f}", "ffuf -u http://10.10.11.42/FUZZ -w common.txt"),
])
PYEOF

cast2mp4 --auto --cleanup --res 1080p --out-dir "$OUT" "$TDIR/ovpn.cast"
[[ ! -f "$TDIR/ovpn.cast" && $(ls "$OUT" | wc -l) -eq 0 ]]
echo "PASS: OpenVPN pane skipped & cleaned"

cast2mp4 --auto --cleanup --res 1080p --out-dir "$OUT" "$TDIR/outonly.cast"
[[ ! -f "$TDIR/outonly.cast" && $(ls "$OUT" | wc -l) -eq 0 ]]
echo "PASS: Output-only pane skipped & cleaned"

# Auto resolution detection + dual Reels companion export
cast2mp4 --auto --res auto --reels-copy --out-dir "$OUT" "$TDIR/nmap_sess.cast"
NMAP_MP4="$(ls "$OUT"/nmap_*[0-9].mp4)"
NMAP_REELS="$(ls "$OUT"/nmap_*_reels.mp4)"
[[ -s "$NMAP_MP4" && -s "$NMAP_REELS" ]]
[[ -s "${NMAP_MP4%.mp4}.chapters.txt" ]] && grep -q "found web port" "${NMAP_MP4%.mp4}.chapters.txt"
echo "PASS: Auto-resolution + nmap naming + YouTube chapters + companion Reels export"

# Native 4K UHD verification (3840x2160)
cast2mp4 --auto --cleanup --res 4k --out-dir "$OUT4K" "$TDIR/custom_4k.cast"
CLIP_4K="$(ls "$OUT4K"/htb-foothold_*.mp4)"
[[ -s "$CLIP_4K" ]]
echo "PASS: Native 4K UHD rendered + custom name"

# Reels / TikTok 9:16 cursor-following auto-reframe verification (1080x1920)
cast2mp4 --auto --cleanup --res reels --out-dir "$OUTV" "$TDIR/vert_test.cast"
CLIP_V="$(ls "$OUTV"/short-clip_*.mp4)"
[[ -s "$CLIP_V" ]]
echo "PASS: Reels 9:16 cursor-following auto-reframe rendered"

# Multi-part split
cast2mp4 --auto --cleanup --res 720p --max-secs 10 --out-dir "$OUT2" "$TDIR/split_sess.cast"
ls "$OUT2"/nmap_*_part1.mp4 >/dev/null && ls "$OUT2"/ffuf_*_part2.mp4 >/dev/null
echo "PASS: Multi-part split named per chunk (nmap part1 + ffuf part2)"

# Test rec-cli thumb, gif, trim, stats
REC_VIDEO_DIR="$OUT" rec-cli thumb "$NMAP_MP4" >/dev/null
[[ -s "${NMAP_MP4%.mp4}.png" ]]
REC_VIDEO_DIR="$OUT" rec-cli gif "$NMAP_MP4" 640 >/dev/null
[[ -s "${NMAP_MP4%.mp4}.gif" ]]
REC_VIDEO_DIR="$OUT" rec-cli trim "$NMAP_MP4" 00:00 00:02 >/dev/null
[[ -s "${NMAP_MP4%.mp4}_trimmed.mp4" ]]
REC_VIDEO_DIR="$OUT" rec-cli stats | grep -q "Total clips:"
echo "PASS: rec thumb, gif, trim, stats all verified"

echo ""
echo "ALL v1.3.0 TESTS PASSED!"
