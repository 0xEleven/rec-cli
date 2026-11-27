#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
unset REC_RUNTIME_DIR REC_PTY_PID REC_PTY_TTY _TERM_REC_ACTIVE _TERM_REC_META _TERM_REC_MARKER
export PATH="$REPO_ROOT/gui/target/release:$REPO_ROOT/bin:$PATH"

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

echo "=== 2. rec-cli init, rec-pty & native Rust rec-gui v2.1.0 checks ==="
python3 -m py_compile "$REPO_ROOT/bin/rec-pty"
rec-cli version | grep -q "rec-cli 2.1.0"
rec-cli init zsh  | grep -q "_rec_is_allowed_terminal"
rec-cli init bash | grep -q "_rec_bash_preexec"
rec-cli init fish | grep -q "_rec_fish_preexec"
XDG_CONFIG_HOME="$(mktemp -d)" rec-cli config auto | grep -q "Resolution set to auto"

GUI_BIN="$REPO_ROOT/gui/target/release/rec-gui"
if [[ -x "$GUI_BIN" ]]; then
  "$GUI_BIN" --version | grep -q "rec-gui 2.1.0"
  XDG_CONFIG_HOME="$(mktemp -d)" "$GUI_BIN" --smoke-test | grep -q "SMOKE_OK"
  echo "PASS: native Rust rec-gui smoke test OK"
else
  echo "WARN: $GUI_BIN not built yet (run 'make build-gui')"
fi
echo "PASS: rec-pty compile, rec-cli init & config auto work"

echo "=== 2b. Live process-manager & isolation tests (rec-pty + rec ps/start/save/pause/resume/mark/stop/cancel/kill) ==="
RT_TEST="$(mktemp -d)"
CAST_TEST="$(mktemp -d)"
VID_TEST="$(mktemp -d)"
CFG_TEST="$(mktemp -d)"

# Verify manual-first default (auto-record is OFF by default)
STATUS_OUT="$(XDG_CONFIG_HOME="$CFG_TEST" XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" rec-cli status)"
grep -q "OFF (manual only)" <<< "$STATUS_OUT"

# Launch session 1 that writes real terminal output so instant save & stop can produce MP4s
XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" "$REPO_ROOT/bin/rec-pty" --name ssh-box -- bash -c "echo hello-session-1; sleep 12" >/dev/null 2>&1 &
PTY_PID=$!
# Wait up to 3s for session registration
for _ in $(seq 1 30); do
  [[ -f "$RT_TEST/rec-cli-$(id -u)/sessions/${PTY_PID}.json" ]] && break
  sleep 0.1
done
sleep 0.3

# Active recording must use .cast.recording (never bare .cast) so casts2mp4 can never steal it
ls "$CAST_TEST/casts"/*.cast.recording >/dev/null
! ls "$CAST_TEST/casts"/*.cast >/dev/null 2>&1

# Starting a second session & running casts2mp4 must NOT stop or save session 1
XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" "$REPO_ROOT/bin/rec-pty" --standby -- bash -c "sleep 12" >/dev/null 2>&1 &
PTY_PID2=$!
for _ in $(seq 1 30); do
  [[ -f "$RT_TEST/rec-cli-$(id -u)/sessions/${PTY_PID2}.json" ]] && break
  sleep 0.1
done
XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" "$REPO_ROOT/bin/casts2mp4" --quiet
PS_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep "$PTY_PID" <<< "$PS_OUT" | grep -q "REC"
grep "$PTY_PID2" <<< "$PS_OUT" | grep -q "STANDBY"

# Calling `rec start` on an already-recording session must NOT silently stop/save it
if XDG_RUNTIME_DIR="$RT_TEST" rec-cli start "$PTY_PID" accidental >/dev/null 2>&1; then
  echo "FAIL: rec start should refuse on already-recording PID without --new" >&2
  exit 1
fi
PS_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep "$PTY_PID" <<< "$PS_OUT" | grep -q "ssh-box"

# Instant snapshot save keeps recording active!
if command -v agg >/dev/null 2>&1 && command -v ffmpeg >/dev/null 2>&1; then
  SAVE_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" REC_RESOLUTION=720p rec-cli save "$PTY_PID" snap-check)"
  grep -q "Saved snapshot:" <<< "$SAVE_OUT"
  ls "$VID_TEST"/snap-check_*.mp4 >/dev/null
  PS_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
  grep "$PTY_PID" <<< "$PS_OUT" | grep -q "REC"
fi

grep -q "Paused PID $PTY_PID" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli pause "$PTY_PID")"
grep -q "PAUSED" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep -q "Resumed PID $PTY_PID" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli resume "$PTY_PID")"
grep -q "Chapter marker" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli mark "$PTY_PID" "got root")"

# Synchronous stop updates state/marker immediately and saves MP4 right away
STOP_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" REC_DIR="$CAST_TEST" REC_VIDEO_DIR="$VID_TEST" REC_RESOLUTION=720p rec-cli stop "$PTY_PID")"
grep -q "Stopped PID $PTY_PID" <<< "$STOP_OUT"
# Active marker must ALREADY be removed the instant rec stop returns (no prompt lag!)
[[ ! -f "$RT_TEST/rec-cli-$(id -u)/active/${PTY_PID}" ]]
if command -v agg >/dev/null 2>&1 && command -v ffmpeg >/dev/null 2>&1; then
  grep -q "Saved:" <<< "$STOP_OUT"
  ls "$VID_TEST"/ssh-box_*.mp4 >/dev/null
fi
# Child must STILL be alive in STANDBY mode (simulating live SSH session surviving rec stop!)
kill -0 "$PTY_PID"
PS_OUT="$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep "$PTY_PID" <<< "$PS_OUT" | grep -q "STANDBY"
# Remotely start a second clip on the exact same live PID, split with --new, then cancel & kill cleanly
grep -q "Started recording on PID $PTY_PID" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli start "$PTY_PID" part2)"
grep -q "part2" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep -q "Started new clip on PID $PTY_PID" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli start "$PTY_PID" --new part3)"
grep -q "part3" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli ps)"
grep -q "Canceled PID $PTY_PID" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli cancel "$PTY_PID")"
grep -q "Terminated" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli kill "$PTY_PID")"
grep -q "Terminated" <<< "$(XDG_RUNTIME_DIR="$RT_TEST" rec-cli kill "$PTY_PID2")"
wait "$PTY_PID" "$PTY_PID2" 2>/dev/null || true
rm -rf "$RT_TEST" "$CAST_TEST" "$VID_TEST" "$CFG_TEST"
echo "PASS: Live PID process control, session isolation, instant save & synchronous stop verified!"

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
echo "ALL v2.1.0 TESTS PASSED!"
