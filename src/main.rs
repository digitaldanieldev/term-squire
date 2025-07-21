use axum::extract::{DefaultBodyLimit, State};
use axum::routing::{delete, get, post};
use axum::Router;
use clap::{arg, command, Parser};
use std::sync::{Arc, Mutex};
use term_squire::constants::CURRENT_DB_NAME;
use term_squire::dictionary::{database::*, handlers::*};
use term_squire::logging::*;
use tracing::info;

/// simple dictionary
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory where data is stored
    #[arg(short, long, default_value = "/data/term-squire-data")]
    data_dir: String,
    /// Logging level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    /// Port number used for server
    #[arg(short, long, default_value_t = 1234)]
    port: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let log_level = parse_log_level(&args.log_level)?;
    let _ = load_logging_config(log_level);

    let db_info = Arc::new(DbInfo {
        dir: if args.data_dir.is_empty() {
            "/data/term-squire-data".to_string()
        } else {
            args.data_dir.clone()
        },
        name: CURRENT_DB_NAME.to_string(),
        table_name: "terms".to_string(),
    });

    let app_state = Arc::new(AppState {
        db_info: db_info.clone(),
        terms_cache: Arc::new(Mutex::new(None)),
    });

    init_db(State(app_state.clone()))?;

    let app = Router::new()
        .route("/add_term_set", post(handle_add_term_set))
        .route("/database_management", get(handle_database_management))
        .route("/delete_term", delete(handle_delete_term))
        .route("/download_db_file", get(handle_download_db_file))
        .route("/import_dictionary", post(handle_import_dictionary_data))
        .route("/import_form", get(handle_import_form))
        .route("/insert_form", get(handle_insert_form))
        .route("/insert_term", post(handle_insert_term))
        .route("/search", get(handle_search_terms))
        .route(
            "/search_terms_by_term_set_id",
            get(handle_search_terms_by_term_set_id),
        )
        .route("/settings", get(handle_get_settings))
        .route("/terms", get(handle_terms))
        .route("/term_detail", get(handle_get_term_details))
        .route("/update_term", post(handle_update_term))
        .route("/upload_db_file", post(handle_upload_db_file))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(app_state)
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();
    info!("Starting server on port {}", args.port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
