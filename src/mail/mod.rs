use anyhow::Result;
use chrono;
use dirs;
use html2text;
use imap::Session;
use native_tls::TlsConnector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub email: String,
    pub password: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub use_tls: bool,
}

impl Config {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("tui-mail").join("config.json"))
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("Config directory not found"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub uid: u32,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub body: String,
    pub flags: Vec<String>,
    pub is_read: bool,
    pub is_flagged: bool,
}

pub struct Client {
    session: Option<Session<TlsStream>>,
    config: Config,
}

type TlsStream = native_tls::TlsStream<TcpStream>;

impl Client {
    pub fn new(config: Config) -> Self {
        Self { session: None, config }
    }

    pub fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.imap_host, self.config.imap_port);
        let stream = TcpStream::connect(&addr)?;
        let connector = TlsConnector::new()?;
        let tls_stream = connector.connect(&self.config.imap_host, stream)?;
        let client = imap::Client::new(tls_stream);
        let session = client.login(&self.config.email, &self.config.password)
            .map_err(|e| anyhow::anyhow!("Login failed: {}", e.0))?;
        self.session = Some(session);
        Ok(())
    }

    pub fn select_inbox(&mut self) -> Result<()> {
        let session = self.session.as_mut().ok_or_else(|| anyhow::anyhow!("Not connected"))?;
        session.select("INBOX")?;
        Ok(())
    }

    pub fn list_mails(&mut self, limit: usize) -> Result<Vec<Mail>> {
        let session = self.session.as_mut().ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        // First get the mailbox info to know how many messages
        let mailbox = session.examine("INBOX")?;
        let total = mailbox.exists;

        if total == 0 {
            return Ok(Vec::new());
        }

        // Use UID FETCH with range - try different approaches
        // Calculate UID range (we need to get UIDs first or use sequence)
        // Let's try UID FETCH with *:* or recent UIDs
        let fetch_result = session.uid_fetch("1:*", "(FLAGS ENVELOPE BODY[TEXT])")
            .or_else(|_| session.uid_fetch("*:*", "(FLAGS ENVELOPE BODY[TEXT])"))
            .or_else(|_| session.fetch("1:*", "(UID FLAGS ENVELOPE BODY[TEXT])"))
            .or_else(|_| session.fetch("*", "(UID FLAGS ENVELOPE BODY[TEXT])"))
            .or_else(|_| session.fetch("1:50", "(UID FLAGS ENVELOPE BODY[TEXT])"))?;

        let mut mails = Vec::new();

        for msg in fetch_result.iter().rev().take(limit) {
            let uid = msg.uid.unwrap_or(0);
            let flags: Vec<String> = msg.flags().iter().map(|f| format!("{:?}", f)).collect();
            let is_read = msg.flags().iter().any(|f| format!("{:?}", f) == "Seen");
            let is_flagged = msg.flags().iter().any(|f| format!("{:?}", f) == "Flagged");

            let envelope = msg.envelope();
            let from = envelope.as_ref().and_then(|e| e.from.as_ref()?.first())
                .map(|a| {
                    let mailbox = a.mailbox.as_ref().map(|m| String::from_utf8_lossy(m)).unwrap_or_default();
                    let host = a.host.as_ref().map(|h| String::from_utf8_lossy(h)).unwrap_or_default();
                    format!("{}@{}", mailbox, host)
                })
                .unwrap_or_default();
            let subject = envelope.as_ref().and_then(|e| e.subject.as_ref().map(|s| String::from_utf8_lossy(s).into_owned())).unwrap_or_default();
            let date = envelope.as_ref().and_then(|e| e.date.as_ref())
                .and_then(|d| chrono::DateTime::parse_from_rfc2822(&String::from_utf8_lossy(d)).ok())
                .map(|d| d.format("%d %b %H:%M").to_string())
                .unwrap_or_default();

            let body_bytes = msg.body().unwrap_or_default();
            let body_raw = String::from_utf8_lossy(body_bytes).into_owned();
            let body = html_to_text(&body_raw);

            mails.push(Mail {
                uid,
                from,
                subject,
                date,
                body,
                flags,
                is_read,
                is_flagged,
            });
        }

        Ok(mails)
    }

    pub fn delete_mail(&mut self, uid: u32) -> Result<()> {
        let session = self.session.as_mut().ok_or_else(|| anyhow::anyhow!("Not connected"))?;
        let seq = format!("UID {}", uid);
        session.store(&seq, "+FLAGS.SILENT (\\Deleted)")?;
        session.expunge()?;
        Ok(())
    }

    pub fn archive_mail(&mut self, uid: u32) -> Result<()> {
        let session = self.session.as_mut().ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        let archive_folders = ["[Gmail]/All Mail", "Archive", "Archives", "INBOX.Archive"];
        let seq = format!("UID {}", uid);

        for folder in archive_folders {
            if session.copy(&seq, folder).is_ok() {
                session.store(&seq, "+FLAGS.SILENT (\\Deleted)")?;
                session.expunge()?;
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("No archive folder found"))
    }
}

lazy_static::lazy_static! {
    static ref KNOWN_SERVERS: HashMap<&'static str, Config> = {
        let mut m = HashMap::new();
        m.insert("gmail.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "imap.gmail.com".into(),
            imap_port: 993,
            smtp_host: "smtp.gmail.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("outlook.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "outlook.office365.com".into(),
            imap_port: 993,
            smtp_host: "smtp.office365.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("hotmail.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "outlook.office365.com".into(),
            imap_port: 993,
            smtp_host: "smtp.office365.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("yahoo.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "imap.mail.yahoo.com".into(),
            imap_port: 993,
            smtp_host: "smtp.mail.yahoo.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("icloud.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "imap.mail.me.com".into(),
            imap_port: 993,
            smtp_host: "smtp.mail.me.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("protonmail.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "mail.protonmail.ch".into(),
            imap_port: 993,
            smtp_host: "mail.protonmail.ch".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m.insert("fastmail.com", Config {
            email: String::new(),
            password: String::new(),
            imap_host: "imap.fastmail.com".into(),
            imap_port: 993,
            smtp_host: "smtp.fastmail.com".into(),
            smtp_port: 587,
            use_tls: true,
        });
        m
    };
}

pub fn detect_config(email: &str) -> Option<Config> {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return None;
    }
    let domain = parts[1].to_lowercase();

    if let Some(mut cfg) = KNOWN_SERVERS.get(domain.as_str()).cloned() {
        cfg.email = email.to_string();
        return Some(cfg);
    }

    for (suffix, cfg) in KNOWN_SERVERS.iter() {
        if domain.ends_with(suffix) {
            let mut new_cfg = cfg.clone();
            new_cfg.email = email.to_string();
            return Some(new_cfg);
        }
    }

    Some(Config {
        email: email.to_string(),
        password: String::new(),
        imap_host: format!("imap.{}", domain),
        imap_port: 993,
        smtp_host: format!("smtp.{}", domain),
        smtp_port: 587,
        use_tls: true,
    })
}


impl Default for Config {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            imap_host: String::new(),
            imap_port: 993,
            smtp_host: String::new(),
            smtp_port: 587,
            use_tls: true,
        }
    }
}

fn html_to_text(html: &str) -> String {
    // Check if content looks like HTML
    let lower = html.to_lowercase();
    if lower.contains("<html") || lower.contains("<body") || lower.contains("<div") || lower.contains("<p>") || lower.contains("<br") {
        // Convert HTML to plain text
        html2text::from_read(html.as_bytes(), 80)
    } else {
        // Already plain text
        html.to_string()
    }
}