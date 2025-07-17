use crate::import::parse::TermLanguageSet;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TermsList {
    pub term_id: i32,
    pub term_set_id: i32,
    pub term_language_set: TermLanguageSet,
}

impl TermsList {
    pub fn term_or_default(&self) -> &str {
        self.term_language_set.term.as_deref().unwrap_or("No Term")
    }

    pub fn language_or_default(&self) -> &str {
        self.term_language_set
            .language
            .as_deref()
            .unwrap_or("No Language")
    }

    pub fn term_type_or_default(&self) -> &str {
        self.term_language_set
            .term_type
            .as_deref()
            .unwrap_or("No Term Type")
    }

    pub fn created_by_or_default(&self) -> &str {
        self.term_language_set
            .creator_id
            .as_deref()
            .unwrap_or("No Creator")
    }

    pub fn created_date_or_default(&self) -> String {
        self.term_language_set
            .creation_timestamp
            .map_or("No Date".to_string(), |dt| dt.to_string())
    }

    pub fn updated_by_or_default(&self) -> &str {
        self.term_language_set
            .updater_id
            .as_deref()
            .unwrap_or("No Updater")
    }

    pub fn updated_date_or_default(&self) -> String {
        self.term_language_set
            .update_timestamp
            .map_or("No Date".to_string(), |dt| dt.to_string())
    }

    pub fn subject_or_default(&self) -> &str {
        self.term_language_set
            .subject
            .as_deref()
            .unwrap_or("No Subject")
    }

    pub fn source_or_default(&self) -> &str {
        self.term_language_set
            .source
            .as_deref()
            .unwrap_or("No Source")
    }

    pub fn user_or_default(&self) -> &str {
        self.term_language_set.user.as_deref().unwrap_or("No User")
    }

    pub fn attributes_or_default(&self) -> &str {
        self.term_language_set
            .attributes
            .as_deref()
            .unwrap_or("No Attributes")
    }

    pub fn remark_or_default(&self) -> &str {
        self.term_language_set
            .remark
            .as_deref()
            .unwrap_or("No Remark")
    }

    pub fn url_or_default(&self) -> &str {
        self.term_language_set.url.as_deref().unwrap_or("No URL")
    }

    pub fn context_or_default(&self) -> &str {
        self.term_language_set
            .context
            .as_deref()
            .unwrap_or("No Context")
    }

    pub fn definition_or_default(&self) -> &str {
        self.term_language_set
            .definition
            .as_deref()
            .unwrap_or("No Definition")
    }
}

use axum::Extension;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbName {
    pub name: String,
}

pub fn add_term(db_name: &str, term_set: &TermLanguageSet) -> Result<(), rusqlite::Error> {
    let conn = connect_db(db_name);

    let term_set_id = match get_max_term_set_id(db_name)? {
        id => id + 1,
    };

    conn.execute(
        "INSERT INTO terms (
            term_set_id, 
            term, 
            language, 
            term_type, 
            creator_id, 
            creation_timestamp,
            updater_id, 
            update_timestamp, 
            subject, 
            source,
            user, 
            attributes, 
            remark, 
            url,
            context, 
            definition
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            term_set_id,
            term_set.term,
            term_set.language,
            term_set.term_type,
            term_set.creator_id,
            term_set.creation_timestamp,
            term_set.updater_id,
            term_set.update_timestamp,
            term_set.subject,
            term_set.source,
            term_set.user,
            term_set.attributes,
            term_set.remark,
            term_set.url,
            term_set.context,
            term_set.definition,
        ],
    )?;

    Ok(())
}

pub fn add_term_to_term_set(
    db_name: &str,
    existing_term_set_id: i32,
    term_set: &TermLanguageSet,
) -> Result<(), rusqlite::Error> {
    let conn = connect_db(db_name);
    println!("Debug: existing_term_set_id = {}", existing_term_set_id);

    conn.execute(
        "INSERT INTO terms (
            term_set_id, 
            term, 
            language, 
            term_type, 
            creator_id, 
            creation_timestamp,
            updater_id, 
            update_timestamp, 
            subject, 
            source, 
            user,
            attributes, 
            remark, 
            url, 
            context, 
            definition
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            existing_term_set_id,
            term_set.term,
            term_set.language,
            term_set.term_type,
            term_set.creator_id,
            term_set.creation_timestamp,
            term_set.updater_id,
            term_set.update_timestamp,
            term_set.subject,
            term_set.source,
            term_set.user,
            term_set.attributes,
            term_set.remark,
            term_set.url,
            term_set.context,
            term_set.definition,
        ],
    )?;

    Ok(())
}

pub fn connect_db(db_name: &str) -> Connection {
    let conn = Connection::open(format!("{}", db_name)).unwrap();
    conn
}

