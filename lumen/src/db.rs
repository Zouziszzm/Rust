use crate::Error;
use rusqlite::{params, Connection};

pub fn initialize(conn: &Connection) -> crate::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS decks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            parent_id INTEGER,
            name TEXT NOT NULL,
            full_name TEXT NOT NULL,
            anki_deck_id INTEGER,
            new_per_day INTEGER NOT NULL DEFAULT 20,
            rev_per_day INTEGER NOT NULL DEFAULT 200,
            is_filtered INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS note_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            anki_model_id INTEGER,
            css TEXT NOT NULL DEFAULT '',
            is_cloze INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS fields (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_type_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            ordinal INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_type_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            front_html TEXT NOT NULL,
            back_html TEXT NOT NULL,
            ordinal INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_type_id INTEGER NOT NULL,
            anki_note_id INTEGER,
            tags TEXT NOT NULL DEFAULT '',
            modified_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS note_fields (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_id INTEGER NOT NULL,
            field_id INTEGER NOT NULL,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_id INTEGER NOT NULL,
            deck_id INTEGER NOT NULL,
            template_id INTEGER NOT NULL,
            anki_card_id INTEGER,
            ordinal INTEGER NOT NULL DEFAULT 0,
            state TEXT NOT NULL DEFAULT 'new',
            due INTEGER NOT NULL,
            stability REAL,
            difficulty REAL,
            reps INTEGER NOT NULL DEFAULT 0,
            lapses INTEGER NOT NULL DEFAULT 0,
            scheduled_days INTEGER NOT NULL DEFAULT 0,
            last_review INTEGER,
            first_reviewed_at INTEGER,
            suspended INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            rated_at INTEGER NOT NULL,
            rating INTEGER NOT NULL,
            stability_after REAL
        );
        CREATE TABLE IF NOT EXISTS media (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            path TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_cards_due ON cards(due);
        CREATE INDEX IF NOT EXISTS idx_cards_deck ON cards(deck_id);
        "#,
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM decks", [], |r| r.get(0))?;
    if count == 0 {
        seed(conn)?;
    }
    Ok(())
}

fn seed(conn: &Connection) -> crate::Result<()> {
    conn.execute(
        "INSERT INTO decks (name, full_name) VALUES ('Inbox', 'Inbox')",
        [],
    )?;
    let deck_id = conn.last_insert_rowid();
    conn.execute("INSERT INTO note_types (name) VALUES ('Basic')", [])?;
    let type_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO fields (note_type_id, name, ordinal) VALUES (?1, 'Front', 0)",
        params![type_id],
    )?;
    conn.execute(
        "INSERT INTO fields (note_type_id, name, ordinal) VALUES (?1, 'Back', 1)",
        params![type_id],
    )?;
    conn.execute(
        "INSERT INTO templates (note_type_id, name, front_html, back_html, ordinal)
         VALUES (?1, 'Card 1', '{{Front}}', '{{FrontSide}}<hr id=answer>{{Back}}', 0)",
        params![type_id],
    )?;
    upsert_setting(conn, "default_deck_id", &deck_id.to_string())?;
    upsert_setting(conn, "default_note_type_id", &type_id.to_string())?;
    upsert_setting(conn, "new_per_day", "20")?;
    upsert_setting(conn, "desired_retention", "0.90")?;
    Ok(())
}

pub fn upsert_setting(conn: &Connection, key: &str, value: &str) -> crate::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn read_setting(conn: &Connection, key: &str) -> crate::Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let value = stmt
        .query_row(params![key], |r| r.get(0))
        .optional()
        .map_err(Error::from)?;
    Ok(value)
}

trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
