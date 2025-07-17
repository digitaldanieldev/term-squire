use askama::Template;
use axum::{
    debug_handler,
    extract::{Json, State},
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json as AxumJson, Response},
};
use axum::{
    extract::{Multipart, Query},
    Form,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{error, info};

use crate::{
    dictionary::database::{
        add_term, add_term_to_term_set, current_epoch, delete_term,
        extract_and_insert_unique_values, get_all_terms, get_term_by_id, search_terms,
        search_terms_by_term_set_id, update_term, DbName, TermsList,
    },
    import::{parse::TermLanguageSet, process::import_dictionary_data},
};

// cache to store search results.
lazy_static! {
    static ref SEARCH_CACHE: RwLock<HashMap<String, Vec<TermsList>>> = RwLock::new(HashMap::new());
}

// clears the search cache
pub fn clear_cache() {
    SEARCH_CACHE.write().unwrap().clear();
}

// add term set to term
#[derive(Debug, Deserialize)]
pub struct AddTermSetRequest {
    existing_term_set_id: i32,
    term_language_set: TermLanguageSet,
}

#[debug_handler]
pub async fn handle_add_term_set(
    State(db_name): State<Arc<DbName>>,
    Json(payload): Json<AddTermSetRequest>,
) -> impl IntoResponse {
    let existing_term_set_id = payload.existing_term_set_id;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.creation_timestamp = Some(now);
    term_set.update_timestamp = Some(now);

    info!(
        "Adding term set to existing term ID: {}",
        existing_term_set_id
    );

    let db_name_str = &db_name.name;

    match add_term_to_term_set(db_name_str, existing_term_set_id, &term_set) {
        Ok(_) => {
            info!("Term set added successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(db_name_str);
            (
                StatusCode::OK,
                "Term set added to existing term successfully",
            )
                .into_response()
        }
        Err(err) => {
            error!("Failed to add term set: {}", &err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to add term set to existing term: {err}"),
            )
                .into_response()
        }
    }
}

// delete term
#[derive(Debug, Deserialize)]
pub struct DeleteTermRequest {
    term_id: i32,
}

#[debug_handler]
pub async fn handle_delete_term(
    State(db_name): State<Arc<DbName>>,
    Query(params): Query<DeleteTermRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let term_id = params.term_id;

    info!("Deleting term with ID: {}", term_id);

    match delete_term(&db_name_str, term_id) {
        Ok(_) => {
            info!("Term deleted successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(&db_name_str);
            (StatusCode::OK, "Term deleted successfully").into_response()
        }
        Err(rusqlite::Error::ExecuteReturnedResults) => {
            info!("Term not found or not deleted.");
            (
                StatusCode::NOT_FOUND,
                "Term not found or not deleted".to_string(),
            )
                .into_response()
        }
        Err(err) => {
            error!("Failed to delete term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete term: {err}"),
            )
                .into_response()
        }
    }
}

// insert term
#[derive(Debug, Deserialize)]
pub struct InsertTermRequest {
    term_language_set: TermLanguageSet,
}

pub async fn handle_insert_term(
    State(db_name): State<Arc<DbName>>,
    Json(payload): Json<InsertTermRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.creation_timestamp = Some(now);
    term_set.update_timestamp = Some(now);

    info!("Inserting new term into database.");

    match add_term(&db_name_str, &term_set) {
        Ok(_) => {
            info!("Term inserted successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(&db_name_str);
            (StatusCode::OK, "Term inserted successfully").into_response()
        }
        Err(err) => {
            error!("Failed to insert term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to insert term: {err}"),
            )
                .into_response()
        }
    }
}

// import dictionary data
pub async fn handle_import_dictionary_data(
    State(db_name): State<Arc<DbName>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(mut field) = match multipart.next_field().await {
        Ok(Some(field)) => Some(field),
        Ok(None) => None,
        Err(err) => {
            error!("Failed to read field: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read field: {err}"),
            )
                .into_response();
        }
    } {
        let name = match field.name() {
            Some(name) => name.to_string(),
            None => {
                error!("Field does not have a name");
                return (
                    StatusCode::BAD_REQUEST,
                    "Field does not have a name".to_string(),
                )
                    .into_response();
            }
        };

        info!("Processing field: {}", name);

        if name == "dictionaryFile" {
            info!("Receiving dictionary file {}", name);

            let file_path = PathBuf::from("uploaded_dictionary.mtf");
            let mut file = match File::create(&file_path).await {
                Ok(file) => file,
                Err(err) => {
                    error!("Failed to create file: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create file: {err}"),
                    )
                        .into_response();
                }
            };

            while let Some(chunk) = match field.chunk().await {
                Ok(Some(chunk)) => Some(chunk),
                Ok(None) => {
                    info!("No more chunks available");
                    None
                }
                Err(err) => {
                    error!("Failed to read chunk: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read chunk: {err}"),
                    )
                        .into_response();
                }
            } {
                info!("Writing chunk of size {}", chunk.len());
                if let Err(err) = file.write_all(&chunk).await {
                    error!("Failed to write to file: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write to file: {err}"),
                    )
                        .into_response();
                }
            }

            if let Err(err) = file.sync_all().await {
                error!("Failed to flush file: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to flush file: {err}"),
                )
                    .into_response();
            }

            info!("Dictionary file {} uploaded successfully.", name);

            if let Err(err) = import_dictionary_data(file_path.to_string_lossy().as_ref()).await {
                error!("Failed to import dictionary: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to import dictionary: {err}"),
                )
                    .into_response();
            }
            clear_cache();
            return (StatusCode::OK, "Dictionary imported successfully").into_response();
        }
    }

    (StatusCode::BAD_REQUEST, "No file was uploaded".to_string()).into_response()
}

