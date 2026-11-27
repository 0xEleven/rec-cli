use crate::config::{runtime_dir, short_path, which, Config};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SHELL_COMMS: &[&str] = &["zsh", "bash", "fish", "sh", "nu", "elvish", "tcsh", "ksh"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Recording,
    Paused,
    Standby,
}

#[derive(Debug, Clone)]
pub struct ManagedSession {
    pub pid: i32,
    pub child_pid: i32,
    pub state: SessionState,
    pub name: String,
    pub terminal: String,
    pub cast: String,
    pub cwd: String,
    pub fg_cmd: String,
    pub last_cmd: String,
    pub started_at: f64,
    pub cast_size: u64,
    pub legacy: bool,
}

impl ManagedSession {
    pub fn friendly_title(&self) -> String {
        if !self.name.trim().is_empty() && self.name != "old-tab" {
            return self.name.clone();
        }
        if !self.fg_cmd.trim().is_empty() {
            if let Some(first) = self.fg_cmd.split_whitespace().next() {
                let base = Path::new(first)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(first);
                if !base.is_empty() {
                    return base.to_string();
                }
            }
        }
        if !self.last_cmd.trim().is_empty() {
            if let Some(first) = self.last_cmd.split_whitespace().next() {
                return first.to_string();
            }
        }
        let dir = Path::new(&self.cwd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Terminal");
        format!("{} — {}", pretty_term(&self.terminal), dir)
    }

    pub fn subtitle(&self) -> String {
        let cwd = short_path(&self.cwd);
        let cmd = if !self.fg_cmd.is_empty() {
            self.fg_cmd.as_str()
        } else if !self.last_cmd.is_empty() {
            self.last_cmd.as_str()
        } else {
            ""
        };
        if cmd.is_empty() || cmd == cwd {
            cwd
        } else {
            format!("{}  ·  {}", truncate(cmd, 56), cwd)
        }
    }

    pub fn elapsed_str(&self) -> String {
        if self.state == SessionState::Standby || self.started_at <= 0.0 {
            return "Ready".to_string();
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let sec = (now - self.started_at).max(0.0) as u64;
        let h = sec / 3600;
        let m = (sec % 3600) / 60;
        let s = sec % 60;
        if h > 0 {
            format!("{h:02}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenTerminal {
    pub pid: i32,
    pub terminal: String,
    pub shell: String,
    pub cwd: String,
    pub fg_cmd: String,
    pub kitty_sock: String,
    pub kitty_win: String,
    pub tmux_pane: String,
}

impl OpenTerminal {
    pub fn can_remote_record(&self) -> bool {
        (!self.kitty_sock.is_empty() && !self.kitty_win.is_empty()) || !self.tmux_pane.is_empty()
    }

    pub fn friendly_title(&self) -> String {
        let dir = Path::new(&self.cwd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("~");
        format!("{} — {}", pretty_term(&self.terminal), dir)
    }

    pub fn subtitle(&self) -> String {
        let cwd = short_path(&self.cwd);
        if self.fg_cmd.is_empty() {
            format!("{}  ·  {}", self.shell, cwd)
        } else {
            format!("{}  ·  {}", truncate(&self.fg_cmd, 52), cwd)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscoverySnapshot {
    pub managed: Vec<ManagedSession>,
    pub open_unrecorded: Vec<OpenTerminal>,
    pub running_counts: HashMap<String, usize>,
    pub pending_casts: usize,
}

#[derive(Debug, Deserialize)]
struct SessionJson {
    #[serde(default)]
    pid: i32,
    #[serde(default)]
    child_pid: i32,
    #[serde(default)]
    state: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    terminal: String,
    #[serde(default)]
    cast: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    fg_cmd: String,
    #[serde(default)]
    last_cmd: String,
    #[serde(default)]
    started_at: f64,
    #[serde(default)]
    cast_size: u64,
}

struct ProcInfo {
    ppid: i32,
    tty_nr: i32,
    comm: String,
    parts: Vec<String>,
}

pub fn pretty_term(id: &str) -> String {
    match id.to_ascii_lowercase().as_str() {
        "kitty" => "Kitty".to_string(),
        "tabby" => "Tabby".to_string(),
        "alacritty" => "Alacritty".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "ghostty" => "Ghostty".to_string(),
        "foot" => "Foot".to_string(),
        "konsole" => "Konsole".to_string(),
        "gnome-terminal" => "GNOME Terminal".to_string(),
        "xfce4-terminal" => "XFCE Terminal".to_string(),
        "tilix" => "Tilix".to_string(),
        "terminator" => "Terminator".to_string(),
        "xterm" => "XTerm".to_string(),
        "" => "Terminal".to_string(),
        other => other.to_string(),
    }
}

pub fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let taken: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{taken}…")
    } else {
        taken
    }
}

pub fn fmt_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut val = bytes as f64;
    let mut idx = 0;
    while val >= 1024.0 && idx < UNITS.len() - 1 {
        val /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{bytes} B")
    } else {
        format!("{val:.1} {}", UNITS[idx])
    }
}

pub fn is_pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    unsafe { libc::kill(pid, 0) == 0 }
}

fn match_term_name(name: &str) -> Option<&'static str> {
    match name {
        "kitty" => Some("kitty"),
        "tabby" => Some("tabby"),
        "alacritty" => Some("alacritty"),
        "wezterm" | "wezterm-gui" => Some("wezterm"),
        "ghostty" => Some("ghostty"),
        "foot" => Some("foot"),
        "konsole" => Some("konsole"),
        "gnome-terminal" | "gnome-terminal-" | "gnome-terminal-server" | "kgx" => {
            Some("gnome-terminal")
        }
        "xfce4-terminal" => Some("xfce4-terminal"),
        "tilix" => Some("tilix"),
        "terminator" => Some("terminator"),
        "xterm" => Some("xterm"),
        "urxvt" => Some("urxvt"),
        "st" => Some("st"),
        _ => None,
    }
}

fn match_term_proc(info: &ProcInfo) -> Option<&'static str> {
    let comm = info.comm.to_ascii_lowercase();
    if let Some(t) = match_term_name(&comm) {
        return Some(t);
    }
    if let Some(first) = info.parts.first() {
        let base = Path::new(first)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if let Some(t) = match_term_name(&base) {
            return Some(t);
        }
        let joined = info
            .parts
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase();
        if joined.contains("tabby")
            && (base.contains("tabby") || base.contains("electron") || joined.contains("/tabby/"))
        {
            return Some("tabby");
        }
    }
    None
}

fn scan_proc_table() -> HashMap<i32, ProcInfo> {
    let uid = unsafe { libc::getuid() };
    let mut procs = HashMap::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return procs;
    };

    for entry in entries.flatten() {
        let fname = entry.file_name();
        let Some(name_str) = fname.to_str() else {
            continue;
        };
        let Ok(pid) = name_str.parse::<i32>() else {
            continue;
        };
        let path = entry.path();
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.uid() != uid {
            continue;
        }
        let Ok(stat) = fs::read_to_string(path.join("stat")) else {
            continue;
        };
        let (Some(lp), Some(rp)) = (stat.find('('), stat.rfind(')')) else {
            continue;
        };
        let comm = stat[lp + 1..rp].to_string();
        let mut fields = stat[rp + 2..].split_whitespace();
        let Some(state_ch) = fields.next() else {
            continue;
        };
        if state_ch == "Z" || state_ch == "X" {
            continue;
        }
        let ppid = fields.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let _pgrp = fields.next();
        let _session = fields.next();
        let tty_nr = fields.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

        let parts = fs::read(path.join("cmdline"))
            .map(|bytes| {
                bytes
                    .split(|&b| b == 0)
                    .filter(|s| !s.is_empty())
                    .map(|s| String::from_utf8_lossy(s).into_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        procs.insert(
            pid,
            ProcInfo {
                ppid,
                tty_nr,
                comm,
                parts,
            },
        );
    }
    procs
}

fn read_proc_cwd(pid: i32) -> String {
    fs::read_link(format!("/proc/{pid}/cwd"))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn read_proc_env(pid: i32, key: &str) -> String {
    let prefix = format!("{key}=");
    let Ok(bytes) = fs::read(format!("/proc/{pid}/environ")) else {
        return String::new();
    };
    for item in bytes.split(|&b| b == 0) {
        if let Ok(s) = std::str::from_utf8(item) {
            if let Some(val) = s.strip_prefix(&prefix) {
                return val.to_string();
            }
        }
    }
    String::new()
}

fn get_pts_path(pid: i32) -> Option<String> {
    let target = fs::read_link(format!("/proc/{pid}/fd/0")).ok()?;
    let s = target.to_string_lossy().into_owned();
    if s.starts_with("/dev/pts/") && !s.ends_with("(deleted)") {
        Some(s)
    } else {
        None
    }
}

fn detect_ancestor_terminal(pid: i32, procs: &HashMap<i32, ProcInfo>) -> Option<&'static str> {
    let mut cur = pid;
    for _ in 0..7 {
        let info = procs.get(&cur)?;
        if info.ppid <= 1 || info.ppid == cur {
            return None;
        }
        let pinfo = procs.get(&info.ppid)?;
        if let Some(t) = match_term_proc(pinfo) {
            return Some(t);
        }
        cur = info.ppid;
    }
    None
}

fn has_ancestor_in(pid: i32, covered: &HashSet<i32>, procs: &HashMap<i32, ProcInfo>) -> bool {
    let mut cur = pid;
    for _ in 0..8 {
        let Some(info) = procs.get(&cur) else {
            return false;
        };
        if covered.contains(&info.ppid) {
            return true;
        }
        if info.ppid <= 1 || info.ppid == cur {
            return false;
        }
        cur = info.ppid;
    }
    false
}

fn mark_shell_ancestors(pid: i32, covered: &mut HashSet<i32>, procs: &HashMap<i32, ProcInfo>) {
    let mut cur = pid;
    for _ in 0..3 {
        let Some(info) = procs.get(&cur) else {
            break;
        };
        let ppid = info.ppid;
        let Some(pinfo) = procs.get(&ppid) else {
            break;
        };
        if !SHELL_COMMS.contains(&pinfo.comm.as_str()) {
            break;
        }
        covered.insert(ppid);
        cur = ppid;
    }
}

fn deepest_fg_cmd(
    root_pid: i32,
    children_map: &HashMap<i32, Vec<i32>>,
    procs: &HashMap<i32, ProcInfo>,
) -> String {
    let mut cur = root_pid;
    let mut visited = HashSet::new();
    visited.insert(cur);
    loop {
        let next = children_map
            .get(&cur)
            .and_then(|kids| kids.iter().copied().filter(|k| !visited.contains(k)).max());
        match next {
            Some(k) => {
                cur = k;
                visited.insert(cur);
            }
            None => break,
        }
    }
    if cur == root_pid {
        return String::new();
    }
    let Some(info) = procs.get(&cur) else {
        return String::new();
    };
    if SHELL_COMMS.contains(&info.comm.as_str()) {
        return String::new();
    }
    if info.parts.len() == 1 {
        let base = Path::new(&info.parts[0])
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if SHELL_COMMS.contains(&base) {
            return String::new();
        }
    }
    if info.parts.is_empty() {
        info.comm.clone()
    } else {
        truncate(&info.parts.join(" "), 80)
    }
}

pub fn count_pending_casts(cast_dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(cast_dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".cast"))
                .unwrap_or(false)
        })
        .count()
}

pub fn discover_all_terminals(cast_dir: &Path) -> DiscoverySnapshot {
    let procs = scan_proc_table();
    let mut children_map: HashMap<i32, Vec<i32>> = HashMap::new();
    for (&pid, info) in &procs {
        children_map.entry(info.ppid).or_default().push(pid);
    }

    let rt = runtime_dir();
    let sess_dir = rt.join("sessions");
    let active_dir = rt.join("active");

    let mut managed = Vec::new();
    let mut covered_pids: HashSet<i32> = HashSet::new();
    let mut covered_pts: HashSet<String> = HashSet::new();
    let mut running_counts: HashMap<String, usize> = HashMap::new();

    // 1. Managed rec-pty sessions
    if let Ok(entries) = fs::read_dir(&sess_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(mut s) = serde_json::from_str::<SessionJson>(&raw) else {
                continue;
            };
            if s.pid <= 0 || !procs.contains_key(&s.pid) || !is_pid_alive(s.pid) {
                let _ = fs::remove_file(&path);
                let _ = fs::remove_file(active_dir.join(s.pid.to_string()));
                continue;
            }
            if !s.cast.is_empty() {
                if let Ok(meta) = fs::metadata(&s.cast) {
                    s.cast_size = meta.len();
                }
            }
            let state = match s.state.as_str() {
                "recording" => SessionState::Recording,
                "paused" => SessionState::Paused,
                _ => SessionState::Standby,
            };
            covered_pids.insert(s.pid);
            mark_shell_ancestors(s.pid, &mut covered_pids, &procs);
            if let Some(pts) = get_pts_path(s.pid) {
                covered_pts.insert(pts);
            }
            if s.child_pid > 0 {
                covered_pids.insert(s.child_pid);
                if let Some(pts) = get_pts_path(s.child_pid) {
                    covered_pts.insert(pts);
                }
            }
            let term_low = s.terminal.to_ascii_lowercase();
            if !term_low.is_empty() {
                *running_counts.entry(term_low).or_insert(0) += 1;
            }
            managed.push(ManagedSession {
                pid: s.pid,
                child_pid: s.child_pid,
                state,
                name: s.name,
                terminal: s.terminal,
                cast: s.cast,
                cwd: s.cwd,
                fg_cmd: s.fg_cmd,
                last_cmd: s.last_cmd,
                started_at: s.started_at,
                cast_size: s.cast_size,
                legacy: false,
            });
        }
    }

    // 2. Legacy asciinema rec sessions attached to an open terminal
    for (&pid, info) in &procs {
        if covered_pids.contains(&pid) || info.parts.is_empty() {
            continue;
        }
        let base0 = Path::new(&info.parts[0])
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let is_asciinema = base0.contains("asciinema")
            || info.parts.iter().take(2).any(|p| p.contains("asciinema"));
        let has_rec = info.parts.iter().skip(1).take(3).any(|p| p == "rec");
        if !(is_asciinema && has_rec) {
            continue;
        }
        let Some(term) = detect_ancestor_terminal(pid, &procs) else {
            continue;
        };
        let Some(pts) = get_pts_path(pid) else {
            continue;
        };
        let cast = info
            .parts
            .iter()
            .rev()
            .find(|p| p.ends_with(".cast"))
            .cloned()
            .unwrap_or_default();
        let (started_at, cast_size) = if !cast.is_empty() {
            fs::metadata(&cast)
                .map(|m| {
                    let ts = m
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    (ts, m.len())
                })
                .unwrap_or((0.0, 0))
        } else {
            (0.0, 0)
        };
        let child_shell = children_map
            .get(&pid)
            .and_then(|kids| {
                kids.iter()
                    .copied()
                    .find(|k| procs.get(k).map(|p| SHELL_COMMS.contains(&p.comm.as_str())).unwrap_or(false))
            })
            .unwrap_or(0);
        let target_pid = if child_shell > 0 { child_shell } else { pid };
        managed.push(ManagedSession {
            pid,
            child_pid: child_shell,
            state: SessionState::Recording,
            name: "Legacy Tab".to_string(),
            terminal: term.to_string(),
            cast,
            cwd: read_proc_cwd(target_pid),
            fg_cmd: deepest_fg_cmd(target_pid, &children_map, &procs),
            last_cmd: String::new(),
            started_at,
            cast_size,
            legacy: true,
        });
        covered_pids.insert(pid);
        covered_pts.insert(pts);
        mark_shell_ancestors(pid, &mut covered_pids, &procs);
        if child_shell > 0 {
            covered_pids.insert(child_shell);
            if let Some(cpts) = get_pts_path(child_shell) {
                covered_pts.insert(cpts);
            }
        }
        *running_counts.entry(term.to_string()).or_insert(0) += 1;
    }

    // 3. Currently open unrecorded terminal tabs
    let mut open_unrecorded = Vec::new();
    for (&pid, info) in &procs {
        if covered_pids.contains(&pid) {
            continue;
        }
        if !SHELL_COMMS.contains(&info.comm.as_str()) || info.comm == "sh" {
            continue;
        }
        if has_ancestor_in(pid, &covered_pids, &procs) {
            continue;
        }
        if info.tty_nr == 0 {
            continue;
        }
        let Some(pts) = get_pts_path(pid) else {
            continue;
        };
        if covered_pts.contains(&pts) {
            continue;
        }
        let Some(term) = detect_ancestor_terminal(pid, &procs) else {
            continue;
        };
        if let Some(parent) = procs.get(&info.ppid) {
            if SHELL_COMMS.contains(&parent.comm.as_str()) {
                continue;
            }
        }

        let kitty_sock = if term == "kitty" {
            read_proc_env(pid, "KITTY_LISTEN_ON")
        } else {
            String::new()
        };
        let kitty_win = if term == "kitty" {
            read_proc_env(pid, "KITTY_WINDOW_ID")
        } else {
            String::new()
        };
        let tmux_pane = read_proc_env(pid, "TMUX_PANE");

        open_unrecorded.push(OpenTerminal {
            pid,
            terminal: term.to_string(),
            shell: info.comm.clone(),
            cwd: read_proc_cwd(pid),
            fg_cmd: deepest_fg_cmd(pid, &children_map, &procs),
            kitty_sock,
            kitty_win,
            tmux_pane,
        });
        covered_pts.insert(pts);
        *running_counts.entry(term.to_string()).or_insert(0) += 1;
    }

    managed.sort_by_key(|s| {
        let rank = match s.state {
            SessionState::Recording => 0,
            SessionState::Paused => 1,
            SessionState::Standby => 2,
        };
        (rank, s.pid)
    });
    open_unrecorded.sort_by(|a, b| a.terminal.cmp(&b.terminal).then(a.pid.cmp(&b.pid)));

    let pending_casts = count_pending_casts(cast_dir);

    DiscoverySnapshot {
        managed,
        open_unrecorded,
        running_counts,
        pending_casts,
    }
}

pub fn send_ctrl_ack(pid: i32, cmd_line: &str, timeout: Duration) -> Option<serde_json::Value> {
    let sess_dir = runtime_dir().join("sessions");
    let _ = fs::create_dir_all(&sess_dir);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let req_id = format!("{}-{nanos}", std::process::id());
    let ack_file = sess_dir.join(format!("{pid}.ack.{req_id}"));
    let cmd_file = sess_dir.join(format!("{pid}.cmd"));

    let mut fp = OpenOptions::new().create(true).append(true).open(&cmd_file).ok()?;
    writeln!(fp, "REQ:{req_id} {}", cmd_line.trim()).ok()?;
    drop(fp);

    if unsafe { libc::kill(pid, libc::SIGUSR1) } != 0 {
        return None;
    }

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if ack_file.exists() {
            let parsed = fs::read_to_string(&ack_file)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
            let _ = fs::remove_file(&ack_file);
            return parsed;
        }
        if !is_pid_alive(pid) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}

pub fn send_ctrl_fire_and_forget(pid: i32, cmd_line: &str) -> bool {
    let sess_dir = runtime_dir().join("sessions");
    let _ = fs::create_dir_all(&sess_dir);
    let cmd_file = sess_dir.join(format!("{pid}.cmd"));
    let Ok(mut fp) = OpenOptions::new().create(true).append(true).open(&cmd_file) else {
        return false;
    };
    if writeln!(fp, "{}", cmd_line.trim()).is_err() {
        return false;
    }
    drop(fp);
    unsafe { libc::kill(pid, libc::SIGUSR1) == 0 }
}

pub fn terminate_session(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    unsafe { libc::kill(pid, libc::SIGTERM) == 0 }
}

pub fn render_cast_sync(cast_path: &str, manual: bool) -> Result<Vec<PathBuf>, String> {
    if cast_path.is_empty() || !Path::new(cast_path).exists() {
        return Ok(Vec::new());
    }
    let cast2mp4 = which("cast2mp4").ok_or_else(|| "cast2mp4 not found in PATH".to_string())?;
    let mut cmd = Command::new(cast2mp4);
    if !manual {
        cmd.arg("--auto");
    }
    cmd.args(["--cleanup", "--print-output", cast_path]);
    let out = cmd.output().map_err(|e| format!("Failed to run cast2mp4: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let saved: Vec<PathBuf> = stdout
        .lines()
        .filter_map(|ln| ln.strip_prefix("SAVED:"))
        .map(|s| PathBuf::from(s.trim()))
        .collect();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Video export failed".to_string()
        } else {
            stderr.lines().last().unwrap_or("Video export failed").to_string()
        });
    }
    Ok(saved)
}

