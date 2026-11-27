use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct TerminalDef {
    pub id: &'static str,
    pub label: &'static str,
    pub bins: &'static [&'static str],
}

pub const TERMINALS: &[TerminalDef] = &[
    TerminalDef { id: "kitty", label: "Kitty", bins: &["kitty"] },
    TerminalDef { id: "tabby", label: "Tabby", bins: &["tabby"] },
    TerminalDef { id: "alacritty", label: "Alacritty", bins: &["alacritty"] },
    TerminalDef { id: "wezterm", label: "WezTerm", bins: &["wezterm", "wezterm-gui"] },
    TerminalDef { id: "ghostty", label: "Ghostty", bins: &["ghostty"] },
    TerminalDef { id: "foot", label: "Foot", bins: &["foot"] },
    TerminalDef { id: "konsole", label: "Konsole", bins: &["konsole"] },
    TerminalDef {
        id: "gnome-terminal",
        label: "GNOME Terminal",
        bins: &["gnome-terminal", "gnome-terminal-server", "kgx"],
    },
];

#[derive(Debug, Clone)]
pub struct Config {
    pub resolution: String,
    pub speed: String,
    pub max_secs: String,
    pub frame: bool,
    pub redact: bool,
    pub reels_copy: bool,
    pub video_theme: String,
    pub terminals: String,
    pub video_dir: PathBuf,
    pub cast_dir: PathBuf,
    pub ui_theme: String,
    pub auto_record: bool,
}

impl Config {
    pub fn load() -> Self {
        let home = home_dir();
        let mut map: HashMap<String, String> = HashMap::new();
        let conf_file = config_file_path();

        if let Ok(content) = fs::read_to_string(&conf_file) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), unquote_shell_val(v.trim()));
                }
            }
        }

        let get = |k: &str, default: &str| -> String {
            map.get(k)
                .cloned()
                .or_else(|| env::var(k).ok())
                .unwrap_or_else(|| default.to_string())
        };

        let parse_bool = |s: &str, default: bool| -> bool {
            let l = s.to_ascii_lowercase();
            match l.as_str() {
                "on" | "1" | "true" | "yes" | "enable" | "enabled" => true,
                "off" | "0" | "false" | "no" | "disable" | "disabled" => false,
                _ => default,
            }
        };

        let video_dir = PathBuf::from(get(
            "REC_VIDEO_DIR",
            &home.join("Videos/content").to_string_lossy(),
        ));
        let rec_dir = PathBuf::from(get(
            "REC_DIR",
            &home.join("Videos/terminal-recordings").to_string_lossy(),
        ));
        let cast_dir = rec_dir.join("casts");

        let auto_disabled = auto_off_flag_path().exists() || legacy_auto_off_path().exists();
        let auto_record = !auto_disabled && parse_bool(&get("REC_AUTO_RECORD", "off"), false);

        Self {
            resolution: get("REC_RESOLUTION", "auto"),
            speed: get("REC_SPEED", "2"),
            max_secs: get("REC_MAX_SECS", "300"),
            frame: parse_bool(&get("REC_FRAME", "on"), true),
            redact: parse_bool(&get("REC_REDACT", "on"), true),
            reels_copy: parse_bool(&get("REC_REELS_COPY", "off"), false),
            video_theme: get("REC_THEME", "tokyo-night"),
            terminals: get("REC_TERMINALS", "kitty,tabby"),
            video_dir,
            cast_dir,
            ui_theme: get("REC_UI_THEME", "dark"),
            auto_record,
        }
    }

    pub fn is_all_terminals(&self) -> bool {
        self.terminals
            .split(',')
            .any(|s| s.trim().eq_ignore_ascii_case("all"))
    }

    pub fn is_terminal_selected(&self, id: &str) -> bool {
        if self.is_all_terminals() {
            return true;
        }
        self.terminals
            .split(',')
            .any(|s| s.trim().eq_ignore_ascii_case(id))
    }
}

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn config_dir() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("rec-cli")
}