// import dictionary file
#[derive(Template)]
#[template(path = "import_form.html")]
struct ImportFormTemplate;

pub async fn handle_import_form() -> Html<String> {
    info!("Serving import form.");
    let template = ImportFormTemplate;
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

#[derive(Template)]
#[template(path = "insert_form.html")]
pub struct InsertFormTemplate;

pub async fn handle_insert_form() -> impl IntoResponse {
    info!("Serving insert form.");
    let template = InsertFormTemplate;
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

// settings
#[derive(Deserialize)]
pub struct SettingsForm {
    columns: Vec<String>,
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    pub columns: Vec<String>,
}

pub async fn handle_settings_update(Form(form): Form<SettingsForm>) -> impl IntoResponse {
    let selected_columns = form.columns;

    info!(
        "Processing settings update with columns: {:?}",
        selected_columns
    );

    let template = SettingsTemplate {
        columns: selected_columns,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

pub async fn handle_get_settings() -> impl IntoResponse {
    let stored_columns = vec![];

    info!("Fetching stored settings.");

    let template = SettingsTemplate {
        columns: stored_columns,
    };
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

// terms
#[derive(Template)]
#[template(path = "terms.html")]
pub struct TermsTemplate {
    pub terms: Vec<TermsList>,
}

pub async fn handle_terms(State(db_name): State<Arc<DbName>>) -> Html<String> {
    let db_name_str = &db_name.name;

    info!("Fetching terms.");

    match get_all_terms(&db_name_str) {
        Ok(terms) => {
            info!("Terms fetched successfully.");
            let template = TermsTemplate { terms };
            Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template rendering error".to_string()),
            )
        }
        Err(err) => {
            error!("Failed to get data: {}", err);
            Html(format!("<h1>Failed to get data: {err}</h1>"))
        }
    }
}

// term details
#[derive(Debug, Deserialize)]
pub struct TermDetailRequest {
    term_id: i32,
}

#[derive(Template)]
#[template(path = "term_detail.html")]
pub struct TermDetailTemplate {
    pub term: TermsList,
}

pub async fn handle_get_term_details(
    State(db_name): State<Arc<DbName>>,
    Query(params): Query<TermDetailRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let term_id = params.term_id;

    info!("Fetching details for term ID: {}", term_id);

    match get_term_by_id(&db_name_str, term_id) {
        Ok(Some(term)) => {
            info!("Term details fetched successfully.");
            let template = TermDetailTemplate { term };
            Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template rendering error".to_string()),
            )
        }
        Ok(None) => {
            info!("Term not found for ID: {}", term_id);
            Html("<h1>Term not found</h1>".to_string())
        }
        Err(err) => {
            error!("Failed to get term details: {}", err);
            Html(format!("<h1>Failed to get term details: {err}</h1>"))
        }
    }
}

// search terms list
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    term: String,
    language: String,
}

#[derive(Template)]
#[template(path = "search_form.html")]
pub struct SearchResultsTemplate {
    pub terms: Vec<TermsList>,
    pub count: usize,
}

pub async fn handle_search_terms(
    State(db_name): State<Arc<DbName>>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let term_select = params.term.clone();
    let language_select = params.language.clone();
    let cache_key = format!("{term_select}:{language_select}");

    // Check the cache first
    if let Some(cached_results) = SEARCH_CACHE.read().unwrap().get(&cache_key) {
        info!(
            "Cache hit for search term: {} and language: {}",
            term_select, language_select
        );
        return Json(cached_results.clone());
    }

    info!(
        "Cache miss for search term: {} and language: {}",
        term_select, language_select
    );

    // Perform the database search if cache miss
    match search_terms(&db_name_str, &term_select, &language_select) {
        Ok(terms) => {
            info!("Search completed successfully.");
            // Update the cache
            SEARCH_CACHE
                .write()
                .unwrap()
                .insert(cache_key, terms.clone());
            Json(terms)
        }
        Err(err) => {
            error!("Failed to search terms: {}", err);
            Json(vec![])
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchByTermSetIdRequest {
    term_set_id: i32,
}

pub async fn handle_search_terms_by_term_set_id(
    State(db_name): State<Arc<DbName>>,
    Query(params): Query<SearchByTermSetIdRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let term_set_id = params.term_set_id;

    info!("Searching for terms with term_set_id: {}", term_set_id);

    match search_terms_by_term_set_id(&db_name_str, term_set_id) {
        Ok(terms) => {
            info!("Search completed successfully.");
            Json(terms)
        }
        Err(err) => {
            error!("Failed to search terms: {}", err);
            Json(vec![])
        }
    }
}

// update term
#[derive(Debug, Deserialize)]
pub struct UpdateTermRequest {
    term_id: i32,
    term_language_set: TermLanguageSet,
}

pub async fn handle_update_term(
    State(db_name): State<Arc<DbName>>,
    Json(payload): Json<UpdateTermRequest>,
) -> impl IntoResponse {
    let db_name_str = &db_name.name;
    let term_id = payload.term_id;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.update_timestamp = Some(now);

    info!("Updating term ID: {}", term_id);

    match update_term(&db_name_str, term_id, &term_set) {
        Ok(_) => {
            info!("Term updated successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(&db_name_str);
            (StatusCode::OK, "Term updated successfully").into_response()
        }
        Err(err) => {
            error!("Failed to update term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update term: {err}"),
            )
                .into_response()
        }
    }
}

// manage database
#[derive(Template)]
#[template(path = "database_management.html")]
struct DatabaseManagementTemplate;

pub async fn handle_database_management() -> Html<String> {
    info!("Serving database management form.");
    let template = DatabaseManagementTemplate;
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

// download the database
pub async fn handle_download_db_file(State(db_name): State<Arc<DbName>>) -> impl IntoResponse {
    let db_file_path = &db_name.name;

    if !Path::new(db_file_path).exists() {
        error!("Database file not found: {}", db_file_path);
        return (StatusCode::NOT_FOUND, "Database file not found").into_response();
    }

    info!("Sending database file: {}", db_file_path);

    match File::open(db_file_path).await {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            if let Err(err) = file.read_to_end(&mut buffer).await {
                error!("Failed to read the database file: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read the database file: {err}"),
                )
                    .into_response();
            }

            let file_name = Path::new(db_file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(db_file_path);

            let mut response = Response::new(buffer.into());

            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );

            response.headers_mut().insert(
                axum::http::header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{file_name}\"")).unwrap(),
            );

            info!("Database file {} sent successfully.", db_file_path);

            response
        }
        Err(err) => {
            error!("Failed to open the database file: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open the database file: {err}"),
            )
                .into_response()
        }
    }
}

// upload database
pub async fn handle_upload_db_file(
    State(db_name): State<Arc<DbName>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db_file_path = &db_name.name;
    info!("Starting database file upload.");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field.file_name().unwrap_or("uploaded_file").to_string();
        let content_type = field.content_type().unwrap_or("unknown");

        if content_type != "application/octet-stream" && !file_name.ends_with(".db") {
            error!(
                "Invalid file type: {}. Only .db files are accepted.",
                content_type
            );
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid file type: {content_type}. Only .db files are accepted."),
            )
                .into_response();
        }

        let data = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("Failed to read uploaded file: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read uploaded file: {err}"),
                )
                    .into_response();
            }
        };

        info!("Writing uploaded file to path: {}", db_file_path);

        match File::create(db_file_path).await {
            Ok(mut file) => {
                if let Err(err) = file.write_all(&data).await {
                    error!("Failed to write to the database file: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write to the database file: {err}"),
                    )
                        .into_response();
                }
            }
            Err(err) => {
                error!("Failed to create the database file: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create the database file: {err}"),
                )
                    .into_response();
            }
        }

        info!("Database file uploaded successfully: {}", db_file_path);

        // Clear cache after uploading the database
        clear_cache();
        info!("Cache cleared after uploading the database.");

        return (
            StatusCode::OK,
            format!("Successfully uploaded and saved as '{db_file_path}'"),
        )
            .into_response();
    }

    error!("No file found in the upload request");
    (
        StatusCode::BAD_REQUEST,
        "No file found in the upload request".to_string(),
    )
        .into_response()
}
