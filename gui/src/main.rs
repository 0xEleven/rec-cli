mod clips;
mod config;
mod icons;
mod sessions;
mod theme;
mod views;

use clips::{ClipItem, ClipProbeResult};
use config::{
    detect_display_preset, enable_shell_integration, installed_terminals, pick_directory_dialog,
    set_auto_enabled, write_config_key, Config, SystemHealth, TerminalDef, TERMINALS,
};
use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::widget::{
    button, column, container, horizontal_space, row, rule, stack, text, toggler, Space,
};
use iced::{Alignment, Element, Font, Length, Padding, Size, Subscription, Task, Theme};
use sessions::{
    attach_open_terminal, discover_all_terminals, launch_recorded_terminal, render_cast_sync,
    send_ctrl_ack, send_ctrl_fire_and_forget, terminate_session, AttachOutcome, DiscoverySnapshot,
    OpenTerminal, SessionState,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use theme::{Palette, UiMode};

const VERSION: &str = "2.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Record,
    Library,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryLayout {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipFilter {
    All,
    Landscape,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipSort {
    Newest,
    Largest,
    Name,
}

impl std::fmt::Display for ClipSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Newest => write!(f, "Newest"),
            Self::Largest => write!(f, "Largest"),
            Self::Name => write!(f, "Name"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipDrawer {
    Rename,
    Trim,
    Chapters,
    Delete,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub play_path: Option<PathBuf>,
    pub expires_tick: u64,
}

pub struct App {
    pub cfg: Config,
    pub ui_mode: UiMode,
    pub page: Page,
    pub display_label: String,
    pub installed_terms: Vec<&'static TerminalDef>,
    pub health: SystemHealth,

    pub snapshot: DiscoverySnapshot,
    pub clips: Vec<ClipItem>,
    pub probe_cache: HashMap<(PathBuf, u64, i64), ClipProbeResult>,
    pub probing_paths: HashSet<PathBuf>,

    // Record view state
    pub new_clip_name: String,
    pub launch_terminal: String,
    pub chapter_input_pid: Option<i32>,
    pub chapter_draft: String,
    pub confirm_session_pid: Option<(i32, bool)>, // (pid, is_kill)

    // Library view state
    pub library_layout: LibraryLayout,
    pub clip_search: String,
    pub clip_filter: ClipFilter,
    pub clip_sort: ClipSort,
    pub active_clip_drawer: Option<(PathBuf, ClipDrawer)>,
    pub rename_draft: String,
    pub trim_start: String,
    pub trim_end: String,
    pub chapters_preview: Option<String>,

    // Feedback & ticks
    pub tick_count: u64,
    pub busy_label: Option<String>,
    pub toast: Option<Toast>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    RefreshNow,
    DiscoveryLoaded(DiscoverySnapshot),
    ClipsScanned(Vec<ClipItem>),
    ClipProbed(ClipProbeResult),

    // Navigation & Theme
    SwitchPage(Page),
    SetUiMode(UiMode),
    KeyPressed(Key, Modifiers),

    // Record page
    NewClipNameChanged(String),
    LaunchTerminalSelected(String),
    LaunchNewRecording,
    StartSession(i32),
    PauseSession(i32),
    ResumeSession(i32),
    StopSession(i32),
    SnapshotSession(i32),
    StopAllSessions,
    ToggleChapterInput(i32),
    ChapterDraftChanged(String),
    SubmitChapter(i32),
    ConfirmDiscardOrKill(i32, bool),
    DiscardSession(i32),
    KillSession(i32),
    AttachOpenTerminal(OpenTerminal),
    RenderFinished(Result<Vec<PathBuf>, String>, String),

    // Library page
    ClipSearchChanged(String),
    ClipFilterChanged(ClipFilter),
    ClipSortChanged(ClipSort),
    SetLibraryLayout(LibraryLayout),
    PlayClip(PathBuf),
    ExportClipGif(PathBuf),
    ExportClipThumb(PathBuf),
    OpenClipDrawer(PathBuf, ClipDrawer),
    RenameDraftChanged(String),
    SubmitRenameClip(PathBuf),
    TrimStartChanged(String),
    TrimEndChanged(String),
    SubmitTrimClip(PathBuf),
    ConfirmDeleteClip(PathBuf),
    CopyClipPath(PathBuf),
    CopyText(String, String),
    RenderPendingCasts,
    ToolFinished(Result<String, String>, String),

    // Settings
    ToggleAutoRecord(bool),
    ToggleAllTerminals(bool),
    ToggleTerminal(String, bool),
    SetConfig(String, String),
    SetBoolConfig(String, bool),
    BrowseVideoFolder,
    VideoFolderChosen(Option<PathBuf>),
    OpenVideoFolder,
    EnableShellIntegration,

    // Common
    CancelInlineAction,
    DismissToast,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let cfg = Config::load();
        let ui_mode = UiMode::from_str(&cfg.ui_theme);
        let display_label = detect_display_preset();
        let installed_terms = installed_terminals();
        let health = SystemHealth::check();
        let cast_dir = cfg.cast_dir.clone();
        let video_dir = cfg.video_dir.clone();

        let app = Self {
            cfg,
            ui_mode,
            page: Page::Record,
            display_label,
            installed_terms,
            health,
            snapshot: DiscoverySnapshot::default(),
            clips: Vec::new(),
            probe_cache: HashMap::new(),
            probing_paths: HashSet::new(),
            new_clip_name: String::new(),
            launch_terminal: "auto".to_string(),
            chapter_input_pid: None,
            chapter_draft: String::new(),
            confirm_session_pid: None,
            library_layout: LibraryLayout::Grid,
            clip_search: String::new(),
            clip_filter: ClipFilter::All,
            clip_sort: ClipSort::Newest,
            active_clip_drawer: None,
            rename_draft: String::new(),
            trim_start: "00:00".to_string(),
            trim_end: "00:30".to_string(),
            chapters_preview: None,
            tick_count: 0,
            busy_label: None,
            toast: None,
        };

        let init_tasks = Task::batch([
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || discover_all_terminals(&cast_dir))
                        .await
                        .unwrap_or_default()
                },
                Message::DiscoveryLoaded,
            ),
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || clips::scan_clips(&video_dir))
                        .await
                        .unwrap_or_default()
                },
                Message::ClipsScanned,
            ),
        ]);

        (app, init_tasks)
    }

    pub fn filtered_clips(&self) -> Vec<&ClipItem> {
        let q = self.clip_search.trim().to_ascii_lowercase();
        let mut out: Vec<&ClipItem> = self
            .clips
            .iter()
            .filter(|c| {
                let match_q = q.is_empty()
                    || c.display_name.to_ascii_lowercase().contains(&q)
                    || c.filename.to_ascii_lowercase().contains(&q);
                let match_f = match self.clip_filter {
                    ClipFilter::All => true,
                    ClipFilter::Landscape => !c.is_vertical,
                    ClipFilter::Vertical => c.is_vertical,
                };
                match_q && match_f
            })
            .collect();

        match self.clip_sort {
            ClipSort::Newest => out.sort_by(|a, b| b.mtime_secs.cmp(&a.mtime_secs)),
            ClipSort::Largest => out.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes)),
            ClipSort::Name => out.sort_by(|a, b| {
                a.display_name
                    .to_ascii_lowercase()
                    .cmp(&b.display_name.to_ascii_lowercase())
            }),
        }
        out
    }

    fn show_toast(&mut self, msg: impl Into<String>, is_error: bool, play_path: Option<PathBuf>) {
        self.toast = Some(Toast {
            message: msg.into(),
            is_error,
            play_path,
            expires_tick: self.tick_count + if is_error { 6 } else { 5 },
        });
    }

    fn spawn_discovery(&self) -> Task<Message> {
        let cast_dir = self.cfg.cast_dir.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || discover_all_terminals(&cast_dir))
                    .await
                    .unwrap_or_default()
            },
            Message::DiscoveryLoaded,
        )
    }

    fn spawn_clips_scan(&self) -> Task<Message> {
        let video_dir = self.cfg.video_dir.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || clips::scan_clips(&video_dir))
                    .await
                    .unwrap_or_default()
            },
            Message::ClipsScanned,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.tick_count = self.tick_count.wrapping_add(1);
                if let Some(t) = &self.toast {
                    if self.tick_count >= t.expires_tick {
                        self.toast = None;
                    }
                }
                let mut tasks = vec![self.spawn_discovery()];
                if self.page == Page::Library && self.tick_count % 3 == 0 {
                    tasks.push(self.spawn_clips_scan());
                }
                Task::batch(tasks)
            }

            Message::RefreshNow => Task::batch([self.spawn_discovery(), self.spawn_clips_scan()]),

            Message::DiscoveryLoaded(snap) => {
                self.snapshot = snap;
                Task::none()
            }

            Message::ClipsScanned(mut scanned) => {
                let mut probe_tasks = Vec::new();
                for clip in &mut scanned {
                    let key = (clip.path.clone(), clip.size_bytes, clip.mtime_secs);
                    if let Some(cached) = self.probe_cache.get(&key) {
                        clip.duration_secs = cached.duration_secs;
                        clip.width = cached.width;
                        clip.height = cached.height;
                        if cached.thumb_path.is_some() {
                            clip.thumb_path = cached.thumb_path.clone();
                        }
                        if let (Some(w), Some(h)) = (clip.width, clip.height) {
                            if h > w {
                                clip.is_vertical = true;
                            }
                        }
                    } else if !self.probing_paths.contains(&clip.path) && probe_tasks.len() < 4 {
                        self.probing_paths.insert(clip.path.clone());
                        let path = clip.path.clone();
                        let size = clip.size_bytes;
                        let mtime = clip.mtime_secs;
                        probe_tasks.push(Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    clips::probe_and_thumbnail(&path, size, mtime)
                                })
                                .await
                                .unwrap_or(ClipProbeResult {
                                    path: PathBuf::new(),
                                    duration_secs: None,
                                    width: None,
                                    height: None,
                                    thumb_path: None,
                                })
                            },
                            Message::ClipProbed,
                        ));
                    }
                }
                self.clips = scanned;
                Task::batch(probe_tasks)
            }

            Message::ClipProbed(res) => {
                if !res.path.as_os_str().is_empty() {
                    self.probing_paths.remove(&res.path);
                    if let Some(clip) = self.clips.iter_mut().find(|c| c.path == res.path) {
                        clip.duration_secs = res.duration_secs;
                        clip.width = res.width;
                        clip.height = res.height;
                        if res.thumb_path.is_some() {
                            clip.thumb_path = res.thumb_path.clone();
                        }
                        if let (Some(w), Some(h)) = (clip.width, clip.height) {
                            if h > w {
                                clip.is_vertical = true;
                            }
                        }
                        self.probe_cache
                            .insert((clip.path.clone(), clip.size_bytes, clip.mtime_secs), res);
                    }
                }
                // Trigger next batch of unprobed clips smoothly
                let mut next_tasks = Vec::new();
                for clip in &self.clips {
                    let key = (clip.path.clone(), clip.size_bytes, clip.mtime_secs);
                    if !self.probe_cache.contains_key(&key)
                        && !self.probing_paths.contains(&clip.path)
                        && next_tasks.len() < 2
                    {
                        self.probing_paths.insert(clip.path.clone());
                        let path = clip.path.clone();
                        let size = clip.size_bytes;
                        let mtime = clip.mtime_secs;
                        next_tasks.push(Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    clips::probe_and_thumbnail(&path, size, mtime)
                                })
                                .await
                                .unwrap_or(ClipProbeResult {
                                    path: PathBuf::new(),
                                    duration_secs: None,
                                    width: None,
                                    height: None,
                                    thumb_path: None,
                                })
                            },
                            Message::ClipProbed,
                        ));
                    }
                }
                Task::batch(next_tasks)
            }

            Message::SwitchPage(p) => {
                self.page = p;
                self.active_clip_drawer = None;
                self.confirm_session_pid = None;
                self.chapter_input_pid = None;
                match p {
                    Page::Record | Page::Settings => self.spawn_discovery(),
                    Page::Library => Task::batch([self.spawn_discovery(), self.spawn_clips_scan()]),
                }
            }

            Message::SetUiMode(mode) => {
                self.ui_mode = mode;
                self.cfg.ui_theme = mode.as_str().to_string();
                let _ = write_config_key("REC_UI_THEME", mode.as_str());
                Task::none()
            }

            Message::KeyPressed(key, mods) => {
                if key == Key::Named(Named::Escape) {
                    self.active_clip_drawer = None;
                    self.confirm_session_pid = None;
                    self.chapter_input_pid = None;
                    return Task::none();
                }
                if mods.command() {
                    if let Key::Character(c) = key {
                        match c.as_str() {
                            "1" => return self.update(Message::SwitchPage(Page::Record)),
                            "2" => return self.update(Message::SwitchPage(Page::Library)),
                            "3" => return self.update(Message::SwitchPage(Page::Settings)),
                            "r" | "R" => return self.update(Message::RefreshNow),
                            _ => {}
                        }
                    }
                }
                Task::none()
            }

            // ---------------- Record actions ----------------
            Message::NewClipNameChanged(s) => {
                self.new_clip_name = s;
                Task::none()
            }

            Message::LaunchTerminalSelected(label) => {
                if label == "Auto terminal" {
                    self.launch_terminal = "auto".to_string();
                } else {
                    self.launch_terminal = label;
                }
                Task::none()
            }

            Message::LaunchNewRecording => {
                let pref_id = TERMINALS
                    .iter()
                    .find(|t| t.label == self.launch_terminal)
                    .map(|t| t.id)
                    .unwrap_or("auto");
                match launch_recorded_terminal(&self.cfg, pref_id, &self.new_clip_name) {
                    Ok(term_name) => {
                        self.new_clip_name.clear();
                        self.show_toast(format!("Opened {term_name} — recording now"), false, None);
                        let cast_dir = self.cfg.cast_dir.clone();
                        Task::perform(
                            async move {
                                tokio::time::sleep(Duration::from_millis(650)).await;
                                tokio::task::spawn_blocking(move || discover_all_terminals(&cast_dir))
                                    .await
                                    .unwrap_or_default()
                            },
                            Message::DiscoveryLoaded,
                        )
                    }
                    Err(e) => {
                        self.show_toast(e, true, None);
                        Task::none()
                    }
                }
            }

            Message::StartSession(pid) => {
                let name = self.new_clip_name.trim().to_string();
                let cmd = if name.is_empty() {
                    "start".to_string()
                } else {
                    format!("start {name}")
                };
                if send_ctrl_fire_and_forget(pid, &cmd) {
                    if !name.is_empty() {
                        self.new_clip_name.clear();
                    }
                    self.show_toast("Recording started", false, None);
                } else {
                    self.show_toast("Could not reach terminal session", true, None);
                }
                self.spawn_delayed_discovery(220)
            }

            Message::PauseSession(pid) => {
                send_ctrl_fire_and_forget(pid, "pause");
                self.show_toast("Recording paused", false, None);
                self.spawn_delayed_discovery(220)
            }

            Message::ResumeSession(pid) => {
                send_ctrl_fire_and_forget(pid, "resume");
                self.show_toast("Recording resumed", false, None);
                self.spawn_delayed_discovery(220)
            }

            Message::StopSession(pid) => {
                self.busy_label = Some("Saving recording to MP4…".to_string());
                self.chapter_input_pid = None;
                self.confirm_session_pid = None;
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let ack = send_ctrl_ack(pid, "stop_handover", Duration::from_secs(2));
                            if let Some(v) = ack {
                                let cast = v.get("cast").and_then(|c| c.as_str()).unwrap_or("");
                                let manual = v.get("manual").and_then(|m| m.as_bool()).unwrap_or(true);
                                render_cast_sync(cast, manual)
                            } else {
                                // Fallback: signal regular stop
                                send_ctrl_fire_and_forget(pid, "stop");
                                Ok(Vec::new())
                            }
                        })
                        .await
                        .unwrap_or_else(|e| Err(format!("Render task failed: {e}")))
                    },
                    |res| Message::RenderFinished(res, "Saved clip".to_string()),
                )
            }

            Message::SnapshotSession(pid) => {
                self.busy_label = Some("Saving instant snapshot…".to_string());
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let ack = send_ctrl_ack(pid, "snapshot", Duration::from_secs(2))
                                .ok_or_else(|| "Could not capture snapshot".to_string())?;
                            if !ack.get("ok").and_then(|b| b.as_bool()).unwrap_or(false) {
                                return Err("Session is not currently recording".to_string());
                            }
                            let cast = ack.get("cast").and_then(|c| c.as_str()).unwrap_or("");
                            render_cast_sync(cast, true)
                        })
                        .await
                        .unwrap_or_else(|e| Err(format!("Snapshot failed: {e}")))
                    },
                    |res| Message::RenderFinished(res, "Saved snapshot (still recording)".to_string()),
                )
            }

            Message::StopAllSessions => {
                let mut count = 0usize;
                for s in &self.snapshot.managed {
                    if !s.legacy && s.state != SessionState::Standby {
                        if send_ctrl_fire_and_forget(s.pid, "stop") {
                            count += 1;
                        }
                    }
                }
                self.show_toast(
                    format!(
                        "Stopped {} {} — saving in background",
                        count,
                        if count == 1 { "recording" } else { "recordings" }
                    ),
                    false,
                    None,
                );
                self.spawn_delayed_discovery(300)
            }

            Message::ToggleChapterInput(pid) => {
                if self.chapter_input_pid == Some(pid) {
                    self.chapter_input_pid = None;
                } else {
                    self.chapter_input_pid = Some(pid);
                    self.chapter_draft.clear();
                    self.confirm_session_pid = None;
                }
                Task::none()
            }

            Message::ChapterDraftChanged(s) => {
                self.chapter_draft = s;
                Task::none()
            }

            Message::SubmitChapter(pid) => {
                let label = self.chapter_draft.trim().to_string();
                if label.is_empty() {
                    return Task::none();
                }
                send_ctrl_fire_and_forget(pid, &format!("mark {label}"));
                self.chapter_input_pid = None;
                self.chapter_draft.clear();
                self.show_toast(format!("Chapter added: {label}"), false, None);
                Task::none()
            }

            Message::ConfirmDiscardOrKill(pid, is_kill) => {
                self.confirm_session_pid = Some((pid, is_kill));
                self.chapter_input_pid = None;
                Task::none()
            }

            Message::DiscardSession(pid) => {
                self.confirm_session_pid = None;
                send_ctrl_fire_and_forget(pid, "cancel");
                self.show_toast("Recording discarded", false, None);
                self.spawn_delayed_discovery(220)
            }

            Message::KillSession(pid) => {
                self.confirm_session_pid = None;
                terminate_session(pid);
                self.show_toast("Session closed", false, None);
                self.spawn_delayed_discovery(250)
            }

            Message::AttachOpenTerminal(info) => match attach_open_terminal(&info, &self.new_clip_name) {
                AttachOutcome::Started(where_lbl) => {
                    self.new_clip_name.clear();
                    self.show_toast(format!("Recording started in {where_lbl}"), false, None);
                    self.spawn_delayed_discovery(600)
                }
                AttachOutcome::CopyCommand => {
                    let cmd = if self.new_clip_name.trim().is_empty() {
                        "rec start".to_string()
                    } else {
                        format!("rec start {}", self.new_clip_name.trim().replace(' ', "-"))
                    };
                    clips::copy_to_system_clipboard(&cmd);
                    self.show_toast(format!("Copied '{cmd}' — paste in that terminal"), false, None);
                    iced::clipboard::write(cmd)
                }
            },

            Message::RenderFinished(res, ok_prefix) => {
                self.busy_label = None;
                match res {
                    Ok(paths) if !paths.is_empty() => {
                        let first = paths[0].clone();
                        let fname = first
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("clip.mp4");
                        self.show_toast(format!("{ok_prefix}: {fname}"), false, Some(first));
                    }
                    Ok(_) => {
                        self.show_toast("Stopped (no terminal output captured)", false, None);
                    }
                    Err(e) => {
                        self.show_toast(e, true, None);
                    }
                }
                Task::batch([self.spawn_discovery(), self.spawn_clips_scan()])
            }

            // ---------------- Library actions ----------------
            Message::ClipSearchChanged(q) => {
                self.clip_search = q;
                Task::none()
            }

            Message::ClipFilterChanged(f) => {
                self.clip_filter = f;
                Task::none()
            }

            Message::ClipSortChanged(s) => {
                self.clip_sort = s;
                Task::none()
            }

            Message::SetLibraryLayout(l) => {
                self.library_layout = l;
                Task::none()
            }

            Message::PlayClip(path) => {
                if let Err(e) = clips::play_clip(&path) {
                    self.show_toast(e, true, None);
                }
                Task::none()
            }

            Message::ExportClipGif(path) => {
                let p_str = path.to_string_lossy().to_string();
                self.busy_label = Some("Exporting optimized GIF…".to_string());
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            clips::run_rec_cli_tool(&["gif".into(), p_str])
                        })
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()))
                    },
                    |r| Message::ToolFinished(r, "Exported GIF next to clip".to_string()),
                )
            }

            Message::ExportClipThumb(path) => {
                let p_str = path.to_string_lossy().to_string();
                self.busy_label = Some("Saving PNG cover image…".to_string());
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            clips::run_rec_cli_tool(&["thumb".into(), p_str])
                        })
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()))
                    },
                    |r| Message::ToolFinished(r, "Saved PNG thumbnail next to clip".to_string()),
                )
            }

            Message::OpenClipDrawer(path, kind) => {
                if self.active_clip_drawer.as_ref() == Some(&(path.clone(), kind.clone())) {
                    self.active_clip_drawer = None;
                    return Task::none();
                }
                if let Some(clip) = self.clips.iter().find(|c| c.path == path) {
                    match kind {
                        ClipDrawer::Rename => {
                            self.rename_draft = clip
                                .path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&clip.display_name)
                                .to_string();
                        }
                        ClipDrawer::Trim => {
                            self.trim_start = "00:00".to_string();
                            self.trim_end = clip
                                .duration_secs
                                .map(clips::format_mm_ss)
                                .unwrap_or_else(|| "00:30".to_string());
                        }
                        ClipDrawer::Chapters => {
                            self.chapters_preview = clips::read_chapters(&clip.path);
                        }
                        ClipDrawer::Delete => {}
                    }
                }
                self.active_clip_drawer = Some((path, kind));
                Task::none()
            }

            Message::RenameDraftChanged(s) => {
                self.rename_draft = s;
                Task::none()
            }

            Message::SubmitRenameClip(path) => {
                match clips::rename_clip(&path, &self.rename_draft) {
                    Ok(_) => {
                        self.active_clip_drawer = None;
                        self.show_toast("Clip renamed", false, None);
                        self.spawn_clips_scan()
                    }
                    Err(e) => {
                        self.show_toast(e, true, None);
                        Task::none()
                    }
                }
            }

            Message::TrimStartChanged(s) => {
                self.trim_start = s;
                Task::none()
            }

            Message::TrimEndChanged(s) => {
                self.trim_end = s;
                Task::none()
            }

            Message::SubmitTrimClip(path) => {
                let start = self.trim_start.trim().to_string();
                let end = self.trim_end.trim().to_string();
                if start.is_empty() || end.is_empty() {
                    self.show_toast("Enter start and end times (e.g. 00:05 and 00:30)", true, None);
                    return Task::none();
                }
                self.active_clip_drawer = None;
                self.busy_label = Some("Trimming clip…".to_string());
                let p_str = path.to_string_lossy().to_string();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            clips::run_rec_cli_tool(&["trim".into(), p_str, start, end])
                        })
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()))
                    },
                    |r| Message::ToolFinished(r, "Saved trimmed clip".to_string()),
                )
            }

            Message::ConfirmDeleteClip(path) => {
                self.active_clip_drawer = None;
                match clips::delete_clip(&path) {
                    Ok(()) => {
                        self.show_toast("Clip deleted", false, None);
                        self.spawn_clips_scan()
                    }
                    Err(e) => {
                        self.show_toast(e, true, None);
                        Task::none()
                    }
                }
            }

            Message::CopyClipPath(path) => {
                let s = path.to_string_lossy().to_string();
                clips::copy_to_system_clipboard(&s);
                self.show_toast("Copied clip path", false, None);
                iced::clipboard::write(s)
            }

            Message::CopyText(content, notice) => {
                clips::copy_to_system_clipboard(&content);
                self.show_toast(notice, false, None);
                iced::clipboard::write(content)
            }

            Message::RenderPendingCasts => {
                self.busy_label = Some("Rendering pending recordings…".to_string());
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(clips::run_render_pending)
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                    },
                    |r| Message::ToolFinished(r, "Finished rendering pending clips".to_string()),
                )
            }

            Message::ToolFinished(res, ok_msg) => {
                self.busy_label = None;
                match res {
                    Ok(_) => self.show_toast(ok_msg, false, None),
                    Err(e) => self.show_toast(e, true, None),
                }
                Task::batch([self.spawn_discovery(), self.spawn_clips_scan()])
            }

            // ---------------- Settings actions ----------------
            Message::ToggleAutoRecord(enabled) => {
                self.cfg.auto_record = enabled;
                let _ = set_auto_enabled(enabled);
                self.show_toast(
                    if enabled {
                        "Auto-record enabled"
                    } else {
                        "Auto-record disabled"
                    },
                    false,
                    None,
                );
                Task::none()
            }

            Message::ToggleAllTerminals(all) => {
                if all {
                    self.cfg.terminals = "all".to_string();
                } else {
                    self.cfg.terminals = "kitty,tabby".to_string();
                }
                let _ = write_config_key("REC_TERMINALS", &self.cfg.terminals);
                self.show_toast("Saved", false, None);
                Task::none()
            }

            Message::ToggleTerminal(id, checked) => {
                let mut chosen: Vec<String> = TERMINALS
                    .iter()
                    .filter(|t| {
                        if t.id == id {
                            checked
                        } else {
                            self.cfg.is_terminal_selected(t.id)
                        }
                    })
                    .map(|t| t.id.to_string())
                    .collect();
                if chosen.is_empty() {
                    chosen.push("none".to_string());
                }
                self.cfg.terminals = chosen.join(",");
                let _ = write_config_key("REC_TERMINALS", &self.cfg.terminals);
                self.show_toast("Saved", false, None);
                Task::none()
            }

            Message::SetConfig(key, val) => {
                match key.as_str() {
                    "REC_RESOLUTION" => self.cfg.resolution = val.clone(),
                    "REC_SPEED" => self.cfg.speed = val.clone(),
                    "REC_MAX_SECS" => self.cfg.max_secs = val.clone(),
                    "REC_THEME" => self.cfg.video_theme = val.clone(),
                    _ => {}
                }
                let _ = write_config_key(&key, &val);
                self.show_toast("Saved", false, None);
                Task::none()
            }

            Message::SetBoolConfig(key, val) => {
                match key.as_str() {
                    "REC_FRAME" => self.cfg.frame = val,
                    "REC_REDACT" => self.cfg.redact = val,
                    "REC_REELS_COPY" => self.cfg.reels_copy = val,
                    _ => {}
                }
                let _ = write_config_key(&key, if val { "on" } else { "off" });
                self.show_toast("Saved", false, None);
                Task::none()
            }

            Message::BrowseVideoFolder => {
                let cur = self.cfg.video_dir.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || pick_directory_dialog(&cur))
                            .await
                            .ok()
                            .flatten()
                    },
                    Message::VideoFolderChosen,
                )
            }

            Message::VideoFolderChosen(opt) => {
                if let Some(dir) = opt {
                    self.cfg.video_dir = dir.clone();
                    let _ = write_config_key("REC_VIDEO_DIR", &dir.to_string_lossy());
                    self.show_toast("Updated output folder", false, None);
                    return self.spawn_clips_scan();
                }
                Task::none()
            }

            Message::OpenVideoFolder => {
                clips::open_folder(&self.cfg.video_dir);
                Task::none()
            }

            Message::EnableShellIntegration => {
                match enable_shell_integration() {
                    Ok(rc_path) => {
                        self.health = SystemHealth::check();
                        self.show_toast(
                            format!("Enabled in {rc_path} — open a new terminal to activate"),
                            false,
                            None,
                        );
                    }
                    Err(e) => self.show_toast(e, true, None),
                }
                Task::none()
            }

            Message::CancelInlineAction => {
                self.active_clip_drawer = None;
                self.confirm_session_pid = None;
                self.chapter_input_pid = None;
                Task::none()
            }

            Message::DismissToast => {
                self.toast = None;
                Task::none()
            }
        }
    }

    fn spawn_delayed_discovery(&self, ms: u64) -> Task<Message> {
        let cast_dir = self.cfg.cast_dir.clone();
        Task::perform(
            async move {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                tokio::task::spawn_blocking(move || discover_all_terminals(&cast_dir))
                    .await
                    .unwrap_or_default()
            },
            Message::DiscoveryLoaded,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            keyboard::on_key_press(|key, mods| Some(Message::KeyPressed(key, mods))),
        ])
    }

    fn theme(&self) -> Theme {
        self.ui_mode.iced_theme()
    }

    fn title(&self) -> String {
        let rec_n = self
            .snapshot
            .managed
            .iter()
            .filter(|s| s.state == SessionState::Recording)
            .count();
        if rec_n > 0 {
            format!("rec-cli · Recording ({rec_n})")
        } else {
            "rec-cli".to_string()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let p = self.ui_mode.palette();
        let sidebar = self.view_sidebar(p);

        let page_view = match self.page {
            Page::Record => views::record::view(self, p),
            Page::Library => views::library::view(self, p),
            Page::Settings => views::settings::view(self, p),
        };

        let main_canvas = container(page_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::canvas_style(p));

        let layout = row![
            sidebar,
            rule::Rule::vertical(1).style(theme::divider_style(p)),
            main_canvas,
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(toast) = &self.toast {
            let mut toast_row = row![
                icons::icon(
                    if toast.is_error {
                        icons::X_CLOSE
                    } else {
                        icons::CHECK
                    },
                    14.0,
                    if toast.is_error { p.rec } else { p.ok },
                ),
                text(&toast.message).size(12.5).color(p.text),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            if let Some(play_p) = &toast.play_path {
                toast_row = toast_row.push(
                    button(
                        row![
                            icons::icon(icons::PLAY, 11.0, p.primary_fg),
                            text("Play").size(12),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center),
                    )
                    .padding([4, 10])
                    .style(theme::primary_btn_style(p))
                    .on_press(Message::PlayClip(play_p.clone())),
                );
            }

            toast_row = toast_row.push(
                button(icons::icon(icons::X_CLOSE, 13.0, p.text_dim))
                    .padding([3, 4])
                    .style(theme::ghost_btn_style(p))
                    .on_press(Message::DismissToast),
            );

            let toast_overlay = container(
                column![
                    Space::with_height(Length::Fill),
                    row![
                        horizontal_space(),
                        container(toast_row)
                            .padding([10, 14])
                            .style(theme::toast_style(p, toast.is_error)),
                    ]
                ]
                .padding(20),
            )
            .width(Length::Fill)
            .height(Length::Fill);

            stack![layout, toast_overlay].into()
        } else {
            layout.into()
        }
    }

    fn view_sidebar(&self, p: Palette) -> Element<'_, Message> {
        let rec_n = self
            .snapshot
            .managed
            .iter()
            .filter(|s| s.state == SessionState::Recording)
            .count();
        let paused_n = self
            .snapshot
            .managed
            .iter()
            .filter(|s| s.state == SessionState::Paused)
            .count();
        let total_open = self.snapshot.managed.len() + self.snapshot.open_unrecorded.len();
        let has_active_stop = self
            .snapshot
            .managed
            .iter()
            .any(|s| !s.legacy && s.state != SessionState::Standby);

        let brand = row![
            icons::logo_mark(36.0, 25.0),
            column![
                text("rec-cli").size(15).font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                text(format!("v{VERSION}")).size(11).color(p.text_dim),
            ]
            .spacing(0),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let live_pill: Element<'_, Message> = if rec_n > 0 {
            let dot_col = iced::Color {
                a: if self.tick_count % 2 == 1 { 0.45 } else { 1.0 },
                ..p.rec
            };
            container(
                row![
                    icons::icon(icons::DOT_FILLED, 8.0, dot_col),
                    text(format!("Recording · {rec_n}"))
                        .size(12)
                        .font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        })
                        .color(p.rec),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .width(Length::Fill)
            .center_x(Length::Fill)
            .style(theme::badge_style(p.rec_soft, p.rec, None))
            .into()
        } else if paused_n > 0 {
            container(
                row![
                    icons::icon(icons::PAUSE, 10.0, p.warn),
                    text(format!("Paused · {paused_n}"))
                        .size(12)
                        .font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        })
                        .color(p.warn),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .width(Length::Fill)
            .center_x(Length::Fill)
            .style(theme::badge_style(p.warn_soft, p.warn, None))
            .into()
        } else {
            let status_txt = if total_open > 0 {
                format!(
                    "{} {} ready",
                    total_open,
                    if total_open == 1 {
                        "terminal"
                    } else {
                        "terminals"
                    }
                )
            } else {
                "Ready to record".to_string()
            };
            container(text(status_txt).size(12).color(p.text_muted))
                .padding([6, 10])
                .width(Length::Fill)
                .center_x(Length::Fill)
                .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border)))
                .into()
        };

        let nav = column![
            nav_item(
                "Record",
                icons::RECORD_CIRCLE,
                if total_open > 0 { Some(total_open) } else { None },
                self.page == Page::Record,
                Page::Record,
                p,
            ),
            nav_item(
                "Clips",
                icons::LIBRARY,
                if !self.clips.is_empty() {
                    Some(self.clips.len())
                } else {
                    None
                },
                self.page == Page::Library,
                Page::Library,
                p,
            ),
            nav_item(
                "Settings",
                icons::SETTINGS,
                None,
                self.page == Page::Settings,
                Page::Settings,
                p,
            ),
        ]
        .spacing(4);

        let mut footer = column![].spacing(10);

        if has_active_stop {
            footer = footer.push(
                button(
                    row![
                        icons::icon(icons::STOP, 11.0, p.rec),
                        text("Stop All").size(12.5).font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 12])
                .width(Length::Fill)
                .style(theme::danger_btn_style(p))
                .on_press(Message::StopAllSessions),
            );
        }

        footer = footer.push(rule::Rule::horizontal(1).style(theme::divider_style(p)));

        footer = footer.push(
            row![
                text("Auto-record").size(12.5).color(p.text_muted),
                horizontal_space(),
                toggler(self.cfg.auto_record)
                    .size(18)
                    .style(theme::toggler_style(p, true))
                    .on_toggle(Message::ToggleAutoRecord),
            ]
            .align_y(Alignment::Center),
        );

        let theme_switch = container(
            row![
                button(
                    row![
                        icons::icon(
                            icons::MOON,
                            13.0,
                            if self.ui_mode == UiMode::Dark {
                                p.text
                            } else {
                                p.text_muted
                            }
                        ),
                        text("Dark").size(12),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center)
                )
                .padding([6, 8])
                .width(Length::FillPortion(1))
                .style(theme::seg_btn_style(p, self.ui_mode == UiMode::Dark))
                .on_press(Message::SetUiMode(UiMode::Dark)),
                button(
                    row![
                        icons::icon(
                            icons::SUN,
                            13.0,
                            if self.ui_mode == UiMode::Light {
                                p.text
                            } else {
                                p.text_muted
                            }
                        ),
                        text("Light").size(12),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center)
                )
                .padding([6, 8])
                .width(Length::FillPortion(1))
                .style(theme::seg_btn_style(p, self.ui_mode == UiMode::Light))
                .on_press(Message::SetUiMode(UiMode::Light)),
            ]
            .spacing(3),
        )
        .padding(3)
        .width(Length::Fill)
        .style(theme::subpanel_style(p));

        footer = footer.push(theme_switch);

        container(
            column![
                brand,
                Space::with_height(6),
                live_pill,
                Space::with_height(8),
                nav,
                Space::with_height(Length::Fill),
                footer,
            ]
            .spacing(6)
            .padding(Padding { top: 18.0, right: 14.0, bottom: 16.0, left: 14.0 })
            .height(Length::Fill),
        )
        .width(Length::Fixed(222.0))
        .height(Length::Fill)
        .style(theme::sidebar_style(p))
        .into()
    }
}