pub fn launch_recorded_terminal(cfg: &Config, preferred: &str, clip_name: &str) -> Result<String, String> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut rec_args = vec!["rec-pty".to_string()];
    let clean_name = clip_name.trim();
    if !clean_name.is_empty() {
        rec_args.push("--name".to_string());
        rec_args.push(clean_name.to_string());
    }
    rec_args.push("--".to_string());
    rec_args.push(shell);

    let mut candidates: Vec<String> = Vec::new();
    if !preferred.is_empty() && preferred != "auto" {
        candidates.push(preferred.to_ascii_lowercase());
    }
    for t in cfg.terminals.split(',') {
        let t = t.trim().to_ascii_lowercase();
        if !t.is_empty() && t != "all" && t != "none" && !candidates.contains(&t) {
            candidates.push(t);
        }
    }
    for fallback in [
        "kitty",
        "alacritty",
        "wezterm",
        "ghostty",
        "foot",
        "konsole",
        "gnome-terminal",
        "tabby",
        "xterm",
    ] {
        if !candidates.iter().any(|c| c == fallback) {
            candidates.push(fallback.to_string());
        }
    }

    for term in candidates {
        let Some(bin) = which(&term) else {
            continue;
        };
        let mut cmd = Command::new(&bin);
        match term.as_str() {
            "kitty" | "alacritty" | "ghostty" | "konsole" | "xterm" => {
                cmd.arg("-e").args(&rec_args);
            }
            "foot" => {
                cmd.args(&rec_args);
            }
            "wezterm" => {
                cmd.arg("start").arg("--").args(&rec_args);
            }
            "gnome-terminal" => {
                cmd.arg("--").args(&rec_args);
            }
            "tabby" => {
                // Tabby does not accept -e; launches normally with shell standby hook
            }
            _ => continue,
        }
        // Strip inherited rec-pty vars so new windows never hijack an existing session
        for key in [
            "REC_PTY_PID",
            "REC_PTY_TTY",
            "REC_RUNTIME_DIR",
            "_TERM_REC_ACTIVE",
            "_TERM_REC_META",
            "_TERM_REC_MARKER",
            "ASCIINEMA_REC",
        ] {
            cmd.env_remove(key);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        if cmd.spawn().is_ok() {
            return Ok(pretty_term(&term));
        }
    }

    Err("No supported terminal app found. Open a terminal and run: rec start".to_string())
}

