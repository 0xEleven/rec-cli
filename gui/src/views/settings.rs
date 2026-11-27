use crate::config::{short_path, which, TERMINALS};
use crate::icons;
use crate::theme::{self, Palette};
use crate::{App, Message};
use iced::widget::{
    button, column, container, pick_list, row, rule, scrollable, text, toggler, Space,
};
use iced::{Alignment, Color, Element, Font, Length, Padding};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
    pub value: String,
    pub label: String,
}

impl std::fmt::Display for ChoiceOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub fn view(app: &App, p: Palette) -> Element<'_, Message> {
    let header = column![
        text("Settings").size(22).font(Font {
            weight: iced::font::Weight::Semibold,
            ..Font::DEFAULT
        }),
        text("Video quality, look & feel, privacy, and which terminal apps record automatically.")
            .size(13)
            .color(p.text_muted),
    ]
    .spacing(3);

    // ---- 1. Video Quality & Pace ----
    let res_options = vec![
        ChoiceOption {
            value: "auto".into(),
            label: format!("Auto ({})", app.display_label),
        },
        ChoiceOption {
            value: "720p".into(),
            label: "720p HD (1280×720)".into(),
        },
        ChoiceOption {
            value: "1080p".into(),
            label: "1080p Full HD (1920×1080)".into(),
        },
        ChoiceOption {
            value: "1440p".into(),
            label: "1440p QHD (2560×1440)".into(),
        },
        ChoiceOption {
            value: "4k".into(),
            label: "4K Ultra HD (3840×2160)".into(),
        },
        ChoiceOption {
            value: "reels".into(),
            label: "9:16 Vertical — Reels / Shorts (1080×1920)".into(),
        },
        ChoiceOption {
            value: "reels-4k".into(),
            label: "9:16 Vertical 4K (2160×3840)".into(),
        },
    ];
    let res_selected = res_options
        .iter()
        .find(|o| o.value.eq_ignore_ascii_case(&app.cfg.resolution))
        .cloned()
        .or_else(|| res_options.first().cloned());

    let speed_options = vec![
        ChoiceOption { value: "1".into(), label: "1x · Real-time".into() },
        ChoiceOption { value: "1.5".into(), label: "1.5x · Brisk".into() },
        ChoiceOption { value: "2".into(), label: "2x · Snappy (Default)".into() },
        ChoiceOption { value: "2.5".into(), label: "2.5x · Fast".into() },
        ChoiceOption { value: "3".into(), label: "3x · Very fast".into() },
    ];
    let speed_selected = speed_options
        .iter()
        .find(|o| o.value == app.cfg.speed)
        .cloned()
        .or_else(|| speed_options.get(2).cloned());

    let len_options = vec![
        ChoiceOption { value: "60".into(), label: "1 minute".into() },
        ChoiceOption { value: "180".into(), label: "3 minutes".into() },
        ChoiceOption { value: "300".into(), label: "5 minutes (Default)".into() },
        ChoiceOption { value: "600".into(), label: "10 minutes".into() },
        ChoiceOption { value: "900".into(), label: "15 minutes".into() },
    ];
    let len_selected = len_options
        .iter()
        .find(|o| o.value == app.cfg.max_secs)
        .cloned()
        .or_else(|| len_options.get(2).cloned());

    let video_panel = container(
        column![
            row![
                labeled_picker(
                    "Resolution",
                    pick_list(res_options, res_selected, |opt| {
                        Message::SetConfig("REC_RESOLUTION".into(), opt.value)
                    })
                    .padding([8, 12])
                    .text_size(12.5)
                    .style(theme::pick_list_style(p))
                    .menu_style(theme::menu_style(p))
                    .width(Length::Fill),
                    p
                ),
                labeled_picker(
                    "Playback Speed",
                    pick_list(speed_options, speed_selected, |opt| {
                        Message::SetConfig("REC_SPEED".into(), opt.value)
                    })
                    .padding([8, 12])
                    .text_size(12.5)
                    .style(theme::pick_list_style(p))
                    .menu_style(theme::menu_style(p))
                    .width(Length::Fill),
                    p
                ),
                labeled_picker(
                    "Max Clip Length",
                    pick_list(len_options, len_selected, |opt| {
                        Message::SetConfig("REC_MAX_SECS".into(), opt.value)
                    })
                    .padding([8, 12])
                    .text_size(12.5)
                    .style(theme::pick_list_style(p))
                    .menu_style(theme::menu_style(p))
                    .width(Length::Fill),
                    p
                ),
            ]
            .spacing(12),
            Space::with_height(4),
            text("Color Theme for Exported Videos")
                .size(12)
                .color(p.text_muted),
            theme_swatch_row(&app.cfg.video_theme, p),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            row![
                column![
                    text("Output Folder").size(13).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                    text(short_path(&app.cfg.video_dir.to_string_lossy()))
                        .size(12)
                        .font(Font::MONOSPACE)
                        .color(p.text_muted),
                ]
                .spacing(2)
                .width(Length::Fill),
                button(text("Change…").size(12.5))
                    .padding([7, 13])
                    .style(theme::secondary_btn_style(p))
                    .on_press(Message::BrowseVideoFolder),
                button(
                    row![
                        icons::icon(icons::FOLDER, 13.5, p.text_muted),
                        text("Open").size(12.5),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([7, 13])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::OpenVideoFolder),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
    .padding(18)
    .width(Length::Fill)
    .style(theme::panel_style(p));

    // ---- 2. Style & Privacy Switches ----
    let switches_panel = container(
        column![
            switch_row(
                "Rounded window frame",
                "Adds clean padding and a subtle frame around exported videos.",
                app.cfg.frame,
                false,
                |v| Message::SetBoolConfig("REC_FRAME".into(), v),
                p,
            ),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            switch_row(
                "Auto-hide passwords & API keys",
                "Automatically masks tokens, keys, and secrets before saving your clip.",
                app.cfg.redact,
                false,
                |v| Message::SetBoolConfig("REC_REDACT".into(), v),
                p,
            ),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            switch_row(
                "Also save a 9:16 vertical clip",
                "Exports a cursor-following portrait version ready for Reels, Shorts, and TikTok.",
                app.cfg.reels_copy,
                false,
                |v| Message::SetBoolConfig("REC_REELS_COPY".into(), v),
                p,
            ),
        ]
        .spacing(12),
    )
    .padding([16, 18])
    .width(Length::Fill)
    .style(theme::panel_style(p));

    // ---- 3. Terminal Apps & Auto-Record ----
    let is_all = app.cfg.is_all_terminals();
    let auto_header = container(
        column![
            switch_row(
                "Auto-record new terminal windows",
                "When enabled, opening one of your selected terminal apps starts recording automatically. When off, terminals stay ready for 1-click recording.",
                app.cfg.auto_record,
                true,
                Message::ToggleAutoRecord,
                p,
            ),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            switch_row(
                "Allow all terminal apps",
                "Apply auto-record to every supported terminal instead of picking individually.",
                is_all,
                false,
                Message::ToggleAllTerminals,
                p,
            ),
        ]
        .spacing(12),
    )
    .padding([16, 18])
    .width(Length::Fill)
    .style(theme::panel_style(p));

    let mut term_grid = column![].spacing(10).width(Length::Fill);
    for pair in TERMINALS.chunks(2) {
        let mut r = row![].spacing(10).width(Length::Fill);
        for def in pair {
            let installed = def.bins.iter().any(|b| which(b).is_some());
            let open_cnt = app.snapshot.running_counts.get(def.id).copied().unwrap_or(0);
            let checked = app.cfg.is_terminal_selected(def.id);

            let status_el: Element<'_, Message> = if open_cnt > 0 {
                container(
                    row![
                        icons::icon(icons::DOT_FILLED, 7.0, p.ok),
                        text(format!(
                            "Open · {} {}",
                            open_cnt,
                            if open_cnt == 1 { "tab" } else { "tabs" }
                        ))
                        .size(11)
                        .color(p.ok),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .padding([2, 7])
                .style(theme::badge_style(p.ok_soft, p.ok, None))
                .into()
            } else if installed {
                text("Installed").size(11.5).color(p.text_muted).into()
            } else {
                text("Not installed").size(11.5).color(p.text_dim).into()
            };

            let id_owned = def.id.to_string();
            let toggle = if is_all {
                toggler(true)
                    .size(18)
                    .style(theme::toggler_style(p, false))
            } else {
                toggler(checked)
                    .size(18)
                    .style(theme::toggler_style(p, false))
                    .on_toggle(move |v| Message::ToggleTerminal(id_owned.clone(), v))
            };

            let card = container(
                row![
                    column![
                        text(def.label).size(13.5).font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                        status_el,
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    toggle,
                ]
                .align_y(Alignment::Center),
            )
            .padding([12, 14])
            .width(Length::FillPortion(1))
            .style(theme::panel_style(p));

            r = r.push(card);
        }
        term_grid = term_grid.push(r);
    }

    // ---- 4. System & Shell Setup Check ----
    let h = &app.health;
    let video_ok = h.has_ffmpeg && h.has_agg;
    let video_desc = if video_ok {
        "ffmpeg and agg are installed and ready to export MP4s and GIFs."
    } else if !h.has_agg {
        "agg is missing — install it to export MP4 recordings."
    } else {
        "ffmpeg is missing — install it to export MP4 recordings."
    };

    let shell_action: Element<'_, Message> = if h.shell_configured {
        status_pill("Active", true, p)
    } else {
        button(text(format!("Enable in {}", h.detected_shell)).size(12))
            .padding([6, 12])
            .style(theme::primary_btn_style(p))
            .on_press(Message::EnableShellIntegration)
            .into()
    };

    let health_panel = container(
        column![
            row![
                column![
                    text("Shell Integration").size(13.5).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                    text(if h.shell_configured {
                        "New terminal tabs automatically appear in Record for 1-click capture."
                    } else {
                        "Add rec-cli to your shell so new terminal tabs appear ready to record."
                    })
                    .size(12)
                    .color(p.text_muted),
                ]
                .spacing(2)
                .width(Length::Fill),
                shell_action,
            ]
            .align_y(Alignment::Center),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            row![
                column![
                    text("Terminal Capture Engine (rec-pty)").size(13.5).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                    text("Records terminal sessions cleanly without interrupting SSH or commands.")
                        .size(12)
                        .color(p.text_muted),
                ]
                .spacing(2)
                .width(Length::Fill),
                status_pill(if h.has_rec_pty { "Ready" } else { "Missing" }, h.has_rec_pty, p),
            ]
            .align_y(Alignment::Center),
            rule::Rule::horizontal(1).style(theme::divider_style(p)),
            row![
                column![
                    text("Video Exporter (ffmpeg + agg)").size(13.5).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                    text(video_desc).size(12).color(p.text_muted),
                ]
                .spacing(2)
                .width(Length::Fill),
                status_pill(if video_ok { "Ready" } else { "Missing" }, video_ok, p),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
    .padding([16, 18])
    .width(Length::Fill)
    .style(theme::panel_style(p));

    let body = column![
        section_heading("VIDEO & EXPORT", p),
        video_panel,
        Space::with_height(4),
        section_heading("STYLE & PRIVACY", p),
        switches_panel,
        Space::with_height(4),
        section_heading("TERMINAL APPS", p),
        auto_header,
        term_grid,
        Space::with_height(4),
        section_heading("SYSTEM SETUP", p),
        health_panel,
    ]
    .spacing(10)
    .padding(Padding { top: 0.0, right: 6.0, bottom: 20.0, left: 0.0 })
    .width(Length::Fill);

    let scroll = scrollable(body)
        .height(Length::Fill)
        .style(theme::scrollable_style(p));

    column![header, scroll]
        .spacing(16)
        .padding(Padding { top: 24.0, right: 28.0, bottom: 16.0, left: 28.0 })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn status_pill<'a>(label: &'a str, ok: bool, p: Palette) -> Element<'a, Message> {
    let fg = if ok { p.ok } else { p.warn };
    let bg = if ok { p.ok_soft } else { p.warn_soft };
    container(
        row![
            icons::icon(icons::DOT_FILLED, 7.0, fg),
            text(label).size(11.5).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }).color(fg),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([3, 9])
    .style(theme::badge_style(bg, fg, None))
    .into()
}

fn section_heading<'a>(label: &str, p: Palette) -> Element<'a, Message> {
    text(label.to_string())
        .size(11)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
        .color(p.text_dim)
        .into()
}

fn labeled_picker<'a>(
    label: &'a str,
    picker: impl Into<Element<'a, Message>>,
    p: Palette,
) -> Element<'a, Message> {
    column![text(label).size(12).color(p.text_muted), picker.into()]
        .spacing(5)
        .width(Length::FillPortion(1))
        .into()
}

fn switch_row<'a>(
    title: &'a str,
    desc: &'a str,
    checked: bool,
    accent_red: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    p: Palette,
) -> Element<'a, Message> {
    row![
        column![
            text(title).size(13.5).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            text(desc).size(12).color(p.text_muted),
        ]
        .spacing(2)
        .width(Length::Fill),
        Space::with_width(14),
        toggler(checked)
            .size(19)
            .style(theme::toggler_style(p, accent_red))
            .on_toggle(on_toggle),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn theme_swatch_row<'a>(current: &str, p: Palette) -> Element<'a, Message> {
    const THEMES: &[(&str, &str, u32, u32, u32)] = &[
        ("tokyo-night", "Tokyo Night", 0x1A1B26, 0x7AA2F7, 0x9ECE6A),
        ("dracula", "Dracula", 0x282A36, 0xBD93F9, 0x50FA7B),
        ("catppuccin", "Catppuccin", 0x1E1E2E, 0x89B4FA, 0xA6E3A1),
        ("gruvbox", "Gruvbox", 0x282828, 0xFABD2F, 0xB8BB26),
        ("nord", "Nord", 0x2E3440, 0x88C0D0, 0xA3BE8C),
    ];

    let mut r = row![].spacing(8).width(Length::Fill);
    for &(id, label, bg_hex, acc1, acc2) in THEMES {
        let active = current.eq_ignore_ascii_case(id);
        let swatch = container(
            row![
                dot(bg_hex, 10.0),
                dot(acc1, 10.0),
                dot(acc2, 10.0),
            ]
            .spacing(3)
            .align_y(Alignment::Center),
        );
        let id_owned = id.to_string();
        let btn = button(
            column![
                swatch,
                text(label).size(11.5).font(Font {
                    weight: if active {
                        iced::font::Weight::Semibold
                    } else {
                        iced::font::Weight::Normal
                    },
                    ..Font::DEFAULT
                }),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .padding([9, 10])
        .width(Length::FillPortion(1))
        .style(move |_, status| {
            let bg = if active {
                p.surface_hover
            } else if matches!(status, button::Status::Hovered) {
                p.surface_alt
            } else {
                p.surface
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if active { p.text } else { p.text_muted },
                border: iced::Border {
                    color: if active { p.primary_bg } else { p.border },
                    width: if active { 1.4 } else { 1.0 },
                    radius: 9.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .on_press(Message::SetConfig("REC_THEME".into(), id_owned));

        r = r.push(btn);
    }
    r.into()
}

fn dot<'a>(rgb: u32, sz: f32) -> Element<'a, Message> {
    let col = Color::from_rgb(
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    );
    container(Space::new(sz, sz))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(col)),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.16),
                width: 0.75,
                radius: (sz / 2.0).into(),
            },
            ..Default::default()
        })
        .into()
}