fn nav_item<'a>(
    label: &'a str,
    icon_bytes: &'static [u8],
    count: Option<usize>,
    active: bool,
    target: Page,
    p: Palette,
) -> Element<'a, Message> {
    let fg = if active { p.text } else { p.text_muted };
    let mut r = row![
        icons::icon(icon_bytes, 16.0, fg),
        text(label).size(13.5).font(Font {
            weight: if active {
                iced::font::Weight::Semibold
            } else {
                iced::font::Weight::Normal
            },
            ..Font::DEFAULT
        }),
        horizontal_space(),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    if let Some(n) = count {
        r = r.push(
            container(text(n.to_string()).size(11).color(p.text_muted))
                .padding([1, 6])
                .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border))),
        );
    }

    button(r)
        .padding([9, 11])
        .width(Length::Fill)
        .style(theme::nav_btn_style(p, active))
        .on_press(Message::SwitchPage(target))
        .into()
}

fn run_smoke_test() -> i32 {
    let cfg = Config::load();
    let snap = discover_all_terminals(&cfg.cast_dir);
    let clips = clips::scan_clips(&cfg.video_dir);
    let disp = detect_display_preset();
    println!(
        "SMOKE_OK (rust-iced v{VERSION} | display={disp} | managed={} | open={} | clips={})",
        snap.managed.len(),
        snap.open_unrecorded.len(),
        clips.len()
    );
    0
}

