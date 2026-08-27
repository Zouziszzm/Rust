use crate::html::rewrite_media_src;
use crate::Error;
use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::models::ImportReport;

struct AnkiField {
    name: String,
    ord: i64,
}

struct AnkiTemplate {
    name: String,
    ord: i64,
    qfmt: String,
    afmt: String,
}

struct AnkiModel {
    id: i64,
    name: String,
    is_cloze: bool,
    css: String,
    fields: Vec<AnkiField>,
    templates: Vec<AnkiTemplate>,
}

struct AnkiDeck {
    id: i64,
    full_name: String,
    filtered: bool,
}

pub fn import_apkg(
    dest: &Connection,
    package: &Path,
    media_dir: &Path,
) -> crate::Result<ImportReport> {
    let tmp = tempfile_dir()?;
    let result = (|| {
        unzip(package, &tmp)?;
        import_extracted(dest, &tmp, media_dir)
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

pub fn import_bytes(
    dest: &Connection,
    bytes: &[u8],
    media_dir: &Path,
) -> crate::Result<ImportReport> {
    let tmp = tempfile_dir()?;
    let pkg = tmp.join("upload.apkg");
    fs::write(&pkg, bytes)?;
    let result = import_apkg(dest, &pkg, media_dir);
    let _ = fs::remove_dir_all(&tmp);
    result
}

fn tempfile_dir() -> crate::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "lumen_apkg_{}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn unzip(package: &Path, dest: &Path) -> crate::Result<()> {
    let file = fs::File::open(package)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().replace('\\', "/");
        if name.contains("..") {
            continue;
        }
        let out = dest.join(&name);
        if !out.starts_with(dest) {
            continue;
        }
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            fs::write(out, buf)?;
        }
    }
    Ok(())
}

