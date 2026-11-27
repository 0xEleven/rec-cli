use iced::overlay::menu;
use iced::widget::{button, container, pick_list, rule, scrollable, text_input, toggler};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Dark,
    Light,
}

impl UiMode {
    pub fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("light") {
            Self::Light
        } else {
            Self::Dark
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Self::Dark => DARK,
            Self::Light => LIGHT,
        }
    }

    pub fn iced_theme(&self) -> Theme {
        let p = self.palette();
        Theme::custom(
            match self {
                Self::Dark => "rec-cli Dark".to_string(),
                Self::Light => "rec-cli Light".to_string(),
            },
            iced::theme::Palette {
                background: p.bg,
                text: p.text,
                primary: p.primary_bg,
                success: p.ok,
                danger: p.rec,
            },
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub sidebar: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub primary_bg: Color,
    pub primary_hover: Color,
    pub primary_fg: Color,
    pub rec: Color,
    pub rec_hover: Color,
    pub rec_soft: Color,
    pub rec_border: Color,
    pub warn: Color,
    pub warn_soft: Color,
    pub ok: Color,
    pub ok_soft: Color,
    pub nav_active: Color,
    pub track_off: Color,
    pub thumb_off: Color,
    pub thumb_on: Color,
    pub shadow: Color,
}

const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    )
}

const fn hexa(rgb: u32, a: f32) -> Color {
    Color::from_rgba(
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
        a,
    )
}

// Warm-neutral architectural monochrome — calm, legible, zero neon/AI slop
pub const DARK: Palette = Palette {
    bg: hex(0x101012),
    sidebar: hex(0x151518),
    surface: hex(0x1A1A1E),
    surface_alt: hex(0x212126),
    surface_hover: hex(0x28282F),
    border: hex(0x28282E),
    border_strong: hex(0x3A3A44),
    text: hex(0xF4F4F6),
    text_muted: hex(0x9A9AA6),
    text_dim: hex(0x666672),
    primary_bg: hex(0xF4F4F6),
    primary_hover: hex(0xFFFFFF),
    primary_fg: hex(0x101012),
    rec: hex(0xE5484D),
    rec_hover: hex(0xF0565B),
    rec_soft: hexa(0xE5484D, 0.14),
    rec_border: hexa(0xE5484D, 0.48),
    warn: hex(0xE5A000),
    warn_soft: hexa(0xE5A000, 0.14),
    ok: hex(0x30A46C),
    ok_soft: hexa(0x30A46C, 0.14),
    nav_active: hex(0x222228),
    track_off: hex(0x303038),
    thumb_off: hex(0xA0A0AC),
    thumb_on: hex(0x101012),
    shadow: hexa(0x000000, 0.28),
};

pub const LIGHT: Palette = Palette {
    bg: hex(0xF6F6F8),
    sidebar: hex(0xEEEEF1),
    surface: hex(0xFFFFFF),
    surface_alt: hex(0xF2F2F5),
    surface_hover: hex(0xE8E8EE),
    border: hex(0xE2E2E8),
    border_strong: hex(0xCBCBD6),
    text: hex(0x141418),
    text_muted: hex(0x5B5B68),
    text_dim: hex(0x8A8A98),
    primary_bg: hex(0x18181D),
    primary_hover: hex(0x2C2C35),
    primary_fg: hex(0xFFFFFF),
    rec: hex(0xD93036),
    rec_hover: hex(0xC0262B),
    rec_soft: hexa(0xD93036, 0.10),
    rec_border: hexa(0xD93036, 0.42),
    warn: hex(0xB4690E),
    warn_soft: hexa(0xB4690E, 0.12),
    ok: hex(0x18794E),
    ok_soft: hexa(0x18794E, 0.11),
    nav_active: hex(0xFFFFFF),
    track_off: hex(0xD2D2DC),
    thumb_off: hex(0xFFFFFF),
    thumb_on: hex(0xFFFFFF),
    shadow: hexa(0x000000, 0.06),
};

// ---------------- Container Styles ----------------

pub fn canvas_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg)),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub fn sidebar_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.sidebar)),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn panel_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: p.shadow,
            offset: Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
    }
}

pub fn panel_rec_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(p.text),
        border: Border {
            color: p.rec_border,
            width: 1.25,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: p.shadow,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    }
}

pub fn subpanel_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface_alt)),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 9.0.into(),
        },
        ..Default::default()
    }
}

pub fn badge_style(bg: Color, fg: Color, border_col: Option<Color>) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        border: Border {
            color: border_col.unwrap_or(Color::TRANSPARENT),
            width: if border_col.is_some() { 1.0 } else { 0.0 },
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn toast_style(p: Palette, is_err: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(if is_err { p.rec } else { p.text }),
        border: Border {
            color: if is_err { p.rec_border } else { p.border_strong },
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
    }
}

pub fn thumb_frame_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface_alt)),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn overlay_pill_style() -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.06, 0.08, 0.82))),
        text_color: Some(Color::WHITE),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

