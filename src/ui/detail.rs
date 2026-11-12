use crate::mail::Mail;
use crate::ui::styles::*;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct DetailView {
    mail: Option<Mail>,
    scroll: usize,
    area: Rect,
}

impl DetailView {
    pub fn new() -> Self {
        Self {
            mail: None,
            scroll: 0,
            area: Rect::default(),
        }
    }

    pub fn set_mail(&mut self, mail: Mail) {
        self.mail = Some(mail);
        self.scroll = 0;
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if let Some(mail) = &self.mail {
            let lines = wrap_text(&mail.body, self.content_width());
            let max_scroll = lines.len().saturating_sub(self.content_height());
            if self.scroll < max_scroll {
                self.scroll += 1;
            }
        }
    }

    fn content_width(&self) -> usize {
        self.area.width.saturating_sub(6) as usize
    }

    fn content_height(&self) -> usize {
        self.area.height.saturating_sub(10) as usize
    }
}

impl ratatui::widgets::Widget for &DetailView {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let Some(mail) = &self.mail else { return };

        let content_w = self.content_width().max(10);
        let visible_h = self.content_height().max(1);

        // Build content
        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled("From: ", DetailFromStyle()),
            Span::raw(&mail.from),
            Span::raw("  "),
            Span::styled("Subject: ", DetailSubjectStyle()),
            Span::raw(&mail.subject),
            Span::raw("  "),
            Span::styled("Date: ", DetailDateStyle()),
            Span::raw(&mail.date),
        ]));

        if mail.is_flagged {
            lines.push(Line::from(vec![
                Span::styled("  ⚑ Flagged ", FLAGGED_STYLE),
            ]));
        }
        if !mail.is_read {
            lines.push(Line::from(vec![
                Span::styled("  ● Unread ", DetailFromStyle()),
            ]));
        }

        lines.push(Line::from(Span::styled("─".repeat(content_w), DETAIL_BORDER_COLOR)));
        lines.push(Line::raw(""));

        // Body with wrap
        let body_lines = wrap_text(&mail.body, content_w);
        for line in &body_lines[self.scroll..] {
            lines.push(Line::raw(line.clone()));
            if lines.len() > visible_h + 8 { break; }
        }

        // Scroll indicator
        if body_lines.len() > visible_h {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                format!("[scroll {}/{}]", self.scroll + 1, body_lines.len() - visible_h + 1),
                HELP_STYLE,
            ));
        }

        lines.push(Line::raw(""));
        lines.push(Line::styled(
            " ←/→ prev/next mail  ↑/↓ scroll  d delete  a archive  Esc back  Ctrl+c quit ",
            HELP_STYLE,
        ));

        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(DETAIL_BORDER_COLOR))
                .padding(ratatui::widgets::Padding::uniform(1)))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

fn DetailFromStyle() -> Style {
    Style::new().fg(FROM_STYLE.fg.unwrap_or(Color::White)).add_modifier(Modifier::BOLD)
}

fn DetailSubjectStyle() -> Style {
    Style::new().fg(SUBJECT_STYLE.fg.unwrap_or(Color::White)).add_modifier(Modifier::BOLD)
}

fn DetailDateStyle() -> Style {
    Style::new().fg(DATE_STYLE.fg.unwrap_or(Color::White))
}

use ratatui::style::{Color, Modifier, Style};

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut result = Vec::new();
    for para in text.split('\n') {
        if para.is_empty() {
            result.push(String::new());
            continue;
        }
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in words {
            if line.is_empty() {
                line.push_str(word);
            } else if line.len() + 1 + word.len() <= width {
                line.push(' ');
                line.push_str(word);
            } else {
                result.push(line);
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            result.push(line);
        }
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}