pub fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("rec-cli")
}

pub fn runtime_dir() -> PathBuf {
    if let Ok(d) = env::var("REC_RUNTIME_DIR") {
        if !d.is_empty() {
            return PathBuf::from(d);
        }
    }
    let base = env::var_os("XDG_RUNTIME_DIR")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let uid = unsafe { libc::getuid() };
    base.join(format!("rec-cli-{uid}"))
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("config")
}

pub fn auto_off_flag_path() -> PathBuf {
    config_dir().join("auto-disabled")
}

pub fn legacy_auto_off_path() -> PathBuf {
    home_dir().join(".config/term-rec/auto-disabled")
}

pub fn write_config_key(key: &str, val: &str) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let conf = config_file_path();

    let mut lines: Vec<String> = if conf.is_file() {
        fs::read_to_string(&conf)
            .unwrap_or_default()
            .lines()
            .filter(|ln| !ln.starts_with(&format!("{key}=")))
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };

    lines.push(format!("{key}={}", shell_quote(val)));
    let tmp = conf.with_extension("tmp");
    fs::write(&tmp, lines.join("\n") + "\n")?;
    fs::rename(&tmp, &conf)?;
    Ok(())
}

pub fn set_auto_enabled(enabled: bool) -> std::io::Result<()> {
    fs::create_dir_all(config_dir())?;
    if enabled {
        let _ = fs::remove_file(auto_off_flag_path());
        let _ = fs::remove_file(legacy_auto_off_path());
        write_config_key("REC_AUTO_RECORD", "on")?;
    } else {
        write_config_key("REC_AUTO_RECORD", "off")?;
        let _ = fs::write(auto_off_flag_path(), "");
    }
    Ok(())
}

fn unquote_shell_val(raw: &str) -> String {
    let s = raw.trim();
    if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        return s[1..s.len() - 1].replace("'\"'\"'", "'");
    }
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        return s[1..s.len() - 1].replace("\\\"", "\"");
    }
    if let Some(rest) = s.strip_prefix("$'") {
        if let Some(inner) = rest.strip_suffix('\'') {
            return inner.to_string();
        }
    }
    s.replace("\\ ", " ")
}

fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/' | b',' | b':')
        })
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

pub fn which(bin: &str) -> Option<PathBuf> {
    if bin.contains('/') {
        let p = PathBuf::from(bin);
        if is_executable(&p) {
            return Some(p);
        }
        return None;
    }
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let full = dir.join(bin);
        if is_executable(&full) {
            return Some(full);
        }
    }
    None
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(p)
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
        .unwrap_or(false)
}

pub fn installed_terminals() -> Vec<&'static TerminalDef> {
    TERMINALS
        .iter()
        .filter(|t| t.bins.iter().any(|b| which(b).is_some()))
        .collect()
}

pub fn short_path(path: &str) -> String {
    if path.is_empty() {
        return "~".to_string();
    }
    let home = home_dir().to_string_lossy().to_string();
    if path == home {
        "~".to_string()
    } else if let Some(rest) = path.strip_prefix(&(home + "/")) {
        format!("~/{rest}")
    } else {
        path.to_string()
    }
}

