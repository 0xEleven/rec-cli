use crate::clips::ClipItem;
use crate::icons;
use crate::sessions::{fmt_size, pretty_term, truncate, ManagedSession, OpenTerminal, SessionState};
use crate::theme::{self, Palette};
use crate::{App, Message, Page};
use iced::widget::{
    button, column, container, horizontal_space, image, pick_list, row, scrollable, stack, text,
    text_input, tooltip, Space,
};
use iced::{Alignment, ContentFit, Element, Font, Length, Padding};

pub fn view(app: &App, p: Palette) -> Element<'_, Message> {
    let header = row![
        column![
            text("Record").size(22).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            text("Capture clean terminal demos, walkthroughs, and proof-of-concepts.")
                .size(13)
                .color(p.text_muted),
        ]
        .spacing(3),
        horizontal_space(),
        tip(
            button(
                row![
                    icons::icon(icons::REFRESH, 14.0, p.text_muted),
                    text("Refresh").size(12.5),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .padding([7, 12])
            .style(theme::secondary_btn_style(p))
            .on_press(Message::RefreshNow),
            "Rescan open terminal windows (Ctrl+R)",
            p,
        ),
    ]
    .align_y(Alignment::Center);

    // Quick-Start Bar: inline clip name + terminal picker + New Recording CTA
    let mut term_options: Vec<String> = vec!["Auto terminal".to_string()];
    for t in &app.installed_terms {
        term_options.push(t.label.to_string());
    }
    let selected_term = if app.launch_terminal == "auto" {
        "Auto terminal".to_string()
    } else {
        app.launch_terminal.clone()
    };

    // Quick 1-click preset strip right inside the quick-start card
    let res_norm = app.cfg.resolution.to_ascii_lowercase();
    let format_pills = container(
        row![
            preset_btn("Auto", res_norm == "auto", Message::SetConfig("REC_RESOLUTION".into(), "auto".into()), p),
            preset_btn("1080p", res_norm == "1080p", Message::SetConfig("REC_RESOLUTION".into(), "1080p".into()), p),
            preset_btn("4K", res_norm == "4k", Message::SetConfig("REC_RESOLUTION".into(), "4k".into()), p),
            preset_btn("9:16 Reels", res_norm == "reels" || res_norm == "reels-4k", Message::SetConfig("REC_RESOLUTION".into(), "reels".into()), p),
        ]
        .spacing(2),
    )
    .padding(2)
    .style(theme::subpanel_style(p));

    let speed_pills = container(
        row![
            preset_btn("1x", app.cfg.speed == "1", Message::SetConfig("REC_SPEED".into(), "1".into()), p),
            preset_btn("1.5x", app.cfg.speed == "1.5", Message::SetConfig("REC_SPEED".into(), "1.5".into()), p),
            preset_btn("2x", app.cfg.speed == "2", Message::SetConfig("REC_SPEED".into(), "2".into()), p),
        ]
        .spacing(2),
    )
    .padding(2)
    .style(theme::subpanel_style(p));

    let redact_on = app.cfg.redact;
    let redact_pill = tip(
        button(
            row![
                icons::icon(icons::SHIELD, 13.0, if redact_on { p.ok } else { p.text_dim }),
                text(if redact_on { "Secrets hidden" } else { "Secrets visible" })
                    .size(11.5)
                    .color(if redact_on { p.ok } else { p.text_muted }),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([4, 9])
        .style(move |_, status| {
            let bg = if redact_on {
                p.ok_soft
            } else if matches!(status, button::Status::Hovered) {
                p.surface_hover
            } else {
                p.surface_alt
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if redact_on { p.ok } else { p.text_muted },
                border: iced::Border {
                    color: if redact_on {
                        iced::Color::TRANSPARENT
                    } else {
                        p.border
                    },
                    width: 1.0,
                    radius: 7.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .on_press(Message::SetBoolConfig("REC_REDACT".into(), !redact_on)),
        "Automatically mask API keys, tokens, and passwords in exported videos",
        p,
    );

    let quick_bar = container(
        column![
            row![
                text_input(
                    "Name your next recording (optional, e.g. login-demo)…",
                    &app.new_clip_name
                )
                .on_input(Message::NewClipNameChanged)
                .on_submit(Message::LaunchNewRecording)
                .padding([9, 12])
                .size(13)
                .style(theme::input_style(p))
                .width(Length::Fill),
                pick_list(term_options, Some(selected_term), Message::LaunchTerminalSelected)
                    .padding([8, 12])
                    .text_size(12.5)
                    .style(theme::pick_list_style(p))
                    .menu_style(theme::menu_style(p))
                    .width(Length::Fixed(148.0)),
                button(
                    row![
                        icons::icon(icons::RECORD_CIRCLE, 15.0, iced::Color::WHITE),
                        text("New Recording").size(13).font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                    ]
                    .spacing(7)
                    .align_y(Alignment::Center)
                )
                .padding([9, 16])
                .style(theme::rec_btn_style(p))
                .on_press(Message::LaunchNewRecording),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Format").size(11.5).color(p.text_dim),
                format_pills,
                Space::with_width(4),
                text("Speed").size(11.5).color(p.text_dim),
                speed_pills,
                horizontal_space(),
                redact_pill,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding([13, 15])
    .style(theme::panel_style(p))
    .width(Length::Fill);

    let mut content = column![].spacing(14).width(Length::Fill);

    // Active background export/save status strip
    if let Some(job_label) = &app.busy_label {
        let strip = container(
            row![
                icons::icon(icons::DOT_FILLED, 9.0, p.ok),
                text(job_label).size(13).color(p.text),
                horizontal_space(),
                text("Working in background…").size(12).color(p.text_muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([10, 14])
        .style(theme::subpanel_style(p))
        .width(Length::Fill);
        content = content.push(strip);
    }

    let active_sessions: Vec<&ManagedSession> = app
        .snapshot
        .managed
        .iter()
        .filter(|s| s.state != SessionState::Standby)
        .collect();
    let ready_sessions: Vec<&ManagedSession> = app
        .snapshot
        .managed
        .iter()
        .filter(|s| s.state == SessionState::Standby)
        .collect();
    let open_terms = &app.snapshot.open_unrecorded;

    // 1. Active Recordings (Recording / Paused)
    if !active_sessions.is_empty() {
        content = content.push(section_heading(
            &format!("RECORDING NOW ({})", active_sessions.len()),
            p,
        ));
        for s in active_sessions {
            content = content.push(active_session_card(app, s, p));
        }
    }

    // 2. Ready Terminals (1-click in-place recording via silent standby)
    if !ready_sessions.is_empty() {
        content = content.push(section_heading(
            &format!("READY TO RECORD ({})", ready_sessions.len()),
            p,
        ));
        for s in ready_sessions {
            content = content.push(ready_session_card(app, s, p));
        }
    }

    // 3. Other Open Terminal Windows (pre-hook / unattached)
    if !open_terms.is_empty() {
        content = content.push(
            column![
                section_heading(&format!("OPEN TERMINAL WINDOWS ({})", open_terms.len()), p),
                text("Opened before rec-cli started. Click Record where supported, or paste 'rec start' once inside the tab.")
                    .size(12)
                    .color(p.text_dim),
            ]
            .spacing(3),
        );
        for o in open_terms {
            content = content.push(open_terminal_card(o, p));
        }
    }

    // 4. Calm Empty State
    if app.snapshot.managed.is_empty() && open_terms.is_empty() {
        let empty = container(
            column![
                container(icons::icon(icons::TERMINAL, 24.0, p.text_muted))
                    .padding(13)
                    .style(theme::subpanel_style(p)),
                Space::with_height(2),
                text("No open terminals right now")
                    .size(15.5)
                    .font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                text("Click New Recording above to open a terminal and start capturing right away,\nor open any terminal app and it will appear here automatically.")
                    .size(13)
                    .color(p.text_muted)
                    .align_x(iced::alignment::Horizontal::Center),
                Space::with_height(4),
                button(
                    row![
                        icons::icon(icons::RECORD_CIRCLE, 15.0, iced::Color::WHITE),
                        text("Start a Recording").size(13).font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                    ]
                    .spacing(7)
                    .align_y(Alignment::Center)
                )
                .padding([9, 18])
                .style(theme::rec_btn_style(p))
                .on_press(Message::LaunchNewRecording),
            ]
            .spacing(9)
            .align_x(Alignment::Center),
        )
        .padding([36, 28])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(theme::panel_style(p));

        content = content.push(empty);
    }

    // 5. Recent Clips Strip (instant preview & playback right from Home!)
    if !app.clips.is_empty() {
        let recent_hdr = row![
            section_heading("RECENT CLIPS", p),
            horizontal_space(),
            button(
                row![
                    text(format!("All {} clips", app.clips.len()))
                        .size(12)
                        .color(p.text_muted),
                    icons::icon(icons::ARROW_RIGHT, 13.0, p.text_muted),
                ]
                .spacing(4)
                .align_y(Alignment::Center)
            )
            .padding([3, 8])
            .style(theme::ghost_btn_style(p))
            .on_press(Message::SwitchPage(Page::Library)),
        ]
        .align_y(Alignment::Center);

        let mut strip_row = row![].spacing(12).width(Length::Fill);
        for clip in app.clips.iter().take(3) {
            strip_row = strip_row.push(recent_clip_card(clip, p));
        }
        let missing = 3usize.saturating_sub(app.clips.len().min(3));
        for _ in 0..missing {
            strip_row = strip_row.push(Space::with_width(Length::FillPortion(1)));
        }

        content = content.push(Space::with_height(4));
        content = content.push(recent_hdr);
        content = content.push(strip_row);
    }

    let scroll = scrollable(container(content).padding(Padding {
        top: 0.0,
        right: 6.0,
        bottom: 16.0,
        left: 0.0,
    }))
    .height(Length::Fill)
    .style(theme::scrollable_style(p));

    column![header, quick_bar, scroll]
        .spacing(15)
        .padding(Padding {
            top: 24.0,
            right: 28.0,
            bottom: 16.0,
            left: 28.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn preset_btn<'a>(
    label: &'a str,
    active: bool,
    msg: Message,
    p: Palette,
) -> Element<'a, Message> {
    button(text(label).size(11.5))
        .padding([4, 9])
        .style(theme::seg_btn_style(p, active))
        .on_press(msg)
        .into()
}

fn tip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    p: Palette,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(label).size(11.5).color(p.text))
            .padding([5, 9])
            .style(theme::tooltip_style(p)),
        tooltip::Position::Top,
    )
    .gap(6)
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

fn recent_clip_card<'a>(clip: &'a ClipItem, p: Palette) -> Element<'a, Message> {
    let thumb_h = 96.0;
    let base: Element<'_, Message> = if let Some(tp) = &clip.thumb_path {
        container(
            image(image::Handle::from_path(tp))
                .width(Length::Fill)
                .height(Length::Fixed(thumb_h))
                .content_fit(ContentFit::Cover),
        )
        .width(Length::Fill)
        .height(Length::Fixed(thumb_h))
        .clip(true)
        .style(theme::thumb_frame_style(p))
        .into()
    } else {
        container(icons::icon(icons::TERMINAL, 20.0, p.text_dim))
            .width(Length::Fill)
            .height(Length::Fixed(thumb_h))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(theme::thumb_frame_style(p))
            .into()
    };

    let dur = clip.duration_label();
    let overlay = container(
        column![
            Space::with_height(Length::Fill),
            row![
                horizontal_space(),
                if !dur.is_empty() {
                    Element::from(
                        container(
                            text(dur)
                                .size(10.5)
                                .font(Font::MONOSPACE)
                                .color(iced::Color::WHITE),
                        )
                        .padding([2, 5])
                        .style(theme::overlay_pill_style()),
                    )
                } else {
                    Space::with_width(0).into()
                }
            ],
        ]
        .padding(6),
    )
    .width(Length::Fill)
    .height(Length::Fixed(thumb_h));

    let preview = stack![base, overlay];

    let card_inner = column![
        preview,
        column![
            text(truncate(&clip.display_name, 24)).size(12.5).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            text(format!("{} · {}", clip.date_label, fmt_size(clip.size_bytes)))
                .size(11)
                .color(p.text_muted),
        ]
        .spacing(2),
    ]
    .spacing(7);

    button(card_inner)
        .padding(9)
        .width(Length::FillPortion(1))
        .style(move |_, status| {
            let border = if matches!(status, button::Status::Hovered) {
                p.border_strong
            } else {
                p.border
            };
            button::Style {
                background: Some(iced::Background::Color(p.surface)),
                text_color: p.text,
                border: iced::Border {
                    color: border,
                    width: 1.0,
                    radius: 11.0.into(),
                },
                shadow: iced::Shadow {
                    color: p.shadow,
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 4.0,
                },
            }
        })
        .on_press(Message::PlayClip(clip.path.clone()))
        .into()
}

fn active_session_card<'a>(
    app: &'a App,
    s: &'a ManagedSession,
    p: Palette,
) -> Element<'a, Message> {
    let is_rec = s.state == SessionState::Recording;
    let pulse_alpha = if is_rec && app.tick_count % 2 == 1 {
        0.45
    } else {
        1.0
    };

    let status_badge = if is_rec {
        let dot_col = iced::Color {
            a: pulse_alpha,
            ..p.rec
        };
        container(
            row![
                icons::icon(icons::DOT_FILLED, 8.0, dot_col),
                text(if s.legacy { "REC · Legacy" } else { "REC" })
                    .size(11)
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::DEFAULT
                    })
                    .color(p.rec),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([3, 8])
        .style(theme::badge_style(p.rec_soft, p.rec, None))
    } else {
        container(
            row![
                icons::icon(icons::PAUSE, 10.0, p.warn),
                text("PAUSED")
                    .size(11)
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::DEFAULT
                    })
                    .color(p.warn),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([3, 8])
        .style(theme::badge_style(p.warn_soft, p.warn, None))
    };

    let term_tag = container(text(pretty_term(&s.terminal)).size(11).color(p.text_muted))
        .padding([2, 7])
        .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border)));

    let timer_and_size = row![
        text(s.elapsed_str())
            .size(15)
            .font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::MONOSPACE
            })
            .color(if is_rec { p.rec } else { p.warn }),
        text(format!("·  {}", fmt_size(s.cast_size)))
            .size(12)
            .color(p.text_muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let top_row = row![
        status_badge,
        text(s.friendly_title()).size(14.5).font(Font {
            weight: iced::font::Weight::Semibold,
            ..Font::DEFAULT
        }),
        term_tag,
        horizontal_space(),
        timer_and_size,
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    let meta_row = row![
        text(s.subtitle()).size(12.5).color(p.text_muted),
        horizontal_space(),
        text(format!("PID {}", s.pid))
            .size(11)
            .font(Font::MONOSPACE)
            .color(p.text_dim),
    ]
    .align_y(Alignment::Center);

    let actions: Element<'_, Message> = if s.legacy {
        row![
            text("Legacy session — type 'exit' in the terminal to save, or close:")
                .size(12)
                .color(p.text_muted),
            horizontal_space(),
            button(text("Close").size(12))
                .padding([6, 12])
                .style(theme::danger_btn_style(p))
                .on_press(Message::ConfirmDiscardOrKill(s.pid, true)),
        ]
        .align_y(Alignment::Center)
        .into()
    } else if is_rec {
        row![
            tip(
                button(
                    row![
                        icons::icon(icons::STOP, 12.0, p.primary_fg),
                        text("Stop & Save").size(12.5).font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([7, 14])
                .style(theme::primary_btn_style(p))
                .on_press(Message::StopSession(s.pid)),
                "Stop recording and export MP4 (keeps terminal open)",
                p,
            ),
            button(
                row![
                    icons::icon(icons::PAUSE, 12.0, p.text),
                    text("Pause").size(12.5),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .padding([7, 12])
            .style(theme::secondary_btn_style(p))
            .on_press(Message::PauseSession(s.pid)),
            tip(
                button(
                    row![
                        icons::icon(icons::CAMERA, 13.0, p.text),
                        text("Snapshot").size(12.5),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([7, 12])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::SnapshotSession(s.pid)),
                "Save an MP4 right now without stopping this recording",
                p,
            ),
            tip(
                button(
                    row![
                        icons::icon(icons::BOOKMARK, 13.0, p.text),
                        text("Chapter").size(12.5),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([7, 12])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::ToggleChapterInput(s.pid)),
                "Drop a YouTube chapter marker at this timestamp",
                p,
            ),
            horizontal_space(),
            tip(
                button(
                    row![
                        icons::icon(icons::TRASH, 13.0, p.rec),
                        text("Discard").size(12.5),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center)
                )
                .padding([7, 11])
                .style(theme::danger_btn_style(p))
                .on_press(Message::ConfirmDiscardOrKill(s.pid, false)),
                "Stop and discard this take without saving",
                p,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            button(
                row![
                    icons::icon(icons::RECORD_CIRCLE, 13.0, iced::Color::WHITE),
                    text("Resume").size(12.5).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .padding([7, 14])
            .style(theme::rec_btn_style(p))
            .on_press(Message::ResumeSession(s.pid)),
            button(
                row![
                    icons::icon(icons::STOP, 12.0, p.primary_fg),
                    text("Stop & Save").size(12.5).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .padding([7, 14])
            .style(theme::primary_btn_style(p))
            .on_press(Message::StopSession(s.pid)),
            tip(
                button(
                    row![
                        icons::icon(icons::CAMERA, 13.0, p.text),
                        text("Snapshot").size(12.5),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .padding([7, 12])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::SnapshotSession(s.pid)),
                "Save an MP4 right now without stopping",
                p,
            ),
            horizontal_space(),
            button(
                row![
                    icons::icon(icons::TRASH, 13.0, p.rec),
                    text("Discard").size(12.5),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            )
            .padding([7, 11])
            .style(theme::danger_btn_style(p))
            .on_press(Message::ConfirmDiscardOrKill(s.pid, false)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let mut card_col = column![top_row, meta_row, Space::with_height(2), actions].spacing(8);

    // Inline chapter marker input drawer (no OS popup dialog!)
    if app.chapter_input_pid == Some(s.pid) {
        let drawer = container(
            row![
                icons::icon(icons::BOOKMARK, 14.0, p.text_muted),
                text_input(
                    "Chapter title (e.g. Install complete, Demo step 2)…",
                    &app.chapter_draft
                )
                .on_input(Message::ChapterDraftChanged)
                .on_submit(Message::SubmitChapter(s.pid))
                .padding([7, 10])
                .size(12.5)
                .style(theme::input_style(p))
                .width(Length::Fill),
                button(text("Add Chapter").size(12.5))
                    .padding([7, 12])
                    .style(theme::primary_btn_style(p))
                    .on_press(Message::SubmitChapter(s.pid)),
                button(icons::icon(icons::X_CLOSE, 14.0, p.text_muted))
                    .padding([6, 8])
                    .style(theme::ghost_btn_style(p))
                    .on_press(Message::CancelInlineAction),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([9, 11])
        .style(theme::subpanel_style(p));
        card_col = card_col.push(drawer);
    }

    // Inline Discard / Kill confirmation drawer
    if app.confirm_session_pid == Some((s.pid, false))
        || app.confirm_session_pid == Some((s.pid, true))
    {
        let is_kill = app.confirm_session_pid == Some((s.pid, true));
        let prompt = if is_kill {
            "Close this terminal session process?"
        } else {
            "Discard this take without saving a video?"
        };
        let confirm = container(
            row![
                text(prompt).size(12.5).color(p.text),
                horizontal_space(),
                button(text("Cancel").size(12))
                    .padding([6, 12])
                    .style(theme::secondary_btn_style(p))
                    .on_press(Message::CancelInlineAction),
                button(text(if is_kill { "Close Session" } else { "Discard Take" }).size(12))
                    .padding([6, 12])
                    .style(theme::rec_btn_style(p))
                    .on_press(if is_kill {
                        Message::KillSession(s.pid)
                    } else {
                        Message::DiscardSession(s.pid)
                    }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([9, 12])
        .style(theme::subpanel_style(p));
        card_col = card_col.push(confirm);
    }

    container(card_col)
        .padding([14, 16])
        .width(Length::Fill)
        .style(move |t| {
            if is_rec {
                theme::panel_rec_style(p)(t)
            } else {
                theme::panel_style(p)(t)
            }
        })
        .into()
}

fn ready_session_card<'a>(
    app: &'a App,
    s: &'a ManagedSession,
    p: Palette,
) -> Element<'a, Message> {
    let badge = container(
        row![
            icons::icon(icons::DOT_FILLED, 7.0, p.ok),
            text("READY")
                .size(10.5)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .color(p.ok),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([2, 7])
    .style(theme::badge_style(p.ok_soft, p.ok, None));

    let term_tag = container(text(pretty_term(&s.terminal)).size(11).color(p.text_muted))
        .padding([2, 7])
        .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border)));

    let info = column![
        row![
            badge,
            text(s.friendly_title()).size(14).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            term_tag,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text(s.subtitle()).size(12.5).color(p.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    let main_row = row![
        info,
        button(
            row![
                icons::icon(icons::RECORD_CIRCLE, 13.0, iced::Color::WHITE),
                text("Record").size(12.5).font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .padding([7, 14])
        .style(theme::rec_btn_style(p))
        .on_press(Message::StartSession(s.pid)),
        tip(
            button(icons::icon(icons::X_CLOSE, 14.0, p.text_dim))
                .padding([7, 8])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::ConfirmDiscardOrKill(s.pid, true)),
            "Close standby session",
            p,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut card_col = column![main_row].spacing(8);

    if app.confirm_session_pid == Some((s.pid, true)) {
        let confirm = container(
            row![
                text("Close this standby terminal session?")
                    .size(12.5)
                    .color(p.text),
                horizontal_space(),
                button(text("Cancel").size(12))
                    .padding([5, 11])
                    .style(theme::secondary_btn_style(p))
                    .on_press(Message::CancelInlineAction),
                button(text("Close").size(12))
                    .padding([5, 11])
                    .style(theme::rec_btn_style(p))
                    .on_press(Message::KillSession(s.pid)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 11])
        .style(theme::subpanel_style(p));
        card_col = card_col.push(confirm);
    }

    container(card_col)
        .padding([12, 16])
        .width(Length::Fill)
        .style(theme::panel_style(p))
        .into()
}

fn open_terminal_card<'a>(o: &'a OpenTerminal, p: Palette) -> Element<'a, Message> {
    let badge = container(
        text("OPEN")
            .size(10.5)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            })
            .color(p.text_muted),
    )
    .padding([2, 7])
    .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border)));

    let info = column![
        row![
            badge,
            text(o.friendly_title()).size(13.5).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text(o.subtitle()).size(12).color(p.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    let action_btn = if o.can_remote_record() {
        button(
            row![
                icons::icon(icons::RECORD_CIRCLE, 13.0, iced::Color::WHITE),
                text("Record").size(12.5).font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([7, 14])
        .style(theme::rec_btn_style(p))
        .on_press(Message::AttachOpenTerminal(o.clone()))
    } else {
        button(
            row![
                icons::icon(icons::COPY, 13.0, p.text),
                text("Copy 'rec start'").size(12.5),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([7, 12])
        .style(theme::secondary_btn_style(p))
        .on_press(Message::AttachOpenTerminal(o.clone()))
    };

    container(
        row![info, action_btn]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([11, 16])
    .width(Length::Fill)
    .style(theme::panel_style(p))
    .into()
}
