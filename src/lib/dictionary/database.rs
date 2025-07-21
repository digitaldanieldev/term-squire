use crate::dictionary::handlers::SEARCH_CACHE;
use crate::import::parse::TermLanguageSet;
use axum::extract::State;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbInfo {
    pub dir: String,
    pub name: String,
    pub table_name: String,
}

impl DbInfo {
    pub fn path(&self) -> String {
        format!("{}/{}.sqlite", self.dir, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db_info: Arc<DbInfo>,
    pub terms_cache: Arc<Mutex<Option<Vec<TermsList>>>>,
}

pub fn clear_terms_cache(app_state: &Arc<AppState>) {
    *app_state.terms_cache.lock().unwrap() = None;
}

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

pub fn add_term(
    State(app_state): State<Arc<AppState>>,
    term_set: &TermLanguageSet,
) -> Result<(), rusqlite::Error> {
    debug!("Add term: {:?}", term_set);
    let conn = connect_db(State(app_state.clone()))?;
    let term_set_id = get_max_term_set_id(State(app_state.clone()))? + 1;
    let insert_sql = format!(
        "INSERT INTO {} (
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
        app_state.db_info.table_name
    );
    conn.execute(
        &insert_sql,
        rusqlite::params![
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
    State(app_state): State<Arc<AppState>>,
    existing_term_set_id: i32,
    term_set: &TermLanguageSet,
) -> Result<(), rusqlite::Error> {
    debug!(
        "Add term {:?} to term_set: {:?}",
        term_set, existing_term_set_id
    );
    let conn = connect_db(State(app_state.clone()))?;
    let insert_sql = format!(
        "INSERT INTO {} (
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
        app_state.db_info.table_name
    );
    conn.execute(
        &insert_sql,
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
pub fn connect_db(State(app_state): State<Arc<AppState>>) -> Result<Connection, rusqlite::Error> {
    debug!("Connect db: {:?}", app_state.db_info.table_name);
    Connection::open(app_state.db_info.path())
}

pub fn check_termset_count(
    State(app_state): State<Arc<AppState>>,
    term_id: i32,
) -> Result<i32, rusqlite::Error> {
    debug!("Check termset count: {:?}", term_id);
    let conn = connect_db(State(app_state.clone()))?;
    let sql = format!(
        "SELECT COUNT(*) FROM {} WHERE term_id = ?1",
        app_state.db_info.table_name
    );
    let termset_count: i32 = conn.query_row(&sql, [&term_id], |row| row.get(0))?;
    Ok(termset_count)
}

pub fn create_terms_table(State(app_state): State<Arc<AppState>>) -> Result<()> {
    debug!("Create terms table: {}", app_state.db_info.table_name);
    let conn = connect_db(State(app_state.clone()))?;
    let create_table_sql = format!(
        "
        BEGIN;
        CREATE TABLE IF NOT EXISTS {} (
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
        app_state.db_info.table_name,
    );

    conn.execute_batch(&create_table_sql)?;

    Ok(())
}

pub fn current_epoch() -> i64 {
    let now = Utc::now();
    let timestamp = now.timestamp();
    timestamp
}

pub fn create_unique_values_tables(State(app_state): State<Arc<AppState>>) -> Result<()> {
    debug!(
        "Create unique values tables: {:?}",
        app_state.db_info.table_name
    );
    let conn = connect_db(State(app_state))?;

    conn.execute_batch(
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
    )?;

    Ok(())
}

pub fn delete_term(
    State(app_state): State<Arc<AppState>>,
    term_id: i32,
) -> Result<(), rusqlite::Error> {
    debug!("Delete term: {:?}", term_id);
    let conn = connect_db(State(app_state.clone()))?;
    debug!("Connecting to DB to delete term_id: {}", term_id);
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

pub fn delete_termset(
    State(app_state): State<Arc<AppState>>,
    termset_to_delete: i32,
) -> Result<(), rusqlite::Error> {
    debug!("Delete termset: {:?}", termset_to_delete);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
        "DELETE FROM {} WHERE term_set_id = ?1",
        app_state.db_info.table_name
    );

    conn.execute(&sql, [&termset_to_delete])?;

    Ok(())
}

pub fn extract_and_insert_unique_values(State(app_state): State<Arc<AppState>>) -> Result<()> {
    debug!("Extract and insert unique values");
    let conn = connect_db(State(app_state.clone()))?;

    conn.execute("BEGIN;", params![])?;

    conn.execute("DELETE FROM unique_languages;", params![])?;
    conn.execute("DELETE FROM unique_term_types;", params![])?;
    conn.execute("DELETE FROM unique_creator_ids;", params![])?;
    conn.execute("DELETE FROM unique_updater_ids;", params![])?;
    conn.execute("DELETE FROM unique_subjects;", params![])?;
    conn.execute("DELETE FROM unique_sources;", params![])?;
    conn.execute("DELETE FROM unique_users;", params![])?;
    conn.execute("DELETE FROM unique_attributes;", params![])?;

    let table = &app_state.db_info.table_name;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_languages (language)
             SELECT DISTINCT language FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_term_types (term_type)
             SELECT DISTINCT term_type FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_creator_ids (creator_id)
             SELECT DISTINCT creator_id FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_updater_ids (updater_id)
             SELECT DISTINCT updater_id FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_subjects (subject)
             SELECT DISTINCT subject FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_sources (source)
             SELECT DISTINCT source FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_users (user)
             SELECT DISTINCT user FROM {};",
            table
        ),
        params![],
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO unique_attributes (attributes)
             SELECT DISTINCT attributes FROM {};",
            table
        ),
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

pub fn init_db(State(app_state): State<Arc<AppState>>) -> Result<()> {
    debug!(
        "Initializing database for {:?}",
        app_state.db_info.table_name
    );

    create_terms_table(State(app_state.clone()))?;
    create_unique_values_tables(State(app_state.clone()))?;

    let all_terms = get_all_terms(State(app_state.clone()))?;

    {
        // Populate app_state cache
        let mut cache = app_state.terms_cache.lock().unwrap();
        *cache = Some(all_terms.clone());
        debug!(
            "Terms cache populated with {} terms.",
            cache.as_ref().map_or(0, |c| c.len())
        );
    }

    {
        // Populate global request-level SEARCH_CACHE with all terms under wildcard key "*:*"
        let mut search_cache = SEARCH_CACHE.write().unwrap();
        search_cache.insert("*:*".to_string(), all_terms);
        debug!("SEARCH_CACHE wildcard '*:*' populated with all terms.");
    }

    Ok(())
}

pub fn get_all_terms(
    State(app_state): State<Arc<AppState>>,
) -> Result<Vec<TermsList>, rusqlite::Error> {
    debug!("Get all terms: {:?}", app_state.db_info.table_name);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
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
        FROM {}",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;

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

    dictionary_iter.collect()
}

pub fn get_max_id_terms(State(app_state): State<Arc<AppState>>) -> Result<i32, String> {
    debug!("Get max id terms: {:?}", app_state.db_info.table_name);
    let hid = get_max_term_id(State(app_state));

    match hid {
        Ok(val) => Ok(val),
        Err(_err) => Err("Failed to get the highest ID".to_string()),
    }
}

pub fn get_max_id_termsets(State(app_state): State<Arc<AppState>>) -> Result<i32, String> {
    debug!("Get max id termsets: {:?}", app_state.db_info.table_name);
    let hid = get_max_term_set_id(State(app_state));

    match hid {
        Ok(val) => Ok(val),
        Err(_err) => Err("Failed to get the highest ID".to_string()),
    }
}

pub fn get_max_term_id(State(app_state): State<Arc<AppState>>) -> Result<i32, rusqlite::Error> {
    debug!("Get max term id: {:?}", app_state.db_info.table_name);
    let conn = connect_db(State(app_state.clone()))?;
    let sql = format!(
        "SELECT COALESCE(MAX(term_id), 0) FROM {}",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;
    let highest_id: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(highest_id)
}

pub fn get_max_term_set_id(State(app_state): State<Arc<AppState>>) -> Result<i32, rusqlite::Error> {
    debug!("Get max termset id: {:?}", app_state.db_info.table_name);
    let conn = connect_db(State(app_state.clone()))?;
    let sql = format!(
        "SELECT COALESCE(MAX(term_set_id), 0) FROM {}",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;
    let highest_id: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(highest_id)
}

pub fn get_term_by_id(
    State(app_state): State<Arc<AppState>>,
    term_id: i32,
) -> Result<Option<TermsList>, rusqlite::Error> {
    debug!("Get term by id: {:?}", term_id);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
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
        FROM {} 
        WHERE term_id = ?",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;
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
    State(app_state): State<Arc<AppState>>,
    term: &str,
    language: &str,
) -> Result<Option<i32>, rusqlite::Error> {
    debug!("Get termset id: term:{:?}, language:{:?}", term, language);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
        "SELECT term_set_id FROM {} WHERE term = ? AND language = ?",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;
    let term_set_id: Option<i32> = stmt
        .query_row(params![term, language], |row| row.get(0))
        .optional()?;

    Ok(term_set_id)
}

pub fn get_term_set_id_by_term_id(
    State(app_state): State<Arc<AppState>>,
    term_id: i32,
) -> Result<Option<i32>, rusqlite::Error> {
    debug!("Get term set id by term id: {:?}", term_id);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
        "SELECT term_set_id FROM {} WHERE term_id = ? LIMIT 1",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;
    let term_set_id: Option<i32> = stmt
        .query_row(params![term_id], |row| row.get(0))
        .optional()?;

    Ok(term_set_id)
}
pub fn search_terms(
    State(app_state): State<Arc<AppState>>,
    term: &str,
    language: &str,
) -> Result<Vec<TermsList>, rusqlite::Error> {
    let cache_guard = app_state.terms_cache.lock().unwrap();

    if let Some(cached_terms) = &*cache_guard {
        debug!("Searching terms from cache.");

        let matches: Vec<TermsList> = cached_terms
            .iter()
            .filter(|term_entry| {
                let term_str = term_entry.term_language_set.term.as_deref().unwrap_or("");
                if !term_str.contains(term) {
                    return false;
                }

                if !language.is_empty() {
                    let lang_str = term_entry
                        .term_language_set
                        .language
                        .as_deref()
                        .unwrap_or("");
                    if lang_str != language {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        debug!("Found {} terms from cache matching query.", matches.len());
        return Ok(matches);
    }

    debug!("Terms cache is empty, querying database directly.");

    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
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
        FROM {} 
        WHERE term LIKE ? AND (language = ? OR ? = '')",
        app_state.db_info.table_name
    );

    let term_pattern = format!("%{}%", term);
    let mut stmt = conn.prepare(&sql)?;

    let terms_iter = stmt.query_map([&term_pattern, language, language], |row| {
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

    terms_iter.collect()
}

pub fn search_terms_by_term_set_id(
    State(app_state): State<Arc<AppState>>,
    term_set_id: i32,
) -> Result<Vec<TermsList>, rusqlite::Error> {
    debug!("Search terms by term set id: {:?}", term_set_id);
    let conn = connect_db(State(app_state.clone()))?;

    let sql = format!(
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
        FROM {} 
        WHERE term_set_id = ?",
        app_state.db_info.table_name
    );

    let mut stmt = conn.prepare(&sql)?;

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

    terms_iter.collect()
}

pub fn update_term(
    State(app_state): State<Arc<AppState>>,
    term_id_to_update: i32,
    termset_update: &TermLanguageSet,
) -> Result<(), rusqlite::Error> {
    debug!(
        "Update term: term_id_to_update: {:?}, termset_update: {:?}",
        term_id_to_update, termset_update
    );
    let conn = connect_db(State(app_state.clone()))?;

    let query = format!(
        "
        UPDATE {} 
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
        ",
        app_state.db_info.table_name
    );

    conn.execute(
        &query,
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
            termset_update.url.as_deref(),
            termset_update.context.as_deref(),
            termset_update.definition.as_deref(),
        ],
    )?;

    Ok(())
}
