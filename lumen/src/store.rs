use crate::db::{self, read_setting, upsert_setting};
use crate::html::prepare_card_html;
use crate::import;
use crate::models::*;
use crate::scheduler::{preview_intervals, rate_card, state_name};
use crate::template::{looks_like_cloze, render_card};
use crate::Error;
use chrono::{TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
    media_dir: PathBuf,
}

impl Store {
    pub fn open(data_dir: impl AsRef<Path>) -> crate::Result<Self> {
        let data_dir = data_dir.as_ref();
        std::fs::create_dir_all(data_dir)?;
        let media_dir = data_dir.join("media");
        std::fs::create_dir_all(&media_dir)?;
        let conn = Connection::open(data_dir.join("lumen.sqlite"))?;
        db::initialize(&conn)?;
        let store = Self {
            conn: Mutex::new(conn),
            media_dir,
        };
        store.ensure_starter()?;
        Ok(store)
    }

    pub fn open_in_memory() -> crate::Result<Self> {
        let media_dir = std::env::temp_dir().join(format!(
            "lumen_media_{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&media_dir)?;
        let conn = Connection::open_in_memory()?;
        db::initialize(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            media_dir,
        })
    }

    pub fn media_dir(&self) -> &Path {
        &self.media_dir
    }

    pub fn import_apkg(&self, path: impl AsRef<Path>) -> crate::Result<ImportReport> {
        let conn = self.conn.lock().expect("db lock");
        import::import_apkg(&conn, path.as_ref(), &self.media_dir)
    }

    pub fn import_bytes(&self, bytes: &[u8]) -> crate::Result<ImportReport> {
        let conn = self.conn.lock().expect("db lock");
        import::import_bytes(&conn, bytes, &self.media_dir)
    }

    pub fn deck_tree(&self) -> crate::Result<Vec<DeckSummary>> {
        let conn = self.conn.lock().expect("db lock");
        let now = Utc::now().timestamp_millis();
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, name, full_name FROM decks ORDER BY full_name",
        )?;
        let decks: Vec<Deck> = stmt
            .query_map([], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    parent_id: r.get(1)?,
                    name: r.get(2)?,
                    full_name: r.get(3)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        let mut summaries = HashMap::new();
        for deck in decks {
            let due: i64 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND suspended = 0
                 AND state != 'new' AND due <= ?2",
                params![deck.id, now],
                |r| r.get(0),
            )?;
            let news: i64 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND suspended = 0 AND state = 'new'",
                params![deck.id],
                |r| r.get(0),
            )?;
            let total: i64 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE deck_id = ?1 AND suspended = 0",
                params![deck.id],
                |r| r.get(0),
            )?;
            summaries.insert(
                deck.id,
                DeckSummary {
                    deck,
                    due,
                    news,
                    total,
                    children: Vec::new(),
                },
            );
        }

        let mut pending: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut root_ids = Vec::new();
        for (id, node) in &summaries {
            match node.deck.parent_id {
                Some(pid) if summaries.contains_key(&pid) => {
                    pending.entry(pid).or_default().push(*id);
                }
                _ => root_ids.push(*id),
            }
        }
        fn take_tree(
            id: i64,
            summaries: &mut HashMap<i64, DeckSummary>,
            pending: &HashMap<i64, Vec<i64>>,
        ) -> DeckSummary {
            let mut node = summaries.remove(&id).expect("deck node");
            if let Some(kids) = pending.get(&id) {
                for kid in kids {
                    node.children.push(take_tree(*kid, summaries, pending));
                }
            }
            node
        }
        let mut roots: Vec<DeckSummary> = root_ids
            .into_iter()
            .map(|id| take_tree(id, &mut summaries, &pending))
            .collect();
        roots.sort_by(|a, b| a.deck.full_name.cmp(&b.deck.full_name));
        Ok(roots)
    }

    pub fn today(&self) -> crate::Result<TodayCounts> {
        let conn = self.conn.lock().expect("db lock");
        let now = Utc::now().timestamp_millis();
        let due: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE suspended = 0 AND state != 'new' AND due <= ?1",
            params![now],
            |r| r.get(0),
        )?;
        let news: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE suspended = 0 AND state = 'new'",
            [],
            |r| r.get(0),
        )?;
        Ok(TodayCounts { due, news })
    }

    pub fn queue(&self, deck_id: Option<i64>, new_limit: i64) -> crate::Result<Vec<StudyCard>> {
        let conn = self.conn.lock().expect("db lock");
        let now = Utc::now().timestamp_millis();
        let deck_ids = match deck_id {
            Some(id) => Some(self.deck_and_descendants(&conn, id)?),
            None => None,
        };

        let mut cards = Vec::new();
        {
            let sql = if deck_ids.is_some() {
                "SELECT id FROM cards WHERE suspended = 0 AND state = 'new' AND deck_id IN (SELECT value FROM json_each(?1)) ORDER BY id LIMIT ?2"
            } else {
                "SELECT id FROM cards WHERE suspended = 0 AND state = 'new' ORDER BY id LIMIT ?1"
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = if let Some(ids) = &deck_ids {
                stmt.query_map(params![serde_json::to_string(ids)?, new_limit], |r| r.get(0))?
                    .collect::<Vec<_>>()
            } else {
                stmt.query_map(params![new_limit], |r| r.get(0))?
                    .collect::<Vec<_>>()
            };
            for id in rows.into_iter().flatten() {
                cards.push(id);
            }
        }
        {
            let sql = if deck_ids.is_some() {
                "SELECT id FROM cards WHERE suspended = 0 AND state != 'new' AND due <= ?1 AND deck_id IN (SELECT value FROM json_each(?2)) ORDER BY due LIMIT 80"
            } else {
                "SELECT id FROM cards WHERE suspended = 0 AND state != 'new' AND due <= ?1 ORDER BY due LIMIT 80"
            };
            let mut stmt = conn.prepare(sql)?;
            let rows = if let Some(ids) = &deck_ids {
                stmt.query_map(params![now, serde_json::to_string(ids)?], |r| r.get(0))?
                    .collect::<Vec<_>>()
            } else {
                stmt.query_map(params![now], |r| r.get(0))?.collect::<Vec<_>>()
            };
            for id in rows.into_iter().flatten() {
                cards.push(id);
            }
        }

        let mut out = Vec::new();
        for id in cards {
            if let Some(card) = self.study_card(&conn, id)? {
                out.push(card);
            }
        }
        Ok(out)
    }

    pub fn rate(&self, card_id: i64, rating: u8) -> crate::Result<StudyCard> {
        let conn = self.conn.lock().expect("db lock");
        let row = self.card_row(&conn, card_id)?;
        let now = Utc::now();
        let next = rate_card(&row, rating, now);
        conn.execute(
            "UPDATE cards SET state = ?1, due = ?2, stability = ?3, difficulty = ?4,
             reps = ?5, lapses = ?6, scheduled_days = ?7, last_review = ?8,
             first_reviewed_at = COALESCE(first_reviewed_at, ?8)
             WHERE id = ?9",
            params![
                state_name(next.state),
                next.due.timestamp_millis(),
                next.stability,
                next.difficulty,
                next.reps,
                next.lapses,
                next.scheduled_days,
                now.timestamp_millis(),
                card_id
            ],
        )?;
        conn.execute(
            "INSERT INTO reviews (card_id, rated_at, rating, stability_after) VALUES (?1, ?2, ?3, ?4)",
            params![card_id, now.timestamp_millis(), rating, next.stability],
        )?;
        self.study_card(&conn, card_id)?
            .ok_or_else(|| Error::msg("card missing after review"))
    }

    pub fn add_basic(
        &self,
        deck_id: i64,
        front: &str,
        back: &str,
        tags: &str,
    ) -> crate::Result<i64> {
        let conn = self.conn.lock().expect("db lock");
        let type_id = match read_setting(&conn, "default_note_type_id")? {
            Some(v) => v.parse().unwrap_or(0),
            None => 0,
        };
        let type_id = if type_id == 0 {
            ensure_basic_type(&conn)?
        } else {
            type_id
        };
        let now = Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO notes (note_type_id, tags, modified_at) VALUES (?1, ?2, ?3)",
            params![type_id, tags, now],
        )?;
        let note_id = conn.last_insert_rowid();
        let mut fstmt = conn.prepare(
            "SELECT id, ordinal FROM fields WHERE note_type_id = ?1 ORDER BY ordinal",
        )?;
        let fields: Vec<(i64, i64)> = fstmt
            .query_map(params![type_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(Result::ok)
            .collect();
        drop(fstmt);
        for (field_id, ord) in fields {
            let value = if ord == 0 {
                front
            } else if ord == 1 {
                back
            } else {
                ""
            };
            conn.execute(
                "INSERT INTO note_fields (note_id, field_id, value) VALUES (?1, ?2, ?3)",
                params![note_id, field_id, value],
            )?;
        }
        let mut tstmt =
            conn.prepare("SELECT id, ordinal FROM templates WHERE note_type_id = ?1")?;
        let templates: Vec<(i64, i64)> = tstmt
            .query_map(params![type_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(Result::ok)
            .collect();
        drop(tstmt);
        for (template_id, ord) in templates {
            conn.execute(
                "INSERT INTO cards (note_id, deck_id, template_id, ordinal, due) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note_id, deck_id, template_id, ord, now],
            )?;
        }
        Ok(note_id)
    }

    pub fn create_deck(&self, name: &str) -> crate::Result<Deck> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO decks (name, full_name) VALUES (?1, ?1)",
            params![name],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Deck {
            id,
            parent_id: None,
            name: name.to_string(),
            full_name: name.to_string(),
        })
    }

    pub fn default_deck_id(&self) -> crate::Result<i64> {
        let conn = self.conn.lock().expect("db lock");
        if let Some(v) = read_setting(&conn, "default_deck_id")? {
            if let Ok(id) = v.parse() {
                return Ok(id);
            }
        }
        Ok(conn.query_row("SELECT id FROM decks ORDER BY id LIMIT 1", [], |r| {
            r.get(0)
        })?)
    }

    pub fn browse(&self, deck_id: Option<i64>, query: &str) -> crate::Result<Vec<BrowseRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut sql = "SELECT id FROM cards".to_string();
        if deck_id.is_some() {
            sql.push_str(" WHERE deck_id = ?1");
        }
        sql.push_str(" ORDER BY id DESC LIMIT 400");
        let mut stmt = conn.prepare(&sql)?;
        let ids: Vec<i64> = if let Some(id) = deck_id {
            stmt.query_map(params![id], |r| r.get(0))?
                .filter_map(Result::ok)
                .collect()
        } else {
            stmt.query_map([], |r| r.get(0))?
                .filter_map(Result::ok)
                .collect()
        };
        let q = query.trim().to_lowercase();
        let mut rows = Vec::new();
        for id in ids {
            let Some(study) = self.study_card(&conn, id)? else {
                continue;
            };
            if !q.is_empty()
                && !study.front.to_lowercase().contains(&q)
                && !study.back.to_lowercase().contains(&q)
                && !study.tags.to_lowercase().contains(&q)
            {
                continue;
            }
            let card = self.card_row(&conn, id)?;
            rows.push(BrowseRow {
                card_id: id,
                note_id: study.note_id,
                front: study.front,
                back: study.back,
                state: card.state,
                due: card.due,
                suspended: card.suspended,
                tags: study.tags,
            });
        }
        Ok(rows)
    }

    pub fn set_suspended(&self, card_id: i64, suspended: bool) -> crate::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE cards SET suspended = ?1 WHERE id = ?2",
            params![i64::from(suspended), card_id],
        )?;
        Ok(())
    }

    pub fn delete_deck(&self, deck_id: i64) -> crate::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        let ids = self.deck_and_descendants(&conn, deck_id)?;
        if ids.is_empty() {
            return Ok(());
        }
        let list = serde_json::to_string(&ids)?;
        conn.execute(
            "DELETE FROM reviews WHERE card_id IN (
                SELECT id FROM cards WHERE deck_id IN (SELECT value FROM json_each(?1))
             )",
            params![list],
        )?;
        let note_ids: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT note_id FROM cards WHERE deck_id IN (SELECT value FROM json_each(?1))",
            )?;
            let ids: Vec<i64> = stmt
                .query_map(params![list], |r| r.get(0))?
                .filter_map(Result::ok)
                .collect();
            ids
        };
        conn.execute(
            "DELETE FROM cards WHERE deck_id IN (SELECT value FROM json_each(?1))",
            params![list],
        )?;
        for note_id in note_ids {
            let remaining: i64 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE note_id = ?1",
                params![note_id],
                |r| r.get(0),
            )?;
            if remaining == 0 {
                conn.execute(
                    "DELETE FROM note_fields WHERE note_id = ?1",
                    params![note_id],
                )?;
                conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])?;
            }
        }
        conn.execute(
            "DELETE FROM decks WHERE id IN (SELECT value FROM json_each(?1))",
            params![list],
        )?;
        let remaining_decks: i64 =
            conn.query_row("SELECT COUNT(*) FROM decks", [], |r| r.get(0))?;
        if remaining_decks == 0 {
            conn.execute(
                "INSERT INTO decks (name, full_name) VALUES ('Inbox', 'Inbox')",
                [],
            )?;
            let id = conn.last_insert_rowid();
            upsert_setting(&conn, "default_deck_id", &id.to_string())?;
        } else if let Some(current) = read_setting(&conn, "default_deck_id")? {
            if current
                .parse::<i64>()
                .ok()
                .is_some_and(|id| ids.contains(&id))
            {
                let next: i64 =
                    conn.query_row("SELECT id FROM decks ORDER BY id LIMIT 1", [], |r| r.get(0))?;
                upsert_setting(&conn, "default_deck_id", &next.to_string())?;
            }
        }
        Ok(())
    }

    pub fn ensure_starter(&self) -> crate::Result<()> {
        if self.setting("starter_v1")?.as_deref() == Some("1") {
            return Ok(());
        }
        let existing: i64 = {
            let conn = self.conn.lock().expect("db lock");
            conn.query_row(
                "SELECT COUNT(*) FROM decks WHERE full_name = 'Getting started'",
                [],
                |r| r.get(0),
            )?
        };
        if existing == 0 {
            let deck = self.create_deck("Getting started")?;
            let wav = self.media_dir.join("lumen-welcome.wav");
            write_sine_wav(&wav, 440.0, 500)?;
            {
                let conn = self.conn.lock().expect("db lock");
                conn.execute(
                    "INSERT INTO media (filename, path) VALUES (?1, ?2)",
                    params!["lumen-welcome.wav", wav.to_string_lossy().as_ref()],
                )?;
            }
            let cards = [
                (
                    "Welcome to Lumen",
                    "A local study app that can import Anki decks. Scheduling starts fresh with FSRS.",
                ),
                (
                    "Show the answer",
                    "Press space or click Show answer. Then rate Again, Hard, Good, or Easy.",
                ),
                (
                    "Cards can play audio\n[sound:lumen-welcome.wav]",
                    "That clip used Anki’s [sound:] tag. mp3, wav, m4a, and ogg play the same way.",
                ),
                (
                    "Cards can play video",
                    "Imported Anki decks can include mp4, webm, and mov files. They play inline with controls, like audio.",
                ),
                ("ありがとう", "Thank you"),
                ("Capital of Japan?", "Tokyo"),
                (
                    "Find more decks",
                    "Sign in to AnkiWeb from Lumen, search shared decks, and download a package. Lumen imports it locally.",
                ),
                (
                    "Remove a deck",
                    "Open a deck in the sidebar and choose Remove. That only deletes it from Lumen, not from AnkiWeb.",
                ),
            ];
            for (front, back) in cards {
                self.add_basic(deck.id, front, back, "starter")?;
            }
        }
        self.set_setting("starter_v1", "1")?;
        Ok(())
    }

    pub fn setting(&self, key: &str) -> crate::Result<Option<String>> {
        let conn = self.conn.lock().expect("db lock");
        read_setting(&conn, key)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> crate::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        upsert_setting(&conn, key, value)
    }

    fn deck_and_descendants(&self, conn: &Connection, id: i64) -> crate::Result<Vec<i64>> {
        let mut stmt = conn.prepare("SELECT id, parent_id FROM decks")?;
        let all: Vec<(i64, Option<i64>)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(Result::ok)
            .collect();
        let mut ids = vec![id];
        let mut changed = true;
        while changed {
            changed = false;
            for (did, parent) in &all {
                if let Some(p) = parent {
                    if ids.contains(p) && !ids.contains(did) {
                        ids.push(*did);
                        changed = true;
                    }
                }
            }
        }
        Ok(ids)
    }

    fn card_row(&self, conn: &Connection, id: i64) -> crate::Result<CardRow> {
        conn.query_row(
            "SELECT id, note_id, deck_id, template_id, ordinal, state, due, stability, difficulty,
                    reps, lapses, scheduled_days, last_review, first_reviewed_at, suspended
             FROM cards WHERE id = ?1",
            params![id],
            |r| {
                Ok(CardRow {
                    id: r.get(0)?,
                    note_id: r.get(1)?,
                    deck_id: r.get(2)?,
                    template_id: r.get(3)?,
                    ordinal: r.get(4)?,
                    state: r.get(5)?,
                    due: millis_to_dt(r.get(6)?),
                    stability: r.get(7)?,
                    difficulty: r.get(8)?,
                    reps: r.get(9)?,
                    lapses: r.get(10)?,
                    scheduled_days: r.get(11)?,
                    last_review: r.get::<_, Option<i64>>(12)?.map(millis_to_dt),
                    first_reviewed_at: r.get::<_, Option<i64>>(13)?.map(millis_to_dt),
                    suspended: r.get::<_, i64>(14)? != 0,
                })
            },
        )
        .map_err(Error::from)
    }

    fn study_card(&self, conn: &Connection, card_id: i64) -> crate::Result<Option<StudyCard>> {
        let Ok(card) = self.card_row(conn, card_id) else {
            return Ok(None);
        };
        let (note_type_id, tags): (i64, String) = conn.query_row(
            "SELECT note_type_id, tags FROM notes WHERE id = ?1",
            params![card.note_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let is_cloze: i64 = conn.query_row(
            "SELECT is_cloze FROM note_types WHERE id = ?1",
            params![note_type_id],
            |r| r.get(0),
        )?;
        let mut fstmt = conn.prepare(
            "SELECT f.name, COALESCE(nf.value, '') FROM fields f
             LEFT JOIN note_fields nf ON nf.field_id = f.id AND nf.note_id = ?1
             WHERE f.note_type_id = ?2 ORDER BY f.ordinal",
        )?;
        let mut fields = HashMap::new();
        let rows = fstmt.query_map(params![card.note_id, note_type_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (name, value) = row?;
            fields.insert(name, value);
        }
        drop(fstmt);
        let (front_html, back_html): (String, String) = conn
            .query_row(
                "SELECT front_html, back_html FROM templates WHERE id = ?1",
                params![card.template_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .unwrap_or_else(|| ("{{Front}}".into(), "{{Back}}".into()));
        let deck_name: String = conn.query_row(
            "SELECT full_name FROM decks WHERE id = ?1",
            params![card.deck_id],
            |r| r.get(0),
        )?;
        let rendered = render_card(
            &front_html,
            &back_html,
            &fields,
            card.ordinal,
            is_cloze != 0 || looks_like_cloze(&fields),
        );
        let intervals = preview_intervals(&card, Utc::now());
        let media = listed_media(conn)?;
        Ok(Some(StudyCard {
            card_id: card.id,
            note_id: card.note_id,
            deck_name,
            tags,
            front: rendered.front,
            back: rendered.back,
            front_html: prepare_card_html(&rendered.front_html, &media),
            back_html: prepare_card_html(&rendered.back_html, &media),
            images: rendered.images,
            audio: rendered.audio,
            video: rendered.video,
            state: card.state,
            intervals,
        }))
    }
}

fn listed_media(conn: &Connection) -> crate::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT filename, path FROM media")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let media = rows.filter_map(Result::ok).collect();
    Ok(media)
}

fn write_sine_wav(path: &Path, freq: f32, ms: u32) -> std::io::Result<()> {
    use std::io::Write;
    let sr = 22050u32;
    let n = sr * ms / 1000;
    let mut pcm = Vec::with_capacity((n as usize) * 2);
    for i in 0..n {
        let t = i as f32 / sr as f32;
        let fade = 800.min(n / 4);
        let env = if i < fade {
            i as f32 / fade as f32
        } else if i + fade > n {
            (n - i) as f32 / fade as f32
        } else {
            1.0
        };
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin() * 0.22 * env;
        let amplitude = (sample * i16::MAX as f32).round() as i16;
        pcm.extend_from_slice(&amplitude.to_le_bytes());
    }
    let data_len = pcm.len() as u32;
    let mut out = std::fs::File::create(path)?;
    out.write_all(b"RIFF")?;
    out.write_all(&(36 + data_len).to_le_bytes())?;
    out.write_all(b"WAVE")?;
    out.write_all(b"fmt ")?;
    out.write_all(&16u32.to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&sr.to_le_bytes())?;
    out.write_all(&(sr * 2).to_le_bytes())?;
    out.write_all(&2u16.to_le_bytes())?;
    out.write_all(&16u16.to_le_bytes())?;
    out.write_all(b"data")?;
    out.write_all(&data_len.to_le_bytes())?;
    out.write_all(&pcm)?;
    Ok(())
}

fn ensure_basic_type(conn: &Connection) -> crate::Result<i64> {
    conn.execute("INSERT INTO note_types (name) VALUES ('Basic')", [])?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO fields (note_type_id, name, ordinal) VALUES (?1, 'Front', 0)",
        params![id],
    )?;
    conn.execute(
        "INSERT INTO fields (note_type_id, name, ordinal) VALUES (?1, 'Back', 1)",
        params![id],
    )?;
    conn.execute(
        "INSERT INTO templates (note_type_id, name, front_html, back_html, ordinal)
         VALUES (?1, 'Card 1', '{{Front}}', '{{FrontSide}}<hr id=answer>{{Back}}', 0)",
        params![id],
    )?;
    upsert_setting(conn, "default_note_type_id", &id.to_string())?;
    Ok(id)
}

fn millis_to_dt(ms: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_fixture_package() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("../../fixtures/lumen_basic.apkg");
        assert!(fixture.exists(), "run python3 fixtures/generate_basic_apkg.py");
        let store = Store::open_in_memory().unwrap();
        let report = store.import_apkg(&fixture).unwrap();
        assert_eq!(report.notes, 1);
        assert_eq!(report.cards, 1);
        assert!(report.root_deck_name.contains("Lumen Sample"));
        let today = store.today().unwrap();
        assert!(today.news >= 1);
        let queue = store.queue(None, 20).unwrap();
        assert!(!queue.is_empty());
        assert!(queue[0].front.contains("France") || queue[0].back.contains("Paris"));
    }

    #[test]
    fn seeds_starter_and_deletes_deck() {
        let store = Store::open_in_memory().unwrap();
        store.ensure_starter().unwrap();
        let tree = store.deck_tree().unwrap();
        let starter = tree
            .iter()
            .find(|d| d.deck.full_name == "Getting started")
            .expect("starter deck");
        assert!(starter.total >= 8);
        let queue = store.queue(Some(starter.deck.id), 20).unwrap();
        assert!(queue.iter().any(|c| c.front_html.contains("<audio") || c.audio.iter().any(|a| a.contains("wav"))));
        store.delete_deck(starter.deck.id).unwrap();
        let tree = store.deck_tree().unwrap();
        assert!(tree.iter().all(|d| d.deck.full_name != "Getting started"));
    }
}