pub enum AttachOutcome {
    Started(String),
    CopyCommand,
}

pub fn attach_open_terminal(info: &OpenTerminal, clip_name: &str) -> AttachOutcome {
    let rec_cmd = if clip_name.trim().is_empty() {
        "rec start".to_string()
    } else {
        format!("rec start {}", clip_name.trim().replace(' ', "-"))
    };

    if !info.kitty_sock.is_empty() && !info.kitty_win.is_empty() {
        if let Some(kitty) = which("kitty") {
            let status = Command::new(kitty)
                .args([
                    "@",
                    "--to",
                    &info.kitty_sock,
                    "send-text",
                    "--match",
                    &format!("id:{}", info.kitty_win),
                    &format!("{rec_cmd}\r"),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if status.map(|s| s.success()).unwrap_or(false) {
                return AttachOutcome::Started(format!("Kitty #{}", info.kitty_win));
            }
        }
    }

    if !info.tmux_pane.is_empty() {
        if let Some(tmux) = which("tmux") {
            let status = Command::new(tmux)
                .args(["send-keys", "-t", &info.tmux_pane, &rec_cmd, "Enter"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if status.map(|s| s.success()).unwrap_or(false) {
                return AttachOutcome::Started(format!("tmux {}", info.tmux_pane));
            }
        }
    }

    AttachOutcome::CopyCommand
}