pub fn check_termset_count(db_name: &str, term_id: i32) -> Result<i32, rusqlite::Error> {
    let conn = connect_db(db_name);

    let termset_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM terms WHERE term_id = ?1",
        [&term_id],
        |row| row.get(0),
    )?;
    Ok(termset_count)
}

pub fn create_terms_table(db_name: &str) {
    info!("Creating terms table.");
    connect_db(db_name)
        .execute_batch(
            "
            BEGIN;
            CREATE TABLE IF NOT EXISTS terms (
                term_id INTEGER PRIMARY KEY,
                term_set_id INTEGER,
                term TEXT,
                language TEXT,
                term_type TEXT,
                creator_id TEXT,
                creation_timestamp INTEGER,
                updater_id TEXT,
                update_timestamp INTEGER,
                subject TEXT,
                source TEXT,
                user TEXT,
                attributes TEXT,
                remark TEXT,
                url TEXT,
                context TEXT,
                definition TEXT
            );
            COMMIT;
            ",
        )
        .unwrap();
}

pub fn current_epoch() -> i64 {
    let now = Utc::now();
    let timestamp = now.timestamp();
    timestamp
}

pub fn create_unique_values_tables(db_name: &str) {
    let connection = connect_db(db_name);

    connection
        .execute_batch(
            "
        BEGIN;

        CREATE TABLE IF NOT EXISTS unique_languages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            language TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_term_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            term_type TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_creator_ids (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            creator_id TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_updater_ids (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            updater_id TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_subjects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            subject TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS unique_attributes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            attributes TEXT UNIQUE
        );

        COMMIT;
        ",
        )
        .unwrap();
}

pub fn delete_term(db_name: &str, term_id: i32) -> Result<(), rusqlite::Error> {
    let conn = connect_db(db_name);

    conn.execute("BEGIN TRANSACTION", [])?;

    println!("Executing DELETE for term_id: {}", term_id);

    let result = conn.execute("DELETE FROM terms WHERE term_id = ?1", params![term_id])?;

    println!("Deleted {} rows", result);

    if result == 0 {
        conn.execute("ROLLBACK", [])?;
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }

    conn.execute("COMMIT", [])?;

    Ok(())
}

pub fn delete_termset(db_name: &str, termset_to_delete: i32) -> Result<(), rusqlite::Error> {
    let conn = connect_db(db_name);

    conn.execute(
        "DELETE FROM terms WHERE term_set_id = ?1",
        [&termset_to_delete],
    )?;

    Ok(())
}

pub fn extract_and_insert_unique_values(db_name: &str) -> Result<()> {
    let conn = connect_db(db_name);

    conn.execute("BEGIN;", params![])?;

    conn.execute("DELETE FROM unique_languages;", params![])?;
    conn.execute("DELETE FROM unique_term_types;", params![])?;
    conn.execute("DELETE FROM unique_creator_ids;", params![])?;
    conn.execute("DELETE FROM unique_updater_ids;", params![])?;
    conn.execute("DELETE FROM unique_subjects;", params![])?;
    conn.execute("DELETE FROM unique_sources;", params![])?;
    conn.execute("DELETE FROM unique_users;", params![])?;
    conn.execute("DELETE FROM unique_attributes;", params![])?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_languages (language)
        SELECT DISTINCT language FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_term_types (term_type)
        SELECT DISTINCT term_type FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_creator_ids (creator_id)
        SELECT DISTINCT creator_id FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_updater_ids (updater_id)
        SELECT DISTINCT updater_id FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_subjects (subject)
        SELECT DISTINCT subject FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_sources (source)
        SELECT DISTINCT source FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_users (user)
        SELECT DISTINCT user FROM terms;
        ",
        params![],
    )?;

    conn.execute(
        "
        INSERT OR IGNORE INTO unique_attributes (attributes)
        SELECT DISTINCT attributes FROM terms;
        ",
        params![],
    )?;

    conn.execute("COMMIT;", params![])?;

    Ok(())
}

pub fn handle_insert_unique_values_result(result: Result<()>) {
    match result {
        Ok(_) => {
            println!("Unique values were successfully inserted.");
        }
        Err(e) => {
            eprintln!("Error inserting unique values: {}", e);
        }
    }
}

pub fn init_db(db_name: &str) {
    create_terms_table(db_name);
    create_unique_values_tables(db_name);
}

