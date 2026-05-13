#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::AXIsProcessTrustedWithOptions;
use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use parking_lot::Mutex;
use plist::Value;
use rusqlite::{Connection, OpenFlags};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub struct MacosNotiSource {
    running: Arc<Mutex<bool>>,
}

impl Default for MacosNotiSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MacosNotiSource {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Kept for the permission UI. Accessibility is no longer required for
    /// notification capture (DB polling); this just reports the trust state.
    pub fn is_trusted(prompt: bool) -> bool {
        unsafe {
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = core_foundation::boolean::CFBoolean::from(prompt);
            let opts = CFDictionary::from_CFType_pairs(&[(key, value)]);
            AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef() as _)
        }
    }
}

fn db_path() -> Option<PathBuf> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let p = home.join("Library/Group Containers/group.com.apple.usernoted/db2/db");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

fn open_db() -> anyhow::Result<Connection> {
    let path = db_path().ok_or_else(|| anyhow::anyhow!("usernoted db not found"))?;
    let conn = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(conn)
}

fn current_max_rec_id(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(rec_id), 0) FROM record",
        [],
        |r| r.get(0),
    )
}

struct Row {
    rec_id: i64,
    app_id: String,
    title: String,
    body: String,
}

fn query_new(conn: &Connection, after: i64) -> rusqlite::Result<Vec<Row>> {
    let mut stmt = conn.prepare(
        "SELECT r.rec_id, COALESCE(a.identifier, ''), r.data
         FROM record r LEFT JOIN app a ON r.app_id = a.app_id
         WHERE r.rec_id > ?1
         ORDER BY r.rec_id",
    )?;
    let rows = stmt
        .query_map([after], |row| {
            let rec_id: i64 = row.get(0)?;
            let identifier: String = row.get(1)?;
            let data: Vec<u8> = row.get(2)?;
            let (title, body) = extract_text(&data);
            Ok(Row {
                rec_id,
                app_id: identifier,
                title,
                body,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn extract_text(data: &[u8]) -> (String, String) {
    let mut title = String::new();
    let mut body = String::new();
    if let Ok(v) = Value::from_reader(Cursor::new(data)) {
        find_text(&v, &mut title, &mut body);
    }
    (title, body)
}

/// Recursively search a plist Value for common Apple notification text keys.
/// Real-world keys observed in `record.data`: `titl`, `body`, `subt`, `appti`.
fn find_text(v: &Value, title: &mut String, body: &mut String) {
    match v {
        Value::Dictionary(d) => {
            for (k, vv) in d {
                match k.as_str() {
                    "titl" | "title" | "appti" => {
                        if title.is_empty() {
                            if let Some(s) = vv.as_string() {
                                *title = s.to_string();
                            }
                        }
                    }
                    "body" | "subt" => {
                        if body.is_empty() {
                            if let Some(s) = vv.as_string() {
                                *body = s.to_string();
                            }
                        }
                    }
                    _ => find_text(vv, title, body),
                }
            }
        }
        Value::Array(a) => {
            for item in a {
                find_text(item, title, body);
            }
        }
        _ => {}
    }
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        // Pre-flight check (fail fast if db unreachable).
        {
            let _ = open_db()?;
        }

        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || {
            let conn = match open_db() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(error = %e, "failed to open usernoted DB in poll thread");
                    return;
                }
            };
            let mut last_rec_id = match current_max_rec_id(&conn) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(error = %e, "failed to read max rec_id");
                    return;
                }
            };
            tracing::info!(
                interval_ms = %POLL_INTERVAL.as_millis(),
                last_rec_id,
                "macOS DB poll loop started"
            );

            let mut tick: u64 = 0;
            while *running.lock() {
                std::thread::sleep(POLL_INTERVAL);
                tick += 1;
                match query_new(&conn, last_rec_id) {
                    Ok(rows) => {
                        if tick % 20 == 0 {
                            tracing::debug!(tick, last_rec_id, "db poll heartbeat");
                        }
                        for r in rows {
                            last_rec_id = last_rec_id.max(r.rec_id);
                            if r.title.is_empty() && r.body.is_empty() {
                                tracing::debug!(rec_id = r.rec_id, "row has empty title+body; skipping");
                                continue;
                            }
                            let app_id = if r.app_id.is_empty() {
                                "com.apple.usernoted".to_string()
                            } else {
                                r.app_id.clone()
                            };
                            tracing::info!(
                                rec_id = r.rec_id,
                                app_id = %app_id,
                                title = %r.title,
                                "new macOS notification from db"
                            );
                            (publish)(NotiEvent::now(
                                app_id.clone(),
                                r.app_id.clone(),
                                r.title,
                                r.body,
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "db query failed");
                    }
                }
            }
            tracing::info!("macOS DB poll loop stopped");
        });

        Ok(())
    }

    fn stop(&self) {
        *self.running.lock() = false;
    }
}