fn make_window_icon() -> Option<iced::window::Icon> {
    // Crisp 64x64 anti-aliased app icon: dark rounded badge with white `>` and red record dot
    let size: u32 = 64;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let s = size as f32;
    let r = 14.0f32;

    let dist_seg = |px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32| -> f32 {
        let vx = bx - ax;
        let vy = by - ay;
        let wx = px - ax;
        let wy = py - ay;
        let len2 = vx * vx + vy * vy;
        let t = if len2 > 0.0 {
            ((wx * vx + wy * vy) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let dx = px - (ax + t * vx);
        let dy = py - (ay + t * vy);
        (dx * dx + dy * dy).sqrt()
    };

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Signed distance to rounded rectangle [2..62, 2..62]
            let cx = (px - s * 0.5).abs() - (s * 0.5 - 2.0 - r);
            let cy = (py - s * 0.5).abs() - (s * 0.5 - 2.0 - r);
            let ox = cx.max(0.0);
            let oy = cy.max(0.0);
            let sd_box = (ox * ox + oy * oy).sqrt() + cx.max(cy).min(0.0) - r;

            let alpha_bg = (0.5 - sd_box).clamp(0.0, 1.0);
            if alpha_bg <= 0.0 {
                continue;
            }

            let mut cr = 24.0f32;
            let mut cg = 24.0f32;
            let mut cb = 28.0f32;

            // Subtle 1.5px border
            if sd_box > -1.8 {
                let b = ((sd_box + 1.8) / 1.2).clamp(0.0, 1.0);
                cr = cr * (1.0 - b) + 52.0 * b;
                cg = cg * (1.0 - b) + 52.0 * b;
                cb = cb * (1.0 - b) + 62.0 * b;
            }

            // White `>` chevron
            let d1 = dist_seg(px, py, 16.0, 22.0, 29.0, 32.0);
            let d2 = dist_seg(px, py, 29.0, 32.0, 16.0, 42.0);
            let d_chev = d1.min(d2) - 2.7;
            let a_chev = (0.5 - d_chev).clamp(0.0, 1.0);
            if a_chev > 0.0 {
                cr = cr * (1.0 - a_chev) + 244.0 * a_chev;
                cg = cg * (1.0 - a_chev) + 244.0 * a_chev;
                cb = cb * (1.0 - a_chev) + 246.0 * a_chev;
            }

            // Recording red dot
            let dx = px - 45.0;
            let dy = py - 32.0;
            let d_dot = (dx * dx + dy * dy).sqrt() - 7.5;
            let a_dot = (0.5 - d_dot).clamp(0.0, 1.0);
            if a_dot > 0.0 {
                cr = cr * (1.0 - a_dot) + 229.0 * a_dot;
                cg = cg * (1.0 - a_dot) + 72.0 * a_dot;
                cb = cb * (1.0 - a_dot) + 77.0 * a_dot;
            }

            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = cr.round().clamp(0.0, 255.0) as u8;
            rgba[idx + 1] = cg.round().clamp(0.0, 255.0) as u8;
            rgba[idx + 2] = cb.round().clamp(0.0, 255.0) as u8;
            rgba[idx + 3] = (alpha_bg * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    iced::window::icon::from_rgba(rgba, size, size).ok()
}

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("rec-gui {VERSION}");
        return Ok(());
    }
    if args.iter().any(|a| a == "--smoke-test") {
        std::process::exit(run_smoke_test());
    }

    let mut app = iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(iced::window::Settings {
            size: Size::new(1080.0, 720.0),
            min_size: Some(Size::new(820.0, 560.0)),
            icon: make_window_icon(),
            ..Default::default()
        });

    // Prefer crisp system UI font (Adwaita Sans / Cantarell) if present on disk
    for font_path in [
        "/usr/share/fonts/truetype/adwaita/AdwaitaSans-Regular.ttf",
        "/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf",
    ] {
        if let Ok(bytes) = std::fs::read(font_path) {
            app = app.font(bytes);
            if font_path.contains("Adwaita") {
                app = app.default_font(Font::with_name("Adwaita Sans"));
            } else if font_path.contains("Cantarell") {
                app = app.default_font(Font::with_name("Cantarell"));
            }
            break;
        }
    }

    app.run_with(App::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_and_discovery_work() {
        let cfg = Config::load();
        assert!(!cfg.resolution.is_empty());
        assert!(!cfg.video_theme.is_empty());
        let snap = discover_all_terminals(&cfg.cast_dir);
        let _ = snap.managed.len() + snap.open_unrecorded.len();
    }

    #[test]
    fn clip_formatting_and_sizes() {
        assert_eq!(sessions::fmt_size(0), "0 B");
        assert_eq!(sessions::fmt_size(512), "512 B");
        assert_eq!(sessions::fmt_size(1536), "1.5 KB");
        assert_eq!(clips::format_mm_ss(65.0), "01:05");
    }
}
