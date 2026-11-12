use crate::ui::styles::*;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Email,
    Password,
    IMAPHost,
    IMAPPort,
    SMTPHost,
    SMTPPort,
    UseTLS,
    Save,
}

impl Field {
    pub fn all() -> &'static [Field] {
        &[
            Field::Email,
            Field::Password,
            Field::IMAPHost,
            Field::IMAPPort,
            Field::SMTPHost,
            Field::SMTPPort,
            Field::UseTLS,
            Field::Save,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Field::Email => "Email:",
            Field::Password => "Password:",
            Field::IMAPHost => "IMAP Host:",
            Field::IMAPPort => "IMAP Port:",
            Field::SMTPHost => "SMTP Host:",
            Field::SMTPPort => "SMTP Port:",
            Field::UseTLS => "Use TLS:",
            Field::Save => "Save & Connect",
        }
    }

    pub fn is_editable(&self) -> bool {
        matches!(self, Field::Email | Field::Password | Field::IMAPHost | Field::IMAPPort | Field::SMTPHost | Field::SMTPPort)
    }
}

pub struct SettingsView {
    fields: [String; 8],
    cursor: usize,
    area: Rect,
    detected: bool,
    error: String,
    editing: bool,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            fields: Default::default(),
            cursor: 0,
            area: Rect::default(),
            detected: false,
            error: String::new(),
            editing: false,
        }
    }

    pub fn set_values(&mut self, email: &str, password: &str, imap_host: &str, imap_port: u16, smtp_host: &str, smtp_port: u16, use_tls: bool) {
        self.fields[0] = email.to_string();
        self.fields[1] = password.to_string();
        self.fields[2] = imap_host.to_string();
        self.fields[3] = imap_port.to_string();
        self.fields[4] = smtp_host.to_string();
        self.fields[5] = smtp_port.to_string();
        self.fields[6] = use_tls.to_string();
        self.fields[7] = String::new(); // Save button
    }

    pub fn values(&self) -> (String, String, String, u16, String, u16, bool) {
        let email = self.fields[0].clone();
        let password = self.fields[1].clone();
        let imap_host = self.fields[2].clone();
        let imap_port = self.fields[3].parse().unwrap_or(993);
        let smtp_host = self.fields[4].clone();
        let smtp_port = self.fields[5].parse().unwrap_or(587);
        let use_tls = self.fields[6] == "true";
        (email, password, imap_host, imap_port, smtp_host, smtp_port, use_tls)
    }

    pub fn set_detected(&mut self, detected: bool) {
        self.detected = detected;
    }

    pub fn set_error(&mut self, err: String) {
        self.error = err;
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn move_up(&mut self) {
        if !self.editing && self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.editing && self.cursor < 7 {
            self.cursor += 1;
        }
    }

    pub fn cursor(&self) -> Field {
        match self.cursor {
            0 => Field::Email,
            1 => Field::Password,
            2 => Field::IMAPHost,
            3 => Field::IMAPPort,
            4 => Field::SMTPHost,
            5 => Field::SMTPPort,
            6 => Field::UseTLS,
            _ => Field::Save,
        }
    }

    pub fn start_editing(&mut self) {
        if self.cursor < 7 && self.cursor().is_editable() {
            self.editing = true;
        }
    }

    pub fn stop_editing(&mut self) {
        self.editing = false;
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn handle_char(&mut self, c: char) {
        if self.editing && self.cursor < 7 {
            self.fields[self.cursor].push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.editing && self.cursor < 7 && !self.fields[self.cursor].is_empty() {
            self.fields[self.cursor].pop();
        }
    }

    pub fn toggle_tls(&mut self) {
        if !self.editing && self.cursor == 6 {
            self.fields[6] = if self.fields[6] == "true" { "false" } else { "true" }.to_string();
        }
    }
}

impl ratatui::widgets::Widget for &SettingsView {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::styled(" ⚙ Settings ", SETTINGS_LABEL_STYLE()));

        for (i, field) in Field::all().iter().enumerate() {
            let is_cursor = i == self.cursor;
            let prefix = if is_cursor {
                Span::styled("▸ ", SETTINGS_CURSOR_STYLE())
            } else {
                Span::raw("  ")
            };

            let label = Span::styled(field.label(), SETTINGS_LABEL_STYLE());
            let mut val = self.fields[i].clone();

            if i == 1 && !val.is_empty() && !self.editing {
                val = "*".repeat(val.len());
            }

            let val_style = SETTINGS_VALUE_STYLE();
            let val_span = if is_cursor && self.editing {
                Span::styled(val + "▌", val_style) // blinking cursor
            } else if is_cursor {
                Span::styled(val + "█", val_style)
            } else {
                Span::styled(val, val_style)
            };

            let mut line = Line::from(vec![prefix, label, Span::raw(" "), val_span]);

            if self.detected && matches!(field, Field::IMAPHost | Field::IMAPPort | Field::SMTPHost | Field::SMTPPort) {
                line.spans.push(Span::raw("  "));
                line.spans.push(Span::styled("(auto-detected)", SETTINGS_DETECTED_STYLE()));
            }

            lines.push(line);
        }

        if !self.error.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                format!("  Error: {}", self.error),
                Style::new().fg(ERROR_COLOR).add_modifier(Modifier::BOLD),
            ));
        }

        lines.push(Line::raw(""));
        let help = if self.editing {
            " Type to edit  Enter/Esc confirm  Ctrl+c quit "
        } else {
            " ↑/↓ nav  Enter edit  Esc back  Ctrl+c quit "
        };
        lines.push(Line::styled(help, HELP_STYLE));

        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(DETAIL_BORDER_COLOR))
                .padding(ratatui::widgets::Padding::uniform(1)));

        paragraph.render(area, buf);
    }
}

fn SETTINGS_LABEL_STYLE() -> Style {
    Style::new().fg(SETTINGS_LABEL_COLOR).add_modifier(Modifier::BOLD)
}

fn SETTINGS_VALUE_STYLE() -> Style {
    Style::new().fg(SETTINGS_VALUE_COLOR)
}

fn SETTINGS_CURSOR_STYLE() -> Style {
    Style::new().fg(SETTINGS_CURSOR_COLOR).add_modifier(Modifier::BOLD)
}

fn SETTINGS_DETECTED_STYLE() -> Style {
    Style::new().fg(SETTINGS_DETECTED_COLOR).add_modifier(Modifier::ITALIC)
}

use ratatui::style::{Color, Modifier, Style};