fn import_extracted(
    dest: &Connection,
    tmp: &Path,
    media_dir: &Path,
) -> crate::Result<ImportReport> {
    let mut warnings = Vec::new();
    let db_file = resolve_collection(tmp, &mut warnings)?;
    let anki = Connection::open_with_flags(&db_file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let tables = table_names(&anki)?;

    let models = read_models(&anki, &tables, &mut warnings)?;
    let anki_decks = read_decks(&anki, &tables, &mut warnings)?;
    let media_map = read_media_map(tmp, &mut warnings)?;

    fs::create_dir_all(media_dir)?;
    let mut filename_to_path = Vec::new();
    for (index, filename) in &media_map {
        let src = tmp.join(index);
        if !src.exists() {
            warnings.push(format!("Missing media file {index} ({filename})"));
            continue;
        }
        let dest_path = media_dir.join(filename);
        fs::copy(&src, &dest_path)?;
        dest.execute(
            "INSERT INTO media (filename, path) VALUES (?1, ?2)",
            params![filename, dest_path.to_string_lossy().as_ref()],
        )?;
        filename_to_path.push((filename.clone(), dest_path.to_string_lossy().into_owned()));
    }

    let mut deck_id_map = HashMap::new();
    let mut imported_decks = 0i64;
    let mut sorted: Vec<&AnkiDeck> = anki_decks.values().collect();
    sorted.sort_by_key(|d| d.full_name.split("::").count());

    for deck in &sorted {
        if deck.filtered {
            warnings.push(format!("Skipped filtered deck \"{}\"", deck.full_name));
            continue;
        }
        if deck.full_name == "Default" && anki_decks.len() > 1 {
            continue;
        }
        let parts: Vec<&str> = deck.full_name.split("::").collect();
        let name = parts.last().copied().unwrap_or("Deck");
        let parent_id = if parts.len() > 1 {
            let parent_name = parts[..parts.len() - 1].join("::");
            anki_decks
                .values()
                .find(|d| d.full_name == parent_name)
                .and_then(|p| deck_id_map.get(&p.id).copied())
        } else {
            None
        };
        dest.execute(
            "INSERT INTO decks (parent_id, name, full_name, anki_deck_id, is_filtered)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![parent_id, name, deck.full_name, deck.id],
        )?;
        deck_id_map.insert(deck.id, dest.last_insert_rowid());
        imported_decks += 1;
    }

    if deck_id_map.is_empty() {
        dest.execute(
            "INSERT INTO decks (name, full_name) VALUES ('Imported', 'Imported')",
            [],
        )?;
        deck_id_map.insert(1, dest.last_insert_rowid());
        imported_decks = 1;
    }

    let mut type_id_map = HashMap::new();
    let mut field_id_map: HashMap<i64, HashMap<i64, i64>> = HashMap::new();
    let mut template_id_map: HashMap<i64, HashMap<i64, i64>> = HashMap::new();

    for model in models.values() {
        dest.execute(
            "INSERT INTO note_types (name, anki_model_id, css, is_cloze) VALUES (?1, ?2, ?3, ?4)",
            params![model.name, model.id, model.css, model.is_cloze as i64],
        )?;
        let type_id = dest.last_insert_rowid();
        type_id_map.insert(model.id, type_id);
        let fields = field_id_map.entry(model.id).or_default();
        for field in &model.fields {
            dest.execute(
                "INSERT INTO fields (note_type_id, name, ordinal) VALUES (?1, ?2, ?3)",
                params![type_id, field.name, field.ord],
            )?;
            fields.insert(field.ord, dest.last_insert_rowid());
        }
        let templates = template_id_map.entry(model.id).or_default();
        for tmpl in &model.templates {
            dest.execute(
                "INSERT INTO templates (note_type_id, name, front_html, back_html, ordinal)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![type_id, tmpl.name, tmpl.qfmt, tmpl.afmt, tmpl.ord],
            )?;
            templates.insert(tmpl.ord, dest.last_insert_rowid());
        }
    }

    let mut note_stmt = anki.prepare("SELECT id, mid, tags, flds FROM notes")?;
    let notes = note_stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2).unwrap_or_default(),
            r.get::<_, String>(3).unwrap_or_default(),
        ))
    })?;

    let mut note_count = 0i64;
    let mut note_id_map = HashMap::new();
    let now = Utc::now().timestamp_millis();

    for note in notes {
        let (anki_note_id, mid, tags, mut flds) = note?;
        let Some(&type_id) = type_id_map.get(&mid) else {
            warnings.push(format!("Note {anki_note_id} has unknown note type {mid}"));
            continue;
        };
        if !filename_to_path.is_empty() {
            flds = rewrite_media_src(&flds, &filename_to_path);
        }
        let values: Vec<&str> = flds.split('\u{1f}').collect();
        dest.execute(
            "INSERT INTO notes (note_type_id, anki_note_id, tags, modified_at) VALUES (?1, ?2, ?3, ?4)",
            params![type_id, anki_note_id, tags.trim(), now],
        )?;
        let lumen_note_id = dest.last_insert_rowid();
        note_id_map.insert(anki_note_id, lumen_note_id);
        if let Some(fields) = field_id_map.get(&mid) {
            for (ord, field_id) in fields {
                let value = values.get(*ord as usize).copied().unwrap_or("");
                dest.execute(
                    "INSERT INTO note_fields (note_id, field_id, value) VALUES (?1, ?2, ?3)",
                    params![lumen_note_id, field_id, value],
                )?;
            }
        }
        note_count += 1;
    }

    let mut card_stmt = anki.prepare("SELECT id, nid, did, ord, queue FROM cards")?;
    let cards = card_stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4).unwrap_or(0),
        ))
    })?;

    let mut card_count = 0i64;
    let fallback_deck = *deck_id_map.values().next().unwrap();
    for card in cards {
        let (anki_card_id, nid, did, ord, queue) = card?;
        let Some(&lumen_note_id) = note_id_map.get(&nid) else {
            continue;
        };
        let deck_id = deck_id_map.get(&did).copied().unwrap_or(fallback_deck);
        let mid = note_mid(&anki, nid)?;
        let template_id = template_id_map
            .get(&mid)
            .and_then(|m| m.get(&ord).or_else(|| m.values().next()))
            .copied();
        let Some(template_id) = template_id else {
            warnings.push(format!("Card {anki_card_id} missing template"));
            continue;
        };
        dest.execute(
            "INSERT INTO cards (note_id, deck_id, template_id, anki_card_id, ordinal, due, suspended)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                lumen_note_id,
                deck_id,
                template_id,
                anki_card_id,
                ord,
                now,
                i64::from(queue == -1)
            ],
        )?;
        card_count += 1;
    }

    let root_name = sorted
        .iter()
        .filter(|d| !d.filtered && d.full_name != "Default")
        .map(|d| {
            d.full_name
                .split("::")
                .next()
                .unwrap_or(&d.full_name)
                .to_string()
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");

    Ok(ImportReport {
        notes: note_count,
        cards: card_count,
        media: filename_to_path.len() as i64,
        decks: imported_decks,
        warnings,
        root_deck_name: if root_name.is_empty() {
            "Imported".into()
        } else {
            root_name
        },
    })
}

fn note_mid(anki: &Connection, nid: i64) -> crate::Result<i64> {
    Ok(anki.query_row("SELECT mid FROM notes WHERE id = ?1", params![nid], |r| {
        r.get(0)
    })?)
}

fn table_names(anki: &Connection) -> crate::Result<std::collections::HashSet<String>> {
    let mut stmt = anki.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    let mut set = std::collections::HashSet::new();
    for row in rows {
        set.insert(row?);
    }
    Ok(set)
}

fn resolve_collection(tmp: &Path, warnings: &mut Vec<String>) -> crate::Result<PathBuf> {
    let anki21b = tmp.join("collection.anki21b");
    let anki21 = tmp.join("collection.anki21");
    let anki2 = tmp.join("collection.anki2");
    if anki21b.exists() {
        match zstd::decode_all(fs::File::open(&anki21b)?) {
            Ok(decoded) => {
                let out = tmp.join("collection.decoded.sqlite");
                fs::write(&out, decoded)?;
                return Ok(out);
            }
            Err(e) => warnings.push(format!(
                "Could not read latest Anki package (anki21b). Re-export with “Support older Anki versions”. ({e})"
            )),
        }
    }
    if anki21.exists() {
        return Ok(anki21);
    }
    if anki2.exists() {
        return Ok(anki2);
    }
    Err(Error::msg("Not a valid Anki package (no collection database)."))
}

fn read_models(
    anki: &Connection,
    tables: &std::collections::HashSet<String>,
    warnings: &mut Vec<String>,
) -> crate::Result<HashMap<i64, AnkiModel>> {
    if tables.contains("col") {
        if let Ok(raw) = anki.query_row("SELECT models FROM col", [], |r| r.get::<_, String>(0)) {
            if raw.trim().starts_with('{') {
                let map: HashMap<String, Value> = serde_json::from_str(&raw)?;
                if !map.is_empty() {
                    let mut out = HashMap::new();
                    for (k, v) in map {
                        if let Ok(id) = k.parse::<i64>() {
                            out.insert(id, model_from_json(&v));
                        }
                    }
                    return Ok(out);
                }
            }
        }
    }
    if tables.contains("notetypes") && tables.contains("fields") {
        return read_models_from_tables(anki, tables, warnings);
    }
    warnings.push("Could not read note types; fields will be numbered.".into());
    Ok(HashMap::new())
}

fn model_from_json(value: &Value) -> AnkiModel {
    let fields = value["flds"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, f)| AnkiField {
            name: f["name"].as_str().unwrap_or("Field").to_string(),
            ord: f["ord"].as_i64().unwrap_or(i as i64),
        })
        .collect();
    let templates = value["tmpls"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, t)| AnkiTemplate {
            name: t["name"].as_str().unwrap_or("Card").to_string(),
            ord: t["ord"].as_i64().unwrap_or(i as i64),
            qfmt: t["qfmt"].as_str().unwrap_or("{{Front}}").to_string(),
            afmt: t["afmt"].as_str().unwrap_or("{{Back}}").to_string(),
        })
        .collect();
    AnkiModel {
        id: value["id"].as_i64().unwrap_or(0),
        name: value["name"].as_str().unwrap_or("Note type").to_string(),
        is_cloze: value["type"].as_i64() == Some(1),
        css: value["css"].as_str().unwrap_or("").to_string(),
        fields,
        templates,
    }
}