pub fn get_all_terms(db_name: &str) -> Result<Vec<TermsList>, rusqlite::Error> {
    let conn = connect_db(db_name);
    let mut stmt = conn.prepare(
        "SELECT 
            term_id,
            term_set_id, 
            term, 
            language, 
            term_type, 
            creator_id, 
            creation_timestamp, 
            updater_id, 
            update_timestamp, 
            subject, 
            source, 
            user, 
            attributes, 
            remark, 
            url, 
            context, 
            definition 
        FROM terms",
    )?;

    let dictionary_iter = stmt.query_map([], |row| {
        Ok(TermsList {
            term_id: row.get(0)?,
            term_set_id: row.get(1)?,
            term_language_set: TermLanguageSet {
                term: row.get(2)?,
                language: row.get(3)?,
                term_type: row.get(4)?,
                creator_id: row.get(5)?,
                creation_timestamp: row.get(6)?,
                updater_id: row.get(7)?,
                update_timestamp: row.get(8)?,
                subject: row.get(9)?,
                source: row.get(10)?,
                user: row.get(11)?,
                attributes: row.get(12)?,
                remark: row.get(13)?,
                url: row.get(14)?,
                context: row.get(15)?,
                definition: row.get(16)?,
            },
        })
    })?;

    let terms: Result<Vec<TermsList>, rusqlite::Error> = dictionary_iter.collect();
    terms
}

pub fn get_max_id_terms(db_name: &str) -> Result<i32, String> {
    let hid = get_max_term_id(db_name);

    match hid {
        Ok(val) => Ok(val),
        Err(_err) => Err("Failed to get the highest ID".to_string()),
    }
}

pub fn get_max_id_termsets(db_name: &str) -> Result<i32, String> {
    let hid = get_max_term_set_id(db_name);

    match hid {
        Ok(val) => Ok(val),
        Err(_err) => Err("Failed to get the highest ID".to_string()),
    }
}

pub fn get_max_term_id(db_name: &str) -> Result<i32, rusqlite::Error> {
    let conn = connect_db(db_name);
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(term_id), 0) FROM terms")
        .unwrap();

    let highest_id: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
    Ok(highest_id)
}

pub fn get_max_term_set_id(db_name: &str) -> Result<i32, rusqlite::Error> {
    let conn = connect_db(db_name);
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(term_set_id), 0) FROM terms")
        .unwrap();

    let highest_id: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
    Ok(highest_id)
}

pub fn get_term_by_id(db_name: &str, term_id: i32) -> Result<Option<TermsList>, rusqlite::Error> {
    let conn = connect_db(db_name);

    let sql = "SELECT 
                    term_id,
                    term_set_id, 
                    term, 
                    language, 
                    term_type, 
                    creator_id, 
                    creation_timestamp, 
                    updater_id, 
                    update_timestamp, 
                    subject, 
                    source, 
                    user, 
                    attributes, 
                    remark, 
                    url, 
                    context, 
                    definition 
                FROM terms 
                WHERE term_id = ?";
    let mut stmt = conn.prepare(sql)?;

    let mut rows = stmt.query([term_id])?;

    if let Some(row) = rows.next()? {
        let term = TermsList {
            term_id: row.get(0)?,
            term_set_id: row.get(1)?,
            term_language_set: TermLanguageSet {
                term: row.get(2)?,
                language: row.get(3)?,
                term_type: row.get(4)?,
                creator_id: row.get(5)?,
                creation_timestamp: row.get(6)?,
                updater_id: row.get(7)?,
                update_timestamp: row.get(8)?,
                subject: row.get(9)?,
                source: row.get(10)?,
                user: row.get(11)?,
                attributes: row.get(12)?,
                remark: row.get(13)?,
                url: row.get(14)?,
                context: row.get(15)?,
                definition: row.get(16)?,
            },
        };
        Ok(Some(term))
    } else {
        Ok(None)
    }
}

pub fn get_term_set_id(
    db_name: &str,
    term: &str,
    language: &str,
) -> Result<Option<i32>, rusqlite::Error> {
    let conn = Connection::open(db_name)?;

    let mut stmt = conn.prepare("SELECT term_set_id FROM terms WHERE term = ? AND language = ?")?;

    let term_set_id: Option<i32> = stmt
        .query_row(params![term, language], |row| row.get(0))
        .optional()?;

    Ok(term_set_id)
}

pub fn get_term_set_id_by_term_id(
    db_name: &str,
    term_id: i32,
) -> Result<Option<i32>, rusqlite::Error> {
    let conn = connect_db(db_name);
    let mut stmt = conn.prepare("SELECT term_set_id FROM terms WHERE term_id = ? LIMIT 1")?;
    let term_set_id: Option<i32> = stmt
        .query_row(params![term_id], |row| row.get(0))
        .optional()?;
    Ok(term_set_id)
}

