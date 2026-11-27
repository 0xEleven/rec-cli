use crate::clips::ClipItem;
use crate::config::short_path;
use crate::icons;
use crate::sessions::{fmt_size, truncate};
use crate::theme::{self, Palette};
use crate::{App, ClipDrawer, ClipFilter, ClipSort, LibraryLayout, Message};
use iced::widget::{
    button, column, container, horizontal_space, image, pick_list, row, scrollable, stack, text,
    text_input, tooltip, Space,
};
use iced::{Alignment, ContentFit, Element, Font, Length, Padding};

pub fn view(app: &App, p: Palette) -> Element<'_, Message> {
    let total_bytes: u64 = app.clips.iter().map(|c| c.size_bytes).sum();
    let subtitle = if app.clips.is_empty() {
        format!("Saved to {}", short_path(&app.cfg.video_dir.to_string_lossy()))
    } else {
        format!(
            "{} {}  ·  {}  ·  {}",
            app.clips.len(),
            if app.clips.len() == 1 { "clip" } else { "clips" },
            fmt_size(total_bytes),
            short_path(&app.cfg.video_dir.to_string_lossy()),
        )
    };

    let mut hdr_actions = row![].spacing(8).align_y(Alignment::Center);
    if app.snapshot.pending_casts > 0 {
        hdr_actions = hdr_actions.push(
            button(
                row![
                    icons::icon(icons::REFRESH, 13.0, p.primary_fg),
                    text(format!("Render Pending ({})", app.snapshot.pending_casts))
                        .size(12.5)
                        .font(Font {
                            weight: iced::font::Weight::Semibold,
                            ..Font::DEFAULT
                        }),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([7, 13])
            .style(theme::primary_btn_style(p))
            .on_press(Message::RenderPendingCasts),
        );
    }
    hdr_actions = hdr_actions.push(
        button(
            row![
                icons::icon(icons::FOLDER, 14.0, p.text_muted),
                text("Open Folder").size(12.5),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([7, 12])
        .style(theme::secondary_btn_style(p))
        .on_press(Message::OpenVideoFolder),
    );

    let header = row![
        column![
            text("Clips").size(22).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            text(subtitle).size(12.5).color(p.text_muted),
        ]
        .spacing(3),
        horizontal_space(),
        hdr_actions,
    ]
    .align_y(Alignment::Center);

    // Search + Filter pills + Sort + Grid/List toggle
    let filter_pills = container(
        row![
            filter_btn("All", ClipFilter::All, app.clip_filter, p),
            filter_btn("Landscape", ClipFilter::Landscape, app.clip_filter, p),
            filter_btn("Vertical", ClipFilter::Vertical, app.clip_filter, p),
        ]
        .spacing(2),
    )
    .padding(3)
    .style(theme::subpanel_style(p));

    let layout_toggle = container(
        row![
            tip(
                button(icons::icon(
                    icons::GRID,
                    14.0,
                    if app.library_layout == LibraryLayout::Grid {
                        p.text
                    } else {
                        p.text_muted
                    }
                ))
                .padding([5, 8])
                .style(theme::seg_btn_style(
                    p,
                    app.library_layout == LibraryLayout::Grid
                ))
                .on_press(Message::SetLibraryLayout(LibraryLayout::Grid)),
                "Grid view",
                p,
            ),
            tip(
                button(icons::icon(
                    icons::LIST,
                    14.0,
                    if app.library_layout == LibraryLayout::List {
                        p.text
                    } else {
                        p.text_muted
                    }
                ))
                .padding([5, 8])
                .style(theme::seg_btn_style(
                    p,
                    app.library_layout == LibraryLayout::List
                ))
                .on_press(Message::SetLibraryLayout(LibraryLayout::List)),
                "List view",
                p,
            ),
        ]
        .spacing(2),
    )
    .padding(3)
    .style(theme::subpanel_style(p));

    let toolbar = row![
        container(
            row![
                icons::icon(icons::SEARCH, 14.0, p.text_dim),
                text_input("Search clips…", &app.clip_search)
                    .on_input(Message::ClipSearchChanged)
                    .padding([6, 4])
                    .size(13)
                    .style(move |t, s| {
                        let mut st = theme::input_style(p)(t, s);
                        st.background = iced::Background::Color(iced::Color::TRANSPARENT);
                        st.border.width = 0.0;
                        st
                    })
                    .width(Length::Fill),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .padding([1, 10])
        .style(theme::subpanel_style(p))
        .width(Length::Fill),
        filter_pills,
        pick_list(
            vec![ClipSort::Newest, ClipSort::Largest, ClipSort::Name],
            Some(app.clip_sort),
            Message::ClipSortChanged,
        )
        .padding([7, 11])
        .text_size(12.5)
        .style(theme::pick_list_style(p))
        .menu_style(theme::menu_style(p))
        .width(Length::Fixed(116.0)),
        layout_toggle,
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let filtered = app.filtered_clips();
    let mut body = column![].spacing(12).width(Length::Fill);

    if let Some(job_label) = &app.busy_label {
        let strip = container(
            row![
                icons::icon(icons::DOT_FILLED, 9.0, p.ok),
                text(job_label).size(13).color(p.text),
                horizontal_space(),
                text("Working…").size(12).color(p.text_muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([9, 14])
        .style(theme::subpanel_style(p))
        .width(Length::Fill);
        body = body.push(strip);
    }

    if filtered.is_empty() {
        let msg = if !app.clip_search.trim().is_empty() || app.clip_filter != ClipFilter::All {
            "No clips match your current filter."
        } else {
            "Your recorded clips will appear here automatically after you stop a recording."
        };
        let empty = container(
            column![
                container(icons::icon(icons::LIBRARY, 24.0, p.text_muted))
                    .padding(14)
                    .style(theme::subpanel_style(p)),
                Space::with_height(4),
                text(if app.clips.is_empty() {
                    "No clips saved yet"
                } else {
                    "No matching clips"
                })
                .size(15.5)
                .font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
                text(msg)
                    .size(13)
                    .color(p.text_muted)
                    .align_x(iced::alignment::Horizontal::Center),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .padding([46, 32])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(theme::panel_style(p));
        body = body.push(empty);
    } else if app.library_layout == LibraryLayout::Grid {
        for pair in filtered.chunks(2) {
            let mut r = row![].spacing(14).width(Length::Fill);
            for clip in pair {
                r = r.push(container(clip_grid_card(app, clip, p)).width(Length::FillPortion(1)));
            }
            if pair.len() == 1 {
                r = r.push(Space::with_width(Length::FillPortion(1)));
            }
            body = body.push(r);
        }
    } else {
        for clip in filtered {
            body = body.push(clip_list_card(app, clip, p));
        }
    }

    let scroll = scrollable(container(body).padding(Padding {
        top: 0.0,
        right: 6.0,
        bottom: 16.0,
        left: 0.0,
    }))
    .height(Length::Fill)
    .style(theme::scrollable_style(p));

    column![header, toolbar, scroll]
        .spacing(14)
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

fn filter_btn<'a>(
    label: &'a str,
    target: ClipFilter,
    active: ClipFilter,
    p: Palette,
) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([5, 10])
        .style(theme::seg_btn_style(p, target == active))
        .on_press(Message::ClipFilterChanged(target))
        .into()
}

fn thumbnail_widget<'a>(
    clip: &'a ClipItem,
    height: f32,
    show_center_play: bool,
    p: Palette,
) -> Element<'a, Message> {
    let base: Element<'_, Message> = if let Some(tp) = &clip.thumb_path {
        container(
            image(image::Handle::from_path(tp))
                .width(Length::Fill)
                .height(Length::Fixed(height))
                .content_fit(ContentFit::Cover),
        )
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .clip(true)
        .style(theme::thumb_frame_style(p))
        .into()
    } else {
        container(
            column![icons::icon(icons::TERMINAL, 24.0, p.text_dim)].align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::thumb_frame_style(p))
        .into()
    };

    let dur = clip.duration_label();
    let res = clip.resolution_badge();

    let mut overlay_bottom = row![].align_y(Alignment::Center);
    if !res.is_empty() {
        overlay_bottom = overlay_bottom.push(
            container(text(res).size(10.5).color(iced::Color::WHITE))
                .padding([2, 6])
                .style(theme::overlay_pill_style()),
        );
    }
    overlay_bottom = overlay_bottom.push(horizontal_space());
    if !dur.is_empty() {
        overlay_bottom = overlay_bottom.push(
            container(
                text(dur)
                    .size(11)
                    .font(Font::MONOSPACE)
                    .color(iced::Color::WHITE),
            )
            .padding([2, 6])
            .style(theme::overlay_pill_style()),
        );
    }

    let center_badge: Element<'_, Message> = if show_center_play {
        container(
            container(icons::icon(icons::PLAY, 16.0, iced::Color::WHITE))
                .padding([9, 10])
                .style(theme::play_circle_style()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        Space::with_height(Length::Fill).into()
    };

    let overlay = container(
        column![center_badge, overlay_bottom]
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fixed(height)),
    )
    .width(Length::Fill)
    .height(Length::Fixed(height));

    let stacked = stack![base, overlay];

    button(stacked)
        .padding(0)
        .width(Length::Fill)
        .style(move |_, _| button::Style {
            background: None,
            text_color: p.text,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
        })
        .on_press(Message::PlayClip(clip.path.clone()))
        .into()
}

fn clip_grid_card<'a>(app: &'a App, clip: &'a ClipItem, p: Palette) -> Element<'a, Message> {
    let thumb = thumbnail_widget(clip, 154.0, true, p);

    let mut badges = row![].spacing(5).align_y(Alignment::Center);
    if clip.has_chapters {
        badges = badges.push(
            button(text("Chapters").size(10.5).color(p.text_muted))
                .padding([2, 7])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::OpenClipDrawer(
                    clip.path.clone(),
                    ClipDrawer::Chapters,
                )),
        );
    }
    if clip.has_gif {
        badges = badges.push(
            container(text("GIF").size(10.5).color(p.ok))
                .padding([2, 6])
                .style(theme::badge_style(p.ok_soft, p.ok, None)),
        );
    }
    if clip.has_png {
        badges = badges.push(
            container(text("PNG").size(10.5).color(p.text_muted))
                .padding([2, 6])
                .style(theme::badge_style(p.surface_alt, p.text_muted, Some(p.border))),
        );
    }

    let title_row = row![
        text(truncate(&clip.display_name, 28))
            .size(14)
            .font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            })
            .width(Length::Fill),
        badges,
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let meta = text(format!(
        "{}  ·  {}",
        clip.date_label,
        fmt_size(clip.size_bytes)
    ))
    .size(12)
    .color(p.text_muted);

    let actions = clip_actions_row(clip, p);

    let mut col = column![thumb, title_row, meta, actions].spacing(8);

    if let Some(drawer) = clip_inline_drawer(app, clip, p) {
        col = col.push(drawer);
    }

    container(col)
        .padding(12)
        .width(Length::Fill)
        .style(theme::panel_style(p))
        .into()
}

fn clip_list_card<'a>(app: &'a App, clip: &'a ClipItem, p: Palette) -> Element<'a, Message> {
    let thumb = container(thumbnail_widget(clip, 68.0, false, p)).width(Length::Fixed(122.0));

    let mut meta_parts = vec![clip.date_label.clone(), fmt_size(clip.size_bytes)];
    let res = clip.resolution_badge();
    if !res.is_empty() {
        meta_parts.push(res);
    }
    let dur = clip.duration_label();
    if !dur.is_empty() {
        meta_parts.push(dur);
    }

    let info = column![
        row![
            text(truncate(&clip.display_name, 42)).size(14).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }),
            if clip.has_chapters {
                Element::from(
                    button(text("Chapters").size(10.5).color(p.text_muted))
                        .padding([2, 7])
                        .style(theme::secondary_btn_style(p))
                        .on_press(Message::OpenClipDrawer(
                            clip.path.clone(),
                            ClipDrawer::Chapters,
                        )),
                )
            } else {
                Space::with_width(0).into()
            },
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text(meta_parts.join("  ·  ")).size(12).color(p.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    let main_row = row![thumb, info, clip_actions_row(clip, p)]
        .spacing(14)
        .align_y(Alignment::Center);

    let mut col = column![main_row].spacing(8);
    if let Some(drawer) = clip_inline_drawer(app, clip, p) {
        col = col.push(drawer);
    }

    container(col)
        .padding([10, 14])
        .width(Length::Fill)
        .style(theme::panel_style(p))
        .into()
}

fn clip_actions_row<'a>(clip: &'a ClipItem, p: Palette) -> Element<'a, Message> {
    row![
        button(
            row![
                icons::icon(icons::PLAY, 12.0, p.primary_fg),
                text("Play").size(12).font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
        )
        .padding([6, 12])
        .style(theme::primary_btn_style(p))
        .on_press(Message::PlayClip(clip.path.clone())),
        tip(
            button(text("GIF").size(12))
                .padding([6, 10])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::ExportClipGif(clip.path.clone())),
            "Export a shareable GIF next to this video",
            p,
        ),
        tip(
            button(
                row![
                    icons::icon(icons::SCISSORS, 12.5, p.text),
                    text("Trim").size(12),
                ]
                .spacing(4)
                .align_y(Alignment::Center)
            )
            .padding([6, 10])
            .style(theme::secondary_btn_style(p))
            .on_press(Message::OpenClipDrawer(clip.path.clone(), ClipDrawer::Trim)),
            "Fast lossless trim into a new copy",
            p,
        ),
        horizontal_space(),
        tip(
            button(icons::icon(icons::CAMERA, 13.5, p.text_muted))
                .padding([6, 7])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::ExportClipThumb(clip.path.clone())),
            "Save PNG cover image",
            p,
        ),
        tip(
            button(icons::icon(icons::EDIT, 13.5, p.text_muted))
                .padding([6, 7])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::OpenClipDrawer(
                    clip.path.clone(),
                    ClipDrawer::Rename,
                )),
            "Rename clip",
            p,
        ),
        tip(
            button(icons::icon(icons::COPY, 13.5, p.text_muted))
                .padding([6, 7])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::CopyClipPath(clip.path.clone())),
            "Copy file path",
            p,
        ),
        tip(
            button(icons::icon(icons::TRASH, 13.5, p.rec))
                .padding([6, 7])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::OpenClipDrawer(
                    clip.path.clone(),
                    ClipDrawer::Delete,
                )),
            "Delete clip",
            p,
        ),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

fn clip_inline_drawer<'a>(
    app: &'a App,
    clip: &'a ClipItem,
    p: Palette,
) -> Option<Element<'a, Message>> {
    let (active_path, drawer_kind) = app.active_clip_drawer.as_ref()?;
    if active_path != &clip.path {
        return None;
    }

    let inner: Element<'_, Message> = match drawer_kind {
        ClipDrawer::Rename => row![
            text_input("New clip name…", &app.rename_draft)
                .on_input(Message::RenameDraftChanged)
                .on_submit(Message::SubmitRenameClip(clip.path.clone()))
                .padding([6, 9])
                .size(12.5)
                .style(theme::input_style(p))
                .width(Length::Fill),
            button(text("Save").size(12))
                .padding([6, 11])
                .style(theme::primary_btn_style(p))
                .on_press(Message::SubmitRenameClip(clip.path.clone())),
            button(icons::icon(icons::X_CLOSE, 13.0, p.text_muted))
                .padding([6, 7])
                .style(theme::ghost_btn_style(p))
                .on_press(Message::CancelInlineAction),
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .into(),

        ClipDrawer::Trim => column![
            row![
                text("Lossless Trim").size(12).font(Font {
                    weight: iced::font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
                horizontal_space(),
                button(icons::icon(icons::X_CLOSE, 13.0, p.text_muted))
                    .padding([2, 4])
                    .style(theme::ghost_btn_style(p))
                    .on_press(Message::CancelInlineAction),
            ]
            .align_y(Alignment::Center),
            row![
                text("Start").size(11.5).color(p.text_muted),
                text_input("00:00", &app.trim_start)
                    .on_input(Message::TrimStartChanged)
                    .padding([5, 8])
                    .size(12)
                    .font(Font::MONOSPACE)
                    .style(theme::input_style(p))
                    .width(Length::Fixed(72.0)),
                text("End").size(11.5).color(p.text_muted),
                text_input("00:30", &app.trim_end)
                    .on_input(Message::TrimEndChanged)
                    .on_submit(Message::SubmitTrimClip(clip.path.clone()))
                    .padding([5, 8])
                    .size(12)
                    .font(Font::MONOSPACE)
                    .style(theme::input_style(p))
                    .width(Length::Fixed(72.0)),
                horizontal_space(),
                button(text("Save Trimmed Copy").size(12))
                    .padding([6, 11])
                    .style(theme::primary_btn_style(p))
                    .on_press(Message::SubmitTrimClip(clip.path.clone())),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .into(),

        ClipDrawer::Chapters => {
            let content = app
                .chapters_preview
                .as_deref()
                .unwrap_or("No chapters found.");
            column![
                row![
                    text("Chapters").size(12).font(Font {
                        weight: iced::font::Weight::Semibold,
                        ..Font::DEFAULT
                    }),
                    horizontal_space(),
                    button(
                        row![
                            icons::icon(icons::COPY, 12.0, p.text),
                            text("Copy for YouTube").size(11.5),
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center)
                    )
                    .padding([4, 9])
                    .style(theme::secondary_btn_style(p))
                    .on_press(Message::CopyText(
                        content.to_string(),
                        "Copied chapters".to_string()
                    )),
                    button(icons::icon(icons::X_CLOSE, 13.0, p.text_muted))
                        .padding([2, 4])
                        .style(theme::ghost_btn_style(p))
                        .on_press(Message::CancelInlineAction),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
                text(content)
                    .size(11.5)
                    .font(Font::MONOSPACE)
                    .color(p.text_muted),
            ]
            .spacing(6)
            .into()
        }

        ClipDrawer::Delete => row![
            text("Delete this clip permanently?").size(12.5).color(p.text),
            horizontal_space(),
            button(text("Cancel").size(12))
                .padding([5, 11])
                .style(theme::secondary_btn_style(p))
                .on_press(Message::CancelInlineAction),
            button(text("Delete").size(12))
                .padding([5, 11])
                .style(theme::rec_btn_style(p))
                .on_press(Message::ConfirmDeleteClip(clip.path.clone())),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
    };

    Some(
        container(inner)
            .padding([9, 11])
            .width(Length::Fill)
            .style(theme::subpanel_style(p))
            .into(),
    )
}