fn read_models_from_tables(
    anki: &Connection,
    tables: &std::collections::HashSet<String>,
    warnings: &mut Vec<String>,
) -> crate::Result<HashMap<i64, AnkiModel>> {
    let mut models = HashMap::new();
    let mut stmt = anki.prepare("SELECT id, name FROM notetypes")?;
    let types = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    for t in types {
        let (id, name) = t?;
        let mut fstmt = anki.prepare("SELECT ord, name FROM fields WHERE ntid = ?1 ORDER BY ord")?;
        let fields = fstmt
            .query_map(params![id], |r| {
                Ok(AnkiField {
                    ord: r.get(0)?,
                    name: r.get::<_, String>(1).unwrap_or_else(|_| "Field".into()),
                })
            })?
            .filter_map(Result::ok)
            .collect();
        let templates = if tables.contains("templates") {
            let mut tstmt =
                anki.prepare("SELECT ord, name FROM templates WHERE ntid = ?1 ORDER BY ord")?;
            let mut out = Vec::new();
            let mapped = tstmt.query_map(params![id], |r| {
                Ok(AnkiTemplate {
                    ord: r.get(0)?,
                    name: r.get::<_, String>(1).unwrap_or_else(|_| "Card".into()),
                    qfmt: "{{Front}}".into(),
                    afmt: "{{Back}}".into(),
                })
            })?;
            for row in mapped {
                out.push(row?);
            }
            out
        } else {
            Vec::new()
        };
        models.insert(
            id,
            AnkiModel {
                id,
                name,
                is_cloze: false,
                css: String::new(),
                fields,
                templates,
            },
        );
    }
    if models.is_empty() {
        warnings.push("Note-type tables were empty.".into());
    }
    Ok(models)
}

