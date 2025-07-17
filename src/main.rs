use axum::extract::DefaultBodyLimit;
use axum::middleware::AddExtension;
use axum::routing::{delete, get, post};
use axum::Extension;
use axum::Router;
use clap::{arg, command, Parser};
use std::sync::Arc;
use term_squire::constants::CURRENT_DB_NAME;
use term_squire::dictionary::{database::*, handlers::*};
use term_squire::logging::*;
use tracing::info;
/// simple dictionary
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port number used for server
    #[arg(short, long, default_value_t = 1234)]
    port: u64,
    /// Directory where data is stored
    #[arg(short, long, default_value = "/data")]
    datadir: Option<String>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let args = Args::parse();

    let db_name = Arc::new(DbName {
        name: format!(
            "{}/{}",
            args.datadir.unwrap_or_else(|| "/data".to_string()),
            CURRENT_DB_NAME
        ),
    });

    init_db(&db_name.name);

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
        .with_state(db_name)
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();
    info!("Starting server on port {}", args.port);
    axum::serve(listener, app).await.unwrap();
}