pub fn play_circle_style() -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.06, 0.08, 0.72))),
        text_color: Some(Color::WHITE),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.22),
            width: 1.0,
            radius: 20.0.into(),
        },
        ..Default::default()
    }
}

pub fn tooltip_style(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(p.text),
        border: Border {
            color: p.border_strong,
            width: 1.0,
            radius: 7.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 10.0,
        },
    }
}

// ---------------- Button Styles ----------------

pub fn nav_btn_style(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if active {
            Some(Background::Color(p.nav_active))
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => Some(Background::Color(p.surface_hover)),
                _ => None,
            }
        };
        button::Style {
            background: bg,
            text_color: if active { p.text } else { p.text_muted },
            border: Border {
                color: if active { p.border } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn primary_btn_style(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.primary_hover,
            button::Status::Pressed => p.primary_bg,
            button::Status::Disabled => p.surface_alt,
            button::Status::Active => p.primary_bg,
        };
        let fg = if matches!(status, button::Status::Disabled) {
            p.text_dim
        } else {
            p.primary_fg
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: bg,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn rec_btn_style(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered => p.rec_hover,
            button::Status::Pressed => p.rec,
            button::Status::Disabled => p.surface_alt,
            button::Status::Active => p.rec,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if matches!(status, button::Status::Disabled) {
                p.text_dim
            } else {
                Color::WHITE
            },
            border: Border {
                color: bg,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn secondary_btn_style(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, border_col) = match status {
            button::Status::Hovered => (p.surface_hover, p.border_strong),
            button::Status::Pressed => (p.surface_alt, p.border_strong),
            button::Status::Disabled => (p.surface_alt, p.border),
            button::Status::Active => (p.surface_alt, p.border),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if matches!(status, button::Status::Disabled) {
                p.text_dim
            } else {
                p.text
            },
            border: Border {
                color: border_col,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn ghost_btn_style(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => Some(Background::Color(p.surface_hover)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: match status {
                button::Status::Hovered => p.text,
                _ => p.text_muted,
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 7.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn danger_btn_style(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (bg, fg) = match status {
            button::Status::Hovered | button::Status::Pressed => (p.rec, Color::WHITE),
            _ => (p.rec_soft, p.rec),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn seg_btn_style(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let bg = if active {
            Some(Background::Color(p.surface))
        } else if matches!(status, button::Status::Hovered) {
            Some(Background::Color(p.surface_hover))
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: if active { p.text } else { p.text_muted },
            border: Border {
                color: if active { p.border } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

// ---------------- Form Controls ----------------

pub fn input_style(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let border_col = match status {
            text_input::Status::Focused => p.border_strong,
            text_input::Status::Hovered => p.border_strong,
            _ => p.border,
        };
        text_input::Style {
            background: Background::Color(p.surface_alt),
            border: Border {
                color: border_col,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: p.text_muted,
            placeholder: p.text_dim,
            value: p.text,
            selection: hexa(0x7AA2F7, 0.30),
        }
    }
}

pub fn pick_list_style(p: Palette) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style {
    move |_, status| {
        let border_col = match status {
            pick_list::Status::Hovered | pick_list::Status::Opened => p.border_strong,
            pick_list::Status::Active => p.border,
        };
        pick_list::Style {
            text_color: p.text,
            placeholder_color: p.text_dim,
            handle_color: p.text_muted,
            background: Background::Color(p.surface_alt),
            border: Border {
                color: border_col,
                width: 1.0,
                radius: 8.0.into(),
            },
        }
    }
}

pub fn menu_style(p: Palette) -> impl Fn(&Theme) -> menu::Style {
    move |_| menu::Style {
        background: Background::Color(p.surface),
        border: Border {
            color: p.border_strong,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: p.text,
        selected_text_color: p.text,
        selected_background: Background::Color(p.surface_hover),
    }
}

pub fn toggler_style(p: Palette, accent_red: bool) -> impl Fn(&Theme, toggler::Status) -> toggler::Style {
    move |_, status| {
        let is_toggled = matches!(
            status,
            toggler::Status::Active { is_toggled: true }
                | toggler::Status::Hovered { is_toggled: true }
        );
        let (bg, fg) = if is_toggled {
            if accent_red {
                (p.rec, Color::WHITE)
            } else {
                (p.primary_bg, p.thumb_on)
            }
        } else {
            (p.track_off, p.thumb_off)
        };
        toggler::Style {
            background: bg,
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: fg,
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
        }
    }
}

pub fn divider_style(p: Palette) -> impl Fn(&Theme) -> rule::Style {
    move |_| rule::Style {
        color: p.border,
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}

pub fn scrollable_style(p: Palette) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_, status| {
        let scroller_color = match status {
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => p.border_strong,
            scrollable::Status::Active => Color { a: 0.55, ..p.border_strong },
        };
        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: scroller_color,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
            },
        };
        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
        }
    }
}
