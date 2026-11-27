use crate::config::{cache_dir, which};
use chrono::{DateTime, Local};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct ClipItem {
    pub path: PathBuf,
    pub filename: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub mtime_secs: i64,
    pub date_label: String,
    pub is_vertical: bool,
    pub has_chapters: bool,
    pub has_gif: bool,
    pub has_png: bool,
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumb_path: Option<PathBuf>,
}

impl ClipItem {
    pub fn duration_label(&self) -> String {
        match self.duration_secs {
            Some(d) if d > 0.0 => {
                let sec = d.round() as u64;
                let m = sec / 60;
                let s = sec % 60;
                format!("{m}:{s:02}")
            }
            _ => String::new(),
        }
    }

    pub fn resolution_badge(&self) -> String {
        match (self.width, self.height) {
            (Some(w), Some(h)) => {
                if h > w {
                    if h >= 3000 {
                        "9:16 4K".to_string()
                    } else {
                        "9:16".to_string()
                    }
                } else if w >= 3800 || h >= 2100 {
                    "4K".to_string()
                } else if w >= 2500 || h >= 1400 {
                    "1440p".to_string()
                } else if w >= 1900 || h >= 1000 {
                    "1080p".to_string()
                } else {
                    "720p".to_string()
                }
            }
            _ => {
                if self.is_vertical {
                    "9:16".to_string()
                } else {
                    String::new()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipProbeResult {
    pub path: PathBuf,
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumb_path: Option<PathBuf>,
}

pub fn scan_clips(video_dir: &Path) -> Vec<ClipItem> {
    let _ = fs::create_dir_all(video_dir);
    let Ok(entries) = fs::read_dir(video_dir) else {
        return Vec::new();
    };

    let thumb_dir = cache_dir().join("thumbs");
    let mut clips = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("mp4") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if filename.is_empty() || filename.contains(".tmp.") {
            continue;
        }
        let size_bytes = meta.len();
        let mtime_secs = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&filename);
        let is_vertical = stem.ends_with("_reels");
        let display_name = prettify_clip_name(stem);
        let has_chapters = path.with_extension("chapters.txt").is_file();
        let has_gif = path.with_extension("gif").is_file();
        let has_png = path.with_extension("png").is_file();

        let cached_thumb = thumb_cache_path(&thumb_dir, &filename, size_bytes, mtime_secs);
        let thumb_path = if cached_thumb.is_file() {
            Some(cached_thumb)
        } else {
            None
        };

        clips.push(ClipItem {
            path,
            display_name,
            filename,
            size_bytes,
            mtime_secs,
            date_label: format_friendly_date(mtime_secs),
            is_vertical,
            has_chapters,
            has_gif,
            has_png,
            duration_secs: None,
            width: None,
            height: None,
            thumb_path,
        });
    }

    clips.sort_by(|a, b| b.mtime_secs.cmp(&a.mtime_secs));
    clips
}

fn prettify_clip_name(stem: &str) -> String {
    // Turn `nmap_20261125-133145` -> `nmap`, `deploy-demo_20261126-155400_reels` -> `deploy-demo (Vertical)`
    let (clean, vertical_tag) = if let Some(base) = stem.strip_suffix("_reels") {
        (base, " · Vertical")
    } else {
        (stem, "")
    };
    let (clean, trimmed_tag) = if let Some(base) = clean.strip_suffix("_trimmed") {
        (base, " · Trimmed")
    } else {
        (clean, "")
    };

    // Strip trailing `_YYYYMMDD-HHMMSS` if present
    let base = if let Some(idx) = clean.rfind('_') {
        let suffix = &clean[idx + 1..];
        if suffix.len() == 15
            && suffix.as_bytes()[8] == b'-'
            && suffix[..8].bytes().all(|b| b.is_ascii_digit())
            && suffix[9..].bytes().all(|b| b.is_ascii_digit())
        {
            &clean[..idx]
        } else {
            clean
        }
    } else {
        clean
    };

    let display = if base.is_empty() { clean } else { base };
    format!("{display}{trimmed_tag}{vertical_tag}")
}

fn format_friendly_date(epoch_secs: i64) -> String {
    if epoch_secs <= 0 {
        return String::new();
    }
    let Some(dt_utc) = DateTime::from_timestamp(epoch_secs, 0) else {
        return String::new();
    };
    let dt: DateTime<Local> = dt_utc.into();
    let now = Local::now();

    if dt.date_naive() == now.date_naive() {
        format!("Today, {}", dt.format("%H:%M"))
    } else if (now.date_naive() - dt.date_naive()).num_days() == 1 {
        format!("Yesterday, {}", dt.format("%H:%M"))
    } else {
        dt.format("%b %d · %H:%M").to_string()
    }
}

pub fn thumb_cache_path(thumb_dir: &Path, filename: &str, size: u64, mtime: i64) -> PathBuf {
    // Simple deterministic cache key
    let safe: String = filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    thumb_dir.join(format!("{safe}_{size}_{mtime}.jpg"))
}

pub fn probe_and_thumbnail(path: &Path, size: u64, mtime: i64) -> ClipProbeResult {
    let mut duration_secs = None;
    let mut width = None;
    let mut height = None;

    if let Some(ffprobe) = which("ffprobe") {
        if let Ok(out) = Command::new(ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height:format=duration",
                "-of",
                "json",
            ])
            .arg(path)
            .output()
        {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                duration_secs = v
                    .pointer("/format/duration")
                    .and_then(|d| d.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                width = v
                    .pointer("/streams/0/width")
                    .and_then(|x| x.as_u64())
                    .map(|x| x as u32);
                height = v
                    .pointer("/streams/0/height")
                    .and_then(|x| x.as_u64())
                    .map(|x| x as u32);
            }
        }
    }

    let thumb_dir = cache_dir().join("thumbs");
    let _ = fs::create_dir_all(&thumb_dir);
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("clip.mp4");
    let dest = thumb_cache_path(&thumb_dir, fname, size, mtime);

    let thumb_path = if dest.is_file() {
        Some(dest)
    } else if let Some(ffmpeg) = which("ffmpeg") {
        let seek = duration_secs.map(|d| (d * 0.55).max(0.3)).unwrap_or(1.0);
        let tmp = dest.with_extension(format!("tmp.{}.jpg", std::process::id()));
        let status = Command::new(ffmpeg)
            .args([
                "-y",
                "-loglevel",
                "error",
                "-ss",
                &format!("{seek:.2}"),
                "-i",
            ])
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale=420:-2:flags=lanczos",
                "-q:v",
                "4",
            ])
            .arg(&tmp)
            .status();
        if status.map(|s| s.success()).unwrap_or(false) && tmp.is_file() {
            let _ = fs::rename(&tmp, &dest);
            Some(dest)
        } else {
            let _ = fs::remove_file(&tmp);
            None
        }
    } else {
        None
    };