fn read_decks(
    anki: &Connection,
    tables: &std::collections::HashSet<String>,
    warnings: &mut Vec<String>,
) -> crate::Result<HashMap<i64, AnkiDeck>> {
    if tables.contains("col") {
        if let Ok(raw) = anki.query_row("SELECT decks FROM col", [], |r| r.get::<_, String>(0)) {
            if raw.trim().starts_with('{') {
                let map: HashMap<String, Value> = serde_json::from_str(&raw)?;
                if !map.is_empty() {
                    let mut out = HashMap::new();
                    for (k, v) in map {
                        if let Ok(id) = k.parse::<i64>() {
                            out.insert(
                                id,
                                AnkiDeck {
                                    id: v["id"].as_i64().unwrap_or(id),
                                    full_name: v["name"].as_str().unwrap_or("Deck").to_string(),
                                    filtered: v["dyn"].as_i64() == Some(1),
                                },
                            );
                        }
                    }
                    return Ok(out);
                }
            }
        }
    }
    if tables.contains("decks") {
        let mut stmt = anki.prepare("SELECT id, name FROM decks")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = HashMap::new();
        for row in rows {
            let (id, name) = row?;
            out.insert(
                id,
                AnkiDeck {
                    id,
                    full_name: name,
                    filtered: false,
                },
            );
        }
        return Ok(out);
    }
    warnings.push("No decks found; using a single Imported deck.".into());
    Ok(HashMap::new())
}

fn read_media_map(tmp: &Path, warnings: &mut Vec<String>) -> crate::Result<HashMap<String, String>> {
    let file = tmp.join("media");
    if !file.exists() {
        return Ok(HashMap::new());
    }
    let bytes = fs::read(&file)?;
    if let Ok(decoded) = String::from_utf8(bytes.clone()) {
        if decoded.trim().starts_with('{') {
            let map: HashMap<String, Value> = serde_json::from_str(&decoded)?;
            return Ok(map
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.as_str()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| v.to_string()),
                    )
                })
                .collect());
        }
    }
    warnings.push("Media map is not JSON (newer Protobuf format). Media may be skipped.".into());
    let mut numbered = HashMap::new();
    if let Ok(entries) = fs::read_dir(tmp) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.chars().all(|c| c.is_ascii_digit()) && entry.path().is_file() {
                numbered.insert(name.clone(), format!("media_{name}"));
            }
        }
    }
    Ok(numbered)
}
