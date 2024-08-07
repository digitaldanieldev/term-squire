use term_squire::dictionary::database::*;
use term_squire::import::parse::*;

use lazy_static::lazy_static;

lazy_static! {
    static ref TERM_SET_1: TermLanguageSet = TermLanguageSet {
        term: Some("term_1".to_string()),
        language: Some("en".to_string()),
        term_type: Some("noun".to_string()),
        creator_id: Some("user_1".to_string()),
        creation_timestamp: Some(current_epoch()),
        updater_id: None,
        update_timestamp: None,
        subject: None,
        source: None,
        user: None,
        attributes: None,
        remark: None,
        url: None,
        context: None,
        definition: None,
    };
    static ref TERM_SET_2: TermLanguageSet = TermLanguageSet {
        term: Some("term_2".to_string()),
        language: Some("nl".to_string()),
        term_type: Some("noun".to_string()),
        creator_id: Some("user_2".to_string()),
        creation_timestamp: Some(current_epoch()),
        updater_id: None,
        update_timestamp: None,
        subject: None,
        source: None,
        user: None,
        attributes: None,
        remark: None,
        url: None,
        context: None,
        definition: None,
    };
}

// helper functions for tests
pub fn create_test_db(db_name: &str) -> String {
    let db_path = format!("{}.sqlite", db_name);
    let _conn = connect_db(&db_path);

    create_terms_table(&db_path);

    db_path
}
pub fn remove_test_db(db_name: &str) {
    std::fs::remove_file(db_name).unwrap_or_else(|_| {
        panic!("Failed to delete database {}", db_name);
    });
}

pub fn assert_term_exists(db_name: &str, term: &str, language: &str) {
    let retrieved_terms = search_terms(db_name, term, language);
    assert!(retrieved_terms.is_ok());
    let terms = retrieved_terms.unwrap();
    assert_eq!(terms.len(), 1);
    let retrieved_term = &terms[0];
    assert_eq!(
        retrieved_term.term_language_set.term.as_ref().unwrap(),
        term
    );
}

#[test]
fn test_create_tables() {
    let db_name = create_test_db("test_create_tables");

    let conn = connect_db(&db_name);
    let termsets_table_exists = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='terms'")
        .unwrap()
        .query_row([], |_| Ok(true))
        .unwrap();

    assert!(termsets_table_exists);

    remove_test_db(&db_name);
}

#[test]
fn test_insert_and_retrieve_termset() {
    let db_name = create_test_db("test_insert_and_retrieve_termset");

    let insert_result = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result.is_ok());

    assert_term_exists(&db_name, "term_1", "en");
    remove_test_db(&db_name);
}

#[test]
fn test_db_get_term_set_id_by_term_and_language() {
    let db_name = create_test_db("test_db_get_term_id_by_term");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    assert_term_exists(&db_name, "term_1", "en");
    let term_id = get_term_set_id(&db_name, "term_1", "en");
    assert!(term_id.is_ok());
    assert_eq!(term_id.unwrap().unwrap(), 1);

    remove_test_db(&db_name);
}

#[test]
fn test_db_get_term_set_id_by_term_id() {
    let db_name = create_test_db("test_db_get_term_set_id_by_term_id");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    assert_term_exists(&db_name, "term_1", "en");
    let term_set_id = get_term_set_id_by_term_id(&db_name, 1);
    assert!(term_set_id.is_ok());
    assert_eq!(term_set_id.unwrap().unwrap(), 1);

    remove_test_db(&db_name);
}

#[test]
fn test_db_add_termset_to_term() {
    let db_name = create_test_db("test_db_add_termset_to_term");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    assert_term_exists(&db_name, "term_1", "en");

    let term_id_1 = get_term_set_id(&db_name, "term_1", "en").unwrap().unwrap();
    assert_eq!(term_id_1, 1);

    let result_db_add_termset_to_term = add_term_to_term_set(&db_name, term_id_1, &TERM_SET_2);

    println!(
        "Debug: result_db_add_termset_to_term = {:?}",
        result_db_add_termset_to_term
    );
    assert!(result_db_add_termset_to_term.is_ok());

    remove_test_db(&db_name);
}

#[test]
fn test_db_update_termset() {
    let db_name = create_test_db("test_db_update_termset");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    assert_term_exists(&db_name, "term_1", "en");

    let term_id_1 = get_term_set_id(&db_name, "term_1", "en").unwrap().unwrap();
    assert_eq!(term_id_1, 1);

    let result_db_update_termset = update_term(&db_name, term_id_1, &TERM_SET_2);

    println!(
        "Debug: result_db_add_termset_to_term = {:?}",
        result_db_update_termset
    );
    assert!(result_db_update_termset.is_ok());
    assert_term_exists(&db_name, "term_2", "nl");

    remove_test_db(&db_name);
}

#[test]
fn test_db_get_data_all() {
    let db_name = create_test_db("test_db_get_data_all");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    let insert_result_2 = add_term(&db_name, &TERM_SET_2);
    assert!(insert_result_2.is_ok());

    let all_dictionary_data = get_all_terms(&db_name);
    assert!(all_dictionary_data.is_ok());
    let terms = all_dictionary_data.unwrap();
    assert_eq!(terms.len(), 2);

    remove_test_db(&db_name);
}

#[test]
fn test_db_get_data_select_term_fuzzy() {
    let db_name = create_test_db("test_db_get_data_select_term_fuzzy");

    let insert_result_1 = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result_1.is_ok());

    assert_term_exists(&db_name, "term_1", "en");

    let insert_result_2 = add_term(&db_name, &TERM_SET_2);
    assert!(insert_result_2.is_ok());

    assert_term_exists(&db_name, "term_2", "nl");

    let data_by_select_term_fuzzy = search_terms(&db_name, "erm", "nl");
    assert!(data_by_select_term_fuzzy.is_ok());
    let terms = data_by_select_term_fuzzy.unwrap();
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].term_language_set.term, Some("term_2".to_string()));

    remove_test_db(&db_name);
}

#[test]
fn test_db_delete_termset() {
    let db_name = create_test_db("test_db_delete_termset");

    let insert_result = add_term(&db_name, &TERM_SET_1);
    assert!(insert_result.is_ok());

    assert_term_exists(&db_name, "term_1", "en");

    let term_id = get_term_set_id(&db_name, "term_1", "en").unwrap().unwrap();
    let delete_result = delete_term(&db_name, term_id);
    assert!(delete_result.is_ok());

    let retrieved_terms = search_terms(&db_name, "term_1", "en");
    assert!(retrieved_terms.is_ok());
    let terms = retrieved_terms.unwrap();
    assert_eq!(terms.len(), 0);

    remove_test_db(&db_name);
}
