use crate::ui::styles::*;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmType {
    Delete,
    Archive,
}

pub struct ConfirmPopup {
    confirm_type: ConfirmType,
    selected: usize, // 0 = No, 1 = Yes
    area: Rect,
}

impl ConfirmPopup {
    pub fn new() -> Self {
        Self {
            confirm_type: ConfirmType::Delete,
            selected: 0,
            area: Rect::default(),
        }
    }

    pub fn set_type(&mut self, t: ConfirmType) {
        self.confirm_type = t;
        self.selected = 0;
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected < 1 {
            self.selected += 1;
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}

impl ratatui::widgets::Widget for &ConfirmPopup {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = match self.confirm_type {
            ConfirmType::Delete => "Confirm Delete",
            ConfirmType::Archive => "Confirm Archive",
        };

        let message = match self.confirm_type {
            ConfirmType::Delete => "Delete this mail? This action cannot be undone.",
            ConfirmType::Archive => "Archive this mail?",
        };

        let mut lines = Vec::new();
        lines.push(Line::styled(title, Style::new().fg(POPUP_TEXT_COLOR).add_modifier(Modifier::BOLD)));
        lines.push(Line::raw(""));
        lines.push(Line::styled(message, Style::new().fg(POPUP_TEXT_COLOR)));
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));

        // Buttons
        let no_style = if self.selected == 0 {
            POPUP_BUTTON_SELECTED_STYLE()
        } else {
            POPUP_BUTTON_STYLE()
        };
        let yes_style = if self.selected == 1 {
            POPUP_BUTTON_SELECTED_STYLE()
        } else {
            POPUP_BUTTON_STYLE()
        };

        let button_line = Line::from(vec![
            Span::styled("       ", Style::default()),
            Span::styled(" No ", no_style),
            Span::styled("       ", Style::default()),
            Span::styled(" Yes ", yes_style),
        ]);
        lines.push(button_line);
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            " ←/→ select  Enter confirm  Esc cancel ",
            HELP_STYLE,
        ));

        let popup_width = (area.width as usize).min(60).max(40);
        let popup_height = lines.len() as u16 + 2;

        let popup_area = Rect::new(
            (area.width.saturating_sub(popup_width as u16)) / 2,
            (area.height.saturating_sub(popup_height)) / 2,
            popup_width as u16,
            popup_height,
        );

        Clear.render(popup_area, buf);

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(POPUP_BORDER_COLOR))
                .style(Style::new().bg(POPUP_BG_COLOR))
                .padding(ratatui::widgets::Padding::uniform(1)));

        paragraph.render(popup_area, buf);
    }
}

fn POPUP_BUTTON_STYLE() -> Style {
    Style::new()
        .fg(POPUP_BG_COLOR)
        .bg(POPUP_BUTTON_COLOR)
        .add_modifier(Modifier::BOLD)
}

fn POPUP_BUTTON_SELECTED_STYLE() -> Style {
    Style::new()
        .fg(POPUP_BG_COLOR)
        .bg(POPUP_BUTTON_SELECTED_COLOR)
        .add_modifier(Modifier::BOLD)
}

use ratatui::style::{Color, Modifier, Style};