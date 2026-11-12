use ratatui::style::{Color, Modifier, Style};

pub const TITLE_STYLE: Style = Style::new()
    .fg(Color::Rgb(0xFA, 0xFA, 0xFA))
    .bg(Color::Rgb(0x7D, 0x56, 0xF4))
    .add_modifier(Modifier::BOLD);

pub const ITEM_STYLE: Style = Style::new();
pub const SELECTED_ITEM_STYLE: Style = Style::new()
    .fg(Color::Rgb(0x7D, 0x56, 0xF4))
    .bg(Color::Rgb(0xED, 0xED, 0xED))
    .add_modifier(Modifier::BOLD);

pub const UNREAD_STYLE: Style = Style::new()
    .fg(Color::Rgb(0x04, 0xB5, 0x75))
    .add_modifier(Modifier::BOLD);

pub const HELP_STYLE: Style = Style::new()
    .fg(Color::Rgb(0x62, 0x62, 0x62))
    .add_modifier(Modifier::ITALIC);

pub const FROM_STYLE: Style = Style::new().fg(Color::Rgb(0x04, 0xB5, 0x75));
pub const SUBJECT_STYLE: Style = Style::new();
pub const DATE_STYLE: Style = Style::new().fg(Color::Rgb(0x62, 0x62, 0x62));
pub const FLAGGED_STYLE: Style = Style::new()
    .fg(Color::Rgb(0xFF, 0x9F, 0x1C))
    .add_modifier(Modifier::BOLD);

pub const DETAIL_BORDER_COLOR: Color = Color::Rgb(0x7D, 0x56, 0xF4);
pub const POPUP_BORDER_COLOR: Color = Color::Rgb(0xFF, 0x44, 0x44);
pub const POPUP_BG_COLOR: Color = Color::Rgb(0x1A, 0x1A, 0x2E);
pub const POPUP_TEXT_COLOR: Color = Color::Rgb(0xFA, 0xFA, 0xFA);
pub const POPUP_BUTTON_COLOR: Color = Color::Rgb(0x04, 0xB5, 0x75);
pub const POPUP_BUTTON_SELECTED_COLOR: Color = Color::Rgb(0xFF, 0x44, 0x44);
pub const SETTINGS_LABEL_COLOR: Color = Color::Rgb(0x7D, 0x56, 0xF4);
pub const SETTINGS_VALUE_COLOR: Color = Color::Rgb(0xFA, 0xFA, 0xFA);
pub const SETTINGS_CURSOR_COLOR: Color = Color::Rgb(0x04, 0xB5, 0x75);
pub const SETTINGS_DETECTED_COLOR: Color = Color::Rgb(0x62, 0x62, 0x62);
pub const ERROR_COLOR: Color = Color::Rgb(0xFF, 0x44, 0x44);