pub fn search_terms(
    db_name: &str,
    term_select: &str,
    language_select: &str,
) -> Result<Vec<TermsList>, rusqlite::Error> {
    let conn = connect_db(db_name);

    let sql = "SELECT 
                    term_id,
                    term_set_id,                         
                    term, 
                    language,
                    term_type, 
                    creator_id, 
                    creation_timestamp, 
                    updater_id, 
                    update_timestamp, 
                    subject, 
                    source, 
                    user, 
                    attributes, 
                    remark, 
                    url, 
                    context, 
                    definition 
               FROM terms 
               WHERE (term LIKE ? OR subject LIKE ? OR remark LIKE ? OR context LIKE ? OR definition LIKE ?) 
                 AND language LIKE ?";

    let mut stmt = conn.prepare(sql)?;

    let fuzzy_term = format!("%{}%", term_select);
    let fuzzy_language = format!("%{}%", language_select);

    let params = rusqlite::params![
        fuzzy_term.as_str(),
        fuzzy_term.as_str(),
        fuzzy_term.as_str(),
        fuzzy_term.as_str(),
        fuzzy_term.as_str(),
        fuzzy_language.as_str()
    ];

    let dictionary_iter = stmt.query_map(params, |row| {
        Ok(TermsList {
            term_id: row.get(0)?,
            term_set_id: row.get(1)?,
            term_language_set: TermLanguageSet {
                term: row.get(2)?,
                language: row.get(3)?,
                term_type: row.get(4)?,
                creator_id: row.get(5)?,
                creation_timestamp: row.get(6)?,
                updater_id: row.get(7)?,
                update_timestamp: row.get(8)?,
                subject: row.get(9)?,
                source: row.get(10)?,
                user: row.get(11)?,
                attributes: row.get(12)?,
                remark: row.get(13)?,
                url: row.get(14)?,
                context: row.get(15)?,
                definition: row.get(16)?,
            },
        })
    })?;

    let terms: Vec<TermsList> = dictionary_iter.collect::<Result<_, _>>()?;
    Ok(terms)
}

pub fn search_terms_by_term_set_id(
    db_name: &str,
    term_set_id: i32,
) -> Result<Vec<TermsList>, rusqlite::Error> {
    let conn = connect_db(db_name);
    let mut stmt = conn.prepare(
        "SELECT 
            term_id, 
            term_set_id, 
            term, 
            language, 
            term_type, 
            creator_id, 
            creation_timestamp, 
            updater_id, 
            update_timestamp, 
            subject, 
            source, 
            user, 
            attributes, 
            remark, 
            url, 
            context, 
            definition 
        FROM terms 
        WHERE term_set_id = ?",
    )?;

    let terms_iter = stmt.query_map(params![term_set_id], |row| {
        Ok(TermsList {
            term_id: row.get(0)?,
            term_set_id: row.get(1)?,
            term_language_set: TermLanguageSet {
                term: row.get(2)?,
                language: row.get(3)?,
                term_type: row.get(4)?,
                creator_id: row.get(5)?,
                creation_timestamp: row.get(6)?,
                updater_id: row.get(7)?,
                update_timestamp: row.get(8)?,
                subject: row.get(9)?,
                source: row.get(10)?,
                user: row.get(11)?,
                attributes: row.get(12)?,
                remark: row.get(13)?,
                url: row.get(14)?,
                context: row.get(15)?,
                definition: row.get(16)?,
            },
        })
    })?;

    let terms: Vec<TermsList> = terms_iter.collect::<Result<_, _>>()?;
    Ok(terms)
}

pub fn update_term(
    db_name: &str,
    term_id_to_update: i32,
    termset_update: &TermLanguageSet,
) -> Result<(), rusqlite::Error> {
    let conn = connect_db(db_name);

    let query = "
        UPDATE terms 
        SET 
            term = COALESCE(?2, term),
            language = COALESCE(?3, language),
            term_type = COALESCE(?4, term_type),
            creator_id = COALESCE(?5, creator_id),
            creation_timestamp = COALESCE(?6, creation_timestamp),
            updater_id = COALESCE(?7, updater_id),
            update_timestamp = COALESCE(?8, update_timestamp),
            subject = COALESCE(?9, subject),
            source = COALESCE(?10, source),
            user = COALESCE(?11, user),
            attributes = COALESCE(?12, attributes),
            remark = COALESCE(?13, remark),
            url = COALESCE(?14, url),
            context = COALESCE(?15, context),
            definition = COALESCE(?16, definition)
        WHERE term_id = ?1
    ";

    conn.execute(
        query,
        params![
            term_id_to_update,
            termset_update.term.as_deref(),
            termset_update.language.as_deref(),
            termset_update.term_type.as_deref(),
            termset_update.creator_id.as_deref(),
            termset_update.creation_timestamp,
            termset_update.updater_id.as_deref(),
            termset_update.update_timestamp,
            termset_update.subject.as_deref(),
            termset_update.source.as_deref(),
            termset_update.user.as_deref(),
            termset_update.attributes.as_deref(),
            termset_update.remark.as_deref(),
            termset_update.url.as_deref(), // Handle optional URL
            termset_update.context.as_deref(),
            termset_update.definition.as_deref(),
        ],
    )?;

    Ok(())
}
