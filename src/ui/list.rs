use crate::mail::Mail;
use crate::ui::styles::*;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub struct ListView {
    mails: Vec<Mail>,
    state: ListState,
    area: Rect,
}

impl ListView {
    pub fn new() -> Self {
        Self {
            mails: Vec::new(),
            state: ListState::default(),
            area: Rect::default(),
        }
    }

    pub fn set_mails(&mut self, mails: &[Mail]) {
        self.mails = mails.to_vec();
        if self.state.selected().unwrap_or(0) >= self.mails.len() && !self.mails.is_empty() {
            self.state.select(Some(self.mails.len() - 1));
        } else if self.mails.is_empty() {
            self.state.select(None);
        }
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected > 0 {
                self.state.select(Some(selected - 1));
            }
        } else if !self.mails.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected + 1 < self.mails.len() {
                self.state.select(Some(selected + 1));
            }
        } else if !self.mails.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn selected_mail<'a>(&self, mails: &'a [Mail]) -> Option<&'a Mail> {
        self.state.selected().and_then(|i| mails.get(i))
    }

    fn render_item<'a>(&self, mail: &'a Mail, is_selected: bool) -> ListItem<'a> {
        let style = if is_selected { SELECTED_ITEM_STYLE } else { ITEM_STYLE };

        let from = Span::styled(&mail.from, FROM_STYLE);
        let subject_style = if mail.is_read { SUBJECT_STYLE } else { UNREAD_STYLE };
        let subject = Span::styled(&mail.subject, subject_style);

        let line = Line::from(vec![
            Span::styled("  ", style),
            from,
            Span::styled("  ", style),
            subject,
        ]);

        ListItem::new(line).style(style)
    }
}

impl ratatui::widgets::Widget for &ListView {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if self.mails.is_empty() {
            let empty = Paragraph::new("No mails").style(HELP_STYLE).block(
                Block::default()
                    .title(" 📧 tui-mail ")
                    .borders(Borders::ALL)
                    .border_style(TITLE_STYLE),
            );
            empty.render(area, buf);
            return;
        }

        // Title with counts
        let unread_count = self.mails.iter().filter(|m| !m.is_read).count();
        let title = format!(" 📧 tui-mail  ({} mails, {} unread) ", self.mails.len(), unread_count);

        let items: Vec<ListItem> = self.mails
            .iter()
            .enumerate()
            .map(|(i, m)| self.render_item(m, Some(i) == self.state.selected()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL).border_style(TITLE_STYLE))
            .highlight_style(SELECTED_ITEM_STYLE);

        // Help text
        let help = Paragraph::new(" ↑/↓ nav  Enter open  d delete  a archive  s settings  Esc/q quit ")
            .style(HELP_STYLE);

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(5),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(area);

        ratatui::widgets::StatefulWidget::render(list, chunks[0], buf, &mut self.state.clone());
        help.render(chunks[1], buf);
    }
}