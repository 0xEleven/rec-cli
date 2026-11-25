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
rec-cli version | grep -q "rec-cli "
rec-cli init zsh  | grep -q "_rec_is_allowed_terminal"
rec-cli init bash | grep -q "_rec_bash_preexec"
rec-cli init fish | grep -q "_rec_fish_preexec"
echo "PASS: rec-cli init works for zsh, bash, fish"

echo "=== 3. End-to-end cast2mp4 filtering, naming & 5-min split ==="
if ! command -v agg >/dev/null 2>&1 || ! command -v ffmpeg >/dev/null 2>&1; then
  echo "SKIP: agg or ffmpeg not installed in this environment (syntax + init tests passed)"
  exit 0
fi

TDIR="$(mktemp -d)"
OUT="$TDIR/out"
OUT2="$TDIR/out2"
mkdir -p "$OUT" "$OUT2"
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
        for k, v in meta_lines:
            f.write(f"{k}\t{v}\n")

# OpenVPN-only pane -> must skip
ovpn_evs = [[0.1, "i", "s"], [0.2, "i", "u"], [0.3, "i", "\r"]] + [
    [0.5 + i * 0.5, "o", f"OpenVPN TUN/TAP Initialization Sequence Completed {i}\r\n"] for i in range(15)
]
write_cast("ovpn", ovpn_evs, [(f"{base_ts + 0.3:.3f}", "sudo openvpn htb.ovpn")])

# Output-only pane -> must skip
out_evs = [[0.5 + i * 0.5, "o", f"background log {i}\r\n"] for i in range(15)]
write_cast("outonly", out_evs, [])

# Real nmap session -> must name nmap_*.mp4
nmap_evs = [
    [0.2, "i", "n"], [0.4, "i", "m"], [0.6, "i", "a"], [0.8, "i", "p"], [1.0, "i", "\r"],
    [1.2, "o", "$ sudo nmap -sC -sV 10.10.11.42\r\n"],
    [2.0, "o", "22/tcp open ssh\r\n"],
    [3.0, "o", "80/tcp open http\r\n"],
    [4.0, "o", "Nmap done\r\n"],
    [5.0, "o", "$ \r\n"],
]
write_cast("nmap_sess", nmap_evs, [
    (f"{base_ts + 0.1:.3f}", "cd ~/htb"),
    (f"{base_ts + 0.2:.3f}", "ls"),
    (f"{base_ts + 1.0:.3f}", "sudo nmap -sC -sV 10.10.11.42"),
])

# Custom named clip -> must name htb-foothold_*.mp4
write_cast("custom_sess", nmap_evs, [
    ("NAME", "htb-foothold"),
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

cast2mp4 --auto --cleanup --out-dir "$OUT" "$TDIR/ovpn.cast"
[[ ! -f "$TDIR/ovpn.cast" && $(ls "$OUT" | wc -l) -eq 0 ]]
echo "PASS: OpenVPN pane skipped & cleaned"

cast2mp4 --auto --cleanup --out-dir "$OUT" "$TDIR/outonly.cast"
[[ ! -f "$TDIR/outonly.cast" && $(ls "$OUT" | wc -l) -eq 0 ]]
echo "PASS: Output-only pane skipped & cleaned"

cast2mp4 --auto --cleanup --out-dir "$OUT" "$TDIR/nmap_sess.cast"
ls "$OUT"/nmap_*.mp4 >/dev/null
echo "PASS: Auto-named after nmap"

cast2mp4 --auto --cleanup --out-dir "$OUT" "$TDIR/custom_sess.cast"
ls "$OUT"/htb-foothold_*.mp4 >/dev/null
echo "PASS: Custom clip name respected"

cast2mp4 --auto --cleanup --max-secs 10 --out-dir "$OUT2" "$TDIR/split_sess.cast"
ls "$OUT2"/nmap_*_part1.mp4 >/dev/null && ls "$OUT2"/ffuf_*_part2.mp4 >/dev/null
echo "PASS: Multi-part split named per chunk (nmap part1 + ffuf part2)"

echo ""
echo "ALL TESTS PASSED!"
