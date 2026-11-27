use iced::widget::svg;
use iced::{Color, Length};

pub fn icon<'a>(data: &'static [u8], size: f32, color: Color) -> svg::Svg<'a> {
    svg::Svg::new(svg::Handle::from_memory(data))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme, _status| svg::Style { color: Some(color) })
}

pub fn logo_mark<'a>(width: f32, height: f32) -> svg::Svg<'a> {
    svg::Svg::new(svg::Handle::from_memory(LOGO))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
}

// Brand mark: crisp dark pill with `>` terminal prompt and red recording dot
pub const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 28" fill="none">
  <rect x="0.75" y="0.75" width="38.5" height="26.5" rx="7" fill="#18181C" stroke="#2E2E36" stroke-width="1.5"/>
  <path d="M10.5 9.5L16.5 14L10.5 18.5" stroke="#F4F4F6" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>
  <circle cx="27.5" cy="14" r="4.2" fill="#E5484D"/>
</svg>"##;

pub const RECORD_CIRCLE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <circle cx="10" cy="10" r="7.2" stroke="currentColor" stroke-width="1.7"/>
  <circle cx="10" cy="10" r="3.2" fill="currentColor"/>
</svg>"#;

pub const DOT_FILLED: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 12 12" fill="none">
  <circle cx="6" cy="6" r="4.5" fill="currentColor"/>
</svg>"#;

pub const LIBRARY: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="2.5" y="3.5" width="15" height="13" rx="2.5" stroke="currentColor" stroke-width="1.6"/>
  <path d="M8.3 7.3L13.2 10L8.3 12.7V7.3Z" fill="currentColor"/>
</svg>"#;

pub const SETTINGS: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M3 6H17M3 14H17" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
  <circle cx="7.5" cy="6" r="2.2" fill="currentColor"/>
  <circle cx="13" cy="14" r="2.2" fill="currentColor"/>
</svg>"#;

pub const TERMINAL: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="2.5" y="3.5" width="15" height="13" rx="2.5" stroke="currentColor" stroke-width="1.6"/>
  <path d="M6.2 7.8L9.2 10.2L6.2 12.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M11 12.6H13.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
</svg>"#;

pub const PLAY: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M6 4.5L15.5 10L6 15.5V4.5Z" fill="currentColor"/>
</svg>"#;

pub const PAUSE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="5" y="4.2" width="3.4" height="11.6" rx="1" fill="currentColor"/>
  <rect x="11.6" y="4.2" width="3.4" height="11.6" rx="1" fill="currentColor"/>
</svg>"#;

pub const STOP: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="4.8" y="4.8" width="10.4" height="10.4" rx="2" fill="currentColor"/>
</svg>"#;

pub const CAMERA: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M3 6.8C3 5.8 3.8 5 4.8 5H6.5L7.6 3.5H12.4L13.5 5H15.2C16.2 5 17 5.8 17 6.8V14.2C17 15.2 16.2 16 15.2 16H4.8C3.8 16 3 15.2 3 14.2V6.8Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
  <circle cx="10" cy="10.5" r="2.7" stroke="currentColor" stroke-width="1.5"/>
</svg>"#;

pub const BOOKMARK: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M5.5 3.8C5.5 3.25 5.95 2.8 6.5 2.8H13.5C14.05 2.8 14.5 3.25 14.5 3.8V16.8L10 13.6L5.5 16.8V3.8Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
</svg>"#;

pub const TRASH: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M4 5.5H16M8 5.5V3.8C8 3.36 8.36 3 8.8 3H11.2C11.64 3 12 3.36 12 3.8V5.5M5.8 5.5L6.5 15.8C6.55 16.47 7.11 17 7.78 17H12.22C12.89 17 13.45 16.47 13.5 15.8L14.2 5.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

pub const FOLDER: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M2.8 5.5C2.8 4.67 3.47 4 4.3 4H7.6L9.3 5.8H15.7C16.53 5.8 17.2 6.47 17.2 7.3V14.5C17.2 15.33 16.53 16 15.7 16H4.3C3.47 16 2.8 15.33 2.8 14.5V5.5Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
</svg>"#;

pub const COPY: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="7" y="7" width="9.5" height="9.5" rx="2" stroke="currentColor" stroke-width="1.5"/>
  <path d="M13 7V5.2C13 4.26 12.24 3.5 11.3 3.5H5.2C4.26 3.5 3.5 4.26 3.5 5.2V11.3C3.5 12.24 4.26 13 5.2 13H7" stroke="currentColor" stroke-width="1.5"/>
</svg>"#;

pub const SCISSORS: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <circle cx="5.8" cy="6.2" r="2.2" stroke="currentColor" stroke-width="1.5"/>
  <circle cx="5.8" cy="13.8" r="2.2" stroke="currentColor" stroke-width="1.5"/>
  <path d="M7.6 7.4L16 14M7.6 12.6L16 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
</svg>"#;

pub const EDIT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M13.5 3.5L16.5 6.5L7.2 15.8L3.5 16.5L4.2 12.8L13.5 3.5Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
</svg>"#;

pub const SEARCH: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <circle cx="9" cy="9" r="5.5" stroke="currentColor" stroke-width="1.6"/>
  <path d="M13.2 13.2L16.8 16.8" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/>
</svg>"#;

pub const SHIELD: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M10 2.6L15.8 5V9.6C15.8 13.3 13.3 16.2 10 17.4C6.7 16.2 4.2 13.3 4.2 9.6V5L10 2.6Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
  <path d="M7.6 10.1L9.3 11.8L12.7 8.4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

pub const ARROW_RIGHT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M4.5 10H15.5M11.5 6L15.5 10L11.5 14" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

pub const REFRESH: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M15.8 10A5.8 5.8 0 1 1 14.1 5.9M15.8 4.2V6.8H13.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

pub const SUN: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <circle cx="10" cy="10" r="3.3" stroke="currentColor" stroke-width="1.5"/>
  <path d="M10 2.8V4.5M10 15.5V17.2M2.8 10H4.5M15.5 10H17.2M4.9 4.9L6.1 6.1M13.9 13.9L15.1 15.1M15.1 4.9L13.9 6.1M6.1 13.9L4.9 15.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
</svg>"#;

pub const MOON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M15.4 12.8A6.2 6.2 0 0 1 7.2 4.6A6.4 6.4 0 1 0 15.4 12.8Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
</svg>"#;

pub const GRID: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <rect x="3" y="3" width="5.8" height="5.8" rx="1.4" stroke="currentColor" stroke-width="1.5"/>
  <rect x="11.2" y="3" width="5.8" height="5.8" rx="1.4" stroke="currentColor" stroke-width="1.5"/>
  <rect x="3" y="11.2" width="5.8" height="5.8" rx="1.4" stroke="currentColor" stroke-width="1.5"/>
  <rect x="11.2" y="11.2" width="5.8" height="5.8" rx="1.4" stroke="currentColor" stroke-width="1.5"/>
</svg>"#;

pub const LIST: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M3.5 5.5H16.5M3.5 10H16.5M3.5 14.5H16.5" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/>
</svg>"#;

pub const X_CLOSE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M5.5 5.5L14.5 14.5M14.5 5.5L5.5 14.5" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/>
</svg>"#;

pub const CHECK: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none">
  <path d="M4.5 10.5L8.2 14.2L15.8 6.2" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;