pub fn detect_display_preset() -> String {
    // Fast native Linux DRM sysfs scan first, then xrandr fallback
    let mut best_area: u64 = 0;
    let mut best_dims: Option<(u32, u32)> = None;
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let status = fs::read_to_string(entry.path().join("status")).unwrap_or_default();
            if status.trim() != "connected" {
                continue;
            }
            if let Ok(modes) = fs::read_to_string(entry.path().join("modes")) {
                if let Some(first) = modes.lines().next() {
                    if let Some((w, h)) = parse_dims(first) {
                        let area = (w as u64) * (h as u64);
                        if area > best_area {
                            best_area = area;
                            best_dims = Some((w, h));
                        }
                    }
                }
            }
        }
    }

    if best_dims.is_none() {
        if let Ok(out) = Command::new("xrandr").arg("--current").output() {
            let txt = String::from_utf8_lossy(&out.stdout);
            for line in txt.lines() {
                if line.contains('*') {
                    if let Some(tok) = line.split_whitespace().next() {
                        if let Some((w, h)) = parse_dims(tok) {
                            best_dims = Some((w, h));
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Some((w, h)) = best_dims {
        let preset = if w >= 3840 || h >= 2160 {
            "4K"
        } else if w >= 2560 || h >= 1440 {
            "1440p"
        } else if w >= 1920 || h >= 1080 {
            "1080p"
        } else {
            "720p"
        };
        format!("{w}×{h} · {preset}")
    } else {
        "1080p".to_string()
    }
}

fn parse_dims(s: &str) -> Option<(u32, u32)> {
    let clean = s.split(|c: char| !c.is_ascii_digit() && c != 'x').next()?;
    let (ws, hs) = clean.split_once('x')?;
    let w: u32 = ws.parse().ok()?;
    let h: u32 = hs.parse().ok()?;
    if w > 0 && h > 0 {
        Some((w, h))
    } else {
        None
    }
}

pub fn pick_directory_dialog(initial: &Path) -> Option<PathBuf> {
    if let Some(zenity) = which("zenity") {
        let out = Command::new(zenity)
            .args([
                "--file-selection",
                "--directory",
                "--title=Choose Clips Output Folder",
                &format!("--filename={}/", initial.display()),
            ])
            .output()
            .ok()?;
        if out.status.success() {
            let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
        return None;
    }
    if let Some(kdialog) = which("kdialog") {
        let out = Command::new(kdialog)
            .args(["--getexistingdirectory", &initial.to_string_lossy()])
            .output()
            .ok()?;
        if out.status.success() {
            let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub has_rec_pty: bool,
    pub has_ffmpeg: bool,
    pub has_agg: bool,
    pub shell_configured: bool,
    pub detected_shell: String,
}

impl SystemHealth {
    pub fn check() -> Self {
        let home = home_dir();
        let shell_env = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let shell_name = Path::new(&shell_env)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("bash")
            .to_string();

        let rc_files = [
            home.join(".zshrc"),
            home.join(".bashrc"),
            home.join(".config/fish/config.fish"),
        ];
        let shell_configured = rc_files.iter().any(|p| {
            fs::read_to_string(p)
                .map(|c| c.contains("rec-cli init"))
                .unwrap_or(false)
        });

        Self {
            has_rec_pty: which("rec-pty").is_some(),
            has_ffmpeg: which("ffmpeg").is_some(),
            has_agg: which("agg").is_some(),
            shell_configured,
            detected_shell: shell_name,
        }
    }
}

pub fn enable_shell_integration() -> Result<String, String> {
    let home = home_dir();
    let shell_env = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let shell_name = Path::new(&shell_env)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("bash");

    let (rc_path, init_line) = match shell_name {
        "zsh" => (home.join(".zshrc"), "eval \"$(rec-cli init zsh)\""),
        "fish" => (
            home.join(".config/fish/config.fish"),
            "rec-cli init fish | source",
        ),
        _ => (home.join(".bashrc"), "eval \"$(rec-cli init bash)\""),
    };

    if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Could not create config dir: {e}"))?;
    }
    let existing = fs::read_to_string(&rc_path).unwrap_or_default();
    if !existing.contains(init_line) {
        let mut updated = existing;
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str("\n# rec-cli terminal clip recorder\n");
        updated.push_str(init_line);
        updated.push('\n');
        fs::write(&rc_path, updated).map_err(|e| format!("Could not write {}: {e}", rc_path.display()))?;
    }
    Ok(short_path(&rc_path.to_string_lossy()))
}
