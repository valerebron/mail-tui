mod list;
mod detail;
mod settings;
mod popup;
mod styles;

pub use list::ListView;
pub use detail::DetailView;
pub use settings::{SettingsView, Field};
pub use popup::ConfirmPopup;
pub use styles::*;

use crate::mail::Mail;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    List,
    Detail,
    Settings,
    ConfirmDelete,
    ConfirmArchive,
}

pub struct App {
    pub mode: Mode,
    pub list: ListView,
    pub detail: DetailView,
    pub settings: SettingsView,
    pub confirm: ConfirmPopup,
    pub mails: Vec<Mail>,
    pub config: crate::mail::Config,
    pub client: Option<crate::mail::Client>,
    pub pending_uid: Option<u32>,
    pub last_refresh: std::time::Instant,
    pub connecting: bool,
}

impl App {
    pub fn new() -> Self {
        let config = crate::mail::Config::load().unwrap_or_default();
        let has_valid_config = !config.email.is_empty() && !config.password.is_empty();
        Self {
            mode: Mode::List,
            list: ListView::new(),
            detail: DetailView::new(),
            settings: SettingsView::new(),
            confirm: ConfirmPopup::new(),
            mails: Vec::new(),
            config,
            client: None,
            pending_uid: None,
            last_refresh: std::time::Instant::now(),
            connecting: has_valid_config,
        }
    }

    pub async fn try_auto_connect(&mut self) -> anyhow::Result<()> {
        if self.config.email.is_empty() || self.config.password.is_empty() {
            return Ok(());
        }
        if self.connecting {
            return Ok(());
        }
        self.connecting = true;

        let config = self.config.clone();
        let mut client = crate::mail::Client::new(config.clone());
        client.connect()?;
        client.select_inbox()?;
        let mails = client.list_mails(50)?;

        self.client = Some(client);
        self.mails = mails;
        self.list.set_mails(&self.mails);
        self.connecting = false;
        self.last_refresh = std::time::Instant::now();
        Ok(())
    }

    pub async fn refresh_mails(&mut self) -> anyhow::Result<()> {
        if self.client.is_none() {
            return self.try_auto_connect().await;
        }

        let client = self.client.as_mut().unwrap();
        client.select_inbox()?;
        let mails = client.list_mails(50)?;

        self.mails = mails;
        self.list.set_mails(&self.mails);
        self.last_refresh = std::time::Instant::now();
        Ok(())
    }

    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed().as_secs() >= 5 && !self.connecting
    }

    pub fn list_move_up(&mut self) {
        self.list.move_up();
    }

    pub fn list_move_down(&mut self) {
        self.list.move_down();
    }

    pub fn selected_mail(&self) -> Option<&Mail> {
        self.list.selected_mail(&self.mails)
    }

    pub fn open_detail(&mut self, mail: Mail) {
        self.detail.set_mail(mail);
        self.mode = Mode::Detail;
    }

    pub fn detail_scroll_up(&mut self) {
        self.detail.scroll_up();
    }

    pub fn detail_scroll_down(&mut self) {
        self.detail.scroll_down();
    }

    pub fn populate_settings(&mut self) {
        self.settings.set_values(
            &self.config.email,
            &self.config.password,
            &self.config.imap_host,
            self.config.imap_port,
            &self.config.smtp_host,
            self.config.smtp_port,
            self.config.use_tls,
        );
        self.settings.set_detected(!self.config.imap_host.is_empty());
    }

    pub async fn try_connect(&mut self) -> anyhow::Result<()> {
        let (email, password, imap_host, imap_port, smtp_host, smtp_port, use_tls) = self.settings.values();

        if email.is_empty() || password.is_empty() {
            return Err(anyhow::anyhow!("Email and password required"));
        }

        let config = crate::mail::Config {
            email,
            password,
            imap_host,
            imap_port,
            smtp_host,
            smtp_port,
            use_tls,
        };

        let mut client = crate::mail::Client::new(config.clone());
        client.connect()?;
        client.select_inbox()?;
        let mails = client.list_mails(50)?;

        // Save config on successful connection
        config.save()?;

        self.client = Some(client);
        self.config = config;
        self.mails = mails;
        self.list.set_mails(&self.mails);
        self.settings.set_error("".into());
        self.last_refresh = std::time::Instant::now();
        Ok(())
    }

    pub fn settings_move_up(&mut self) {
        self.settings.move_up();
    }

    pub fn settings_move_down(&mut self) {
        self.settings.move_down();
    }

    pub fn settings_cursor(&self) -> Field {
        self.settings.cursor()
    }

    pub fn settings_toggle_tls(&mut self) {
        self.settings.toggle_tls();
    }

    pub fn settings_set_error(&mut self, err: String) {
        self.settings.set_error(err);
    }

    pub fn confirm_move_left(&mut self) {
        self.confirm.move_left();
    }

    pub fn confirm_move_right(&mut self) {
        self.confirm.move_right();
    }

    pub fn confirm_selected(&self) -> usize {
        self.confirm.selected()
    }

    pub async fn delete_mail(&mut self, uid: u32) -> anyhow::Result<()> {
        if let Some(client) = &mut self.client {
            client.delete_mail(uid)?;
            self.mails.retain(|m| m.uid != uid);
            self.list.set_mails(&self.mails);
        }
        Ok(())
    }

    pub async fn archive_mail(&mut self, uid: u32) -> anyhow::Result<()> {
        if let Some(client) = &mut self.client {
            client.archive_mail(uid)?;
            self.mails.retain(|m| m.uid != uid);
            self.list.set_mails(&self.mails);
        }
        Ok(())
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    match app.mode {
        Mode::List => {
            app.list.set_area(area);
            f.render_widget(&app.list, area);
        }
        Mode::Detail => {
            app.detail.set_area(area);
            f.render_widget(&app.detail, area);
        }
        Mode::Settings => {
            app.settings.set_area(area);
            f.render_widget(&app.settings, area);
        }
        Mode::ConfirmDelete => {
            app.list.set_area(area);
            f.render_widget(&app.list, area);
            app.confirm.set_type(popup::ConfirmType::Delete);
            app.confirm.set_area(area);
            f.render_widget(Clear, area);
            f.render_widget(&app.confirm, area);
        }
        Mode::ConfirmArchive => {
            app.list.set_area(area);
            f.render_widget(&app.list, area);
            app.confirm.set_type(popup::ConfirmType::Archive);
            app.confirm.set_area(area);
            f.render_widget(Clear, area);
            f.render_widget(&app.confirm, area);
        }
    }
}