    ClipProbeResult {
        path: path.to_path_buf(),
        duration_secs,
        width,
        height,
        thumb_path,
    }
}

pub fn play_clip(path: &Path) -> Result<(), String> {
    for player in ["mpv", "parole", "vlc", "totem", "xdg-open"] {
        if let Some(bin) = which(player) {
            let mut cmd = Command::new(bin);
            cmd.arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            unsafe {
                cmd.pre_exec(|| {
                    libc::setsid();
                    Ok(())
                });
            }
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }
    }
    Err("No video player found (install mpv or vlc)".to_string())
}

pub fn open_folder(dir: &Path) {
    let _ = fs::create_dir_all(dir);
    if let Some(bin) = which("xdg-open") {
        let mut cmd = Command::new(bin);
        cmd.arg(dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let _ = cmd.spawn();
    }
}

pub fn rename_clip(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let clean = new_name.trim().trim_end_matches(".mp4").trim();
    if clean.is_empty() {
        return Err("Clip name cannot be empty".to_string());
    }
    if clean.contains('/') || clean.contains('\\') || clean.contains("..") {
        return Err("Invalid characters in clip name".to_string());
    }
    let parent = path.parent().ok_or("Invalid clip path")?;
    let new_mp4 = parent.join(format!("{clean}.mp4"));
    if new_mp4 == path {
        return Ok(new_mp4);
    }
    if new_mp4.exists() {
        return Err(format!("{clean}.mp4 already exists"));
    }
    fs::rename(path, &new_mp4).map_err(|e| format!("Rename failed: {e}"))?;

    // Also rename matching companion files if present
    for ext in ["chapters.txt", "gif", "png"] {
        let old_comp = path.with_extension(ext);
        if old_comp.is_file() {
            let _ = fs::rename(&old_comp, new_mp4.with_extension(ext));
        }
    }
    Ok(new_mp4)
}

pub fn delete_clip(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("Could not delete clip: {e}"))?;
    let _ = fs::remove_file(path.with_extension("chapters.txt"));
    Ok(())
}

pub fn read_chapters(path: &Path) -> Option<String> {
    let chap = path.with_extension("chapters.txt");
    fs::read_to_string(chap).ok().filter(|s| !s.trim().is_empty())
}

pub fn run_rec_cli_tool(args: &[String]) -> Result<String, String> {
    let bin = which("rec-cli").ok_or_else(|| "rec-cli not found in PATH".to_string())?;
    let out = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run rec-cli: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Command failed".to_string()
        } else {
            err.lines().last().unwrap_or("Command failed").to_string()
        })
    }
}

pub fn run_render_pending() -> Result<String, String> {
    let bin = which("casts2mp4").ok_or_else(|| "casts2mp4 not found in PATH".to_string())?;
    let out = Command::new(bin)
        .output()
        .map_err(|e| format!("Failed to run casts2mp4: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Render failed".to_string()
        } else {
            err
        })
    }
}

pub fn format_mm_ss(secs: f64) -> String {
    let total = secs.max(0.0).round() as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m:02}:{s:02}")
}

pub fn copy_to_system_clipboard(text: &str) {
    // Best-effort native Wayland/X11 clipboard helper in addition to iced::clipboard
    for (bin, args) in [
        ("wl-copy", &[][..]),
        ("xclip", &["-selection", "clipboard"][..]),
        ("xsel", &["--clipboard", "--input"][..]),
    ] {
        if let Some(path) = which(bin) {
            if let Ok(mut child) = Command::new(path)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                }
                return;
            }
        }
    }
}
