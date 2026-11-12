mod mail;
mod ui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;
use tokio::time::interval;
use ui::{App, Mode, Field};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let mut app = App::new();

    // Auto-connect if config is valid
    let _ = app.try_auto_connect().await;

    let mut refresh_interval = interval(Duration::from_secs(5));
    refresh_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check for auto-refresh
        if app.should_refresh() {
            let _ = app.refresh_mails().await;
            continue;
        }

        tokio::select! {
            _ = refresh_interval.tick() => {
                if app.should_refresh() {
                    let _ = app.refresh_mails().await;
                }
            }
            event_result = tokio::task::spawn_blocking(|| event::read()) => {
                match event_result {
                    Ok(Ok(Event::Key(key))) => {
                        if handle_key(&mut app, key).await? {
                            break;
                        }
                    }
                    Ok(Err(e)) => return Err(e.into()),
                    Err(_) => {}
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

async fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    match app.mode {
        Mode::List => handle_list_keys(app, key).await,
        Mode::Detail => handle_detail_keys(app, key).await,
        Mode::Settings => handle_settings_keys(app, key).await,
        Mode::ConfirmDelete => handle_confirm_delete(app, key).await,
        Mode::ConfirmArchive => handle_confirm_archive(app, key).await,
    }
}

async fn handle_list_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

        KeyCode::Up | KeyCode::Char('k') => app.list_move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.list_move_down(),

        KeyCode::Enter => {
            if let Some(mail) = app.selected_mail().cloned() {
                app.open_detail(mail);
            }
        }

        KeyCode::Char('d') => {
            if app.selected_mail().is_some() {
                app.mode = Mode::ConfirmDelete;
            }
        }

        KeyCode::Char('a') => {
            if app.selected_mail().is_some() {
                app.mode = Mode::ConfirmArchive;
            }
        }

        KeyCode::Char('s') => {
            app.populate_settings();
            app.mode = Mode::Settings;
        }

        KeyCode::Char('r') => {
            let _ = app.refresh_mails().await;
        }

        _ => {}
    }
    Ok(false)
}

async fn handle_detail_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => app.mode = Mode::List,

        KeyCode::Left | KeyCode::Char('h') => {
            app.list_move_up();
            if let Some(mail) = app.selected_mail().cloned() {
                app.open_detail(mail);
            }
        }

        KeyCode::Right | KeyCode::Char('l') => {
            app.list_move_down();
            if let Some(mail) = app.selected_mail().cloned() {
                app.open_detail(mail);
            }
        }

        KeyCode::Up | KeyCode::Char('k') => app.detail_scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => app.detail_scroll_down(),

        KeyCode::Char('d') => {
            if app.selected_mail().is_some() {
                app.mode = Mode::ConfirmDelete;
            }
        }

        KeyCode::Char('a') => {
            if app.selected_mail().is_some() {
                app.mode = Mode::ConfirmArchive;
            }
        }

        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Char('r') => {
            let _ = app.refresh_mails().await;
        }

        _ => {}
    }
    Ok(false)
}

async fn handle_settings_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    // If editing, handle text input
    if app.settings.is_editing() {
        match key.code {
            KeyCode::Esc => {
                app.settings.stop_editing();
            }
            KeyCode::Enter => {
                app.settings.stop_editing();
            }
            KeyCode::Backspace => {
                app.settings.handle_backspace();
            }
            KeyCode::Char(c) => {
                app.settings.handle_char(c);
            }
            _ => {}
        }
        return Ok(false);
    }

    // Not editing - normal navigation
    match key.code {
        KeyCode::Esc => app.mode = Mode::List,
        KeyCode::Up | KeyCode::Char('k') => app.settings_move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.settings_move_down(),
        KeyCode::Enter => {
            if app.settings_cursor() == Field::Save {
                if let Err(e) = app.try_connect().await {
                    app.settings_set_error(format!("{}", e));
                } else {
                    app.mode = Mode::List;
                }
            } else if app.settings_cursor() == Field::UseTLS {
                app.settings_toggle_tls();
            } else {
                app.settings.start_editing();
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        _ => {}
    }
    Ok(false)
}

async fn handle_confirm_delete(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => app.mode = Mode::List,
        KeyCode::Left | KeyCode::Char('h') => app.confirm_move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.confirm_move_right(),
        KeyCode::Enter => {
            if app.confirm_selected() == 1 {
                if let Some(mail) = app.selected_mail() {
                    let _ = app.delete_mail(mail.uid).await;
                }
            }
            app.mode = Mode::List;
        }
        _ => {}
    }
    Ok(false)
}

async fn handle_confirm_archive(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => app.mode = Mode::List,
        KeyCode::Left | KeyCode::Char('h') => app.confirm_move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.confirm_move_right(),
        KeyCode::Enter => {
            if app.confirm_selected() == 1 {
                if let Some(mail) = app.selected_mail() {
                    let _ = app.archive_mail(mail.uid).await;
                }
            }
            app.mode = Mode::List;
        }
        _ => {}
    }
    Ok(false)
}