use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::constants::CURRENT_DB_NAME;
use crate::dictionary::database::*;
use crate::logging::*;
use anyhow::Error;

pub fn init_logging(level: &str) -> Result<(), Error> {
    let log_level = parse_log_level(level)?;
    let _ = load_logging_config(log_level);
    Ok(())
}

pub fn init_db_info(datadir: String, tablename: Option<String>) -> Result<Arc<DbInfo>, io::Error> {
    let dir_path = if datadir.is_empty() {
        "/data/term-squire-data".to_string()
    } else {
        datadir.clone()
    };

    let path = Path::new(&dir_path);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' does not exist", dir_path),
        ));
    }
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", dir_path),
        ));
    }

    let table_name = match tablename {
        Some(name) if !name.is_empty() => name,
        _ => "terms".to_string(),
    };

    let db_info = Arc::new(DbInfo {
        dir: dir_path,
        name: CURRENT_DB_NAME.to_string(),
        table_name,
    });

    Ok(db_info)
}

pub fn init_app_state(dbinfo: Arc<DbInfo>) -> Result<Arc<AppState>, io::Error> {
    Ok(Arc::new(AppState {
        db_info: dbinfo.clone(),
        terms_cache: Arc::new(Mutex::new(None)),
    }))
}
