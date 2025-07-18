#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::State;
    use lazy_static::lazy_static;
    use term_squire::dictionary::database::*;
    use term_squire::import::parse::*;
    use term_squire::import::process::*;

    const TEST_DB_NAME: &str = "test_db";

    lazy_static! {
        static ref TEST_DB_INFO: Arc<DbInfo> = Arc::new(DbInfo {
            dir: "/data/term-squire-data".to_string(),
            name: TEST_DB_NAME.to_string(),
            table_name: "terms".to_string(),
        });
    }

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
        static ref TERM_SET_3: TermLanguageSet = TermLanguageSet {
            term: Some("term_3".to_string()),
            language: Some("fr".to_string()),
            term_type: None,
            creator_id: Some("user_3".to_string()),
            creation_timestamp: Some(current_epoch()),
            updater_id: None,
            update_timestamp: None,
            subject: None,
            source: None,
            user: None,
            attributes: None,
            remark: None,
            url: None,
            context: Some("context for term_3".to_string()),
            definition: Some("Definition of term_3".to_string()),
        };
        static ref TERM_SET_4: TermLanguageSet = TermLanguageSet {
            term: None,
            language: Some("de".to_string()),
            term_type: Some("verb".to_string()),
            creator_id: None,
            creation_timestamp: Some(current_epoch()),
            updater_id: None,
            update_timestamp: None,
            subject: None,
            source: None,
            user: None,
            attributes: None,
            remark: Some("No term provided".to_string()),
            url: None,
            context: None,
            definition: None,
        };
    }

    fn create_test_db(test_name: &str) -> Arc<DbInfo> {
        let base_dir = "/data/term-squire-data";
        let db_path = format!("{}/{}.sqlite", base_dir, test_name);

        // Remove any existing file (optional safety)
        let _ = std::fs::remove_file(&db_path);

        let db_info = DbInfo {
            dir: base_dir.to_string(),
            name: test_name.to_string(),
            table_name: "terms".to_string(),
        };

        let db_info = Arc::new(db_info);
        let state = State(db_info.clone());

        create_terms_table(state);

        db_info
    }

    fn remove_test_db(db_info: &Arc<DbInfo>) {
        std::fs::remove_file(&db_info.path()).unwrap_or_else(|_| {
            panic!("Failed to delete database {}", db_info.path());
        });
    }

    fn add_term_wrapper(
        db_info: &Arc<DbInfo>,
        term_set: &TermLanguageSet,
    ) -> Result<(), rusqlite::Error> {
        add_term(State(db_info.clone()), term_set)
    }

    fn assert_term_exists(db_info: &Arc<DbInfo>, term: &str, language: &str) {
        let terms = search_terms(State(db_info.clone()), term, language).expect("Search failed");
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].term_language_set.term.as_deref(), Some(term));
    }

    #[test]
    fn test_create_tables() {
        let db_info = create_test_db("test_create_tables");
        let conn = connect_db(State(db_info.clone()));
        let table_exists = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='terms'")
            .unwrap()
            .query_row([], |_| Ok(true))
            .unwrap();

        assert!(table_exists);
        remove_test_db(&db_info);
    }

    #[test]
    fn test_insert_and_retrieve_termset() {
        let db_info = create_test_db("test_insert_and_retrieve_termset");
        let insert_result = add_term_wrapper(&db_info, &TERM_SET_1);
        assert!(insert_result.is_ok());
        assert_term_exists(&db_info, "term_1", "en");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_get_term_set_id_by_term_and_language() {
        let db_info = create_test_db("test_db_get_term_id_by_term");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        assert_term_exists(&db_info, "term_1", "en");

        let term_set_id = get_term_set_id(State(db_info.clone()), "term_1", "en").unwrap();
        assert_eq!(term_set_id.unwrap(), 1);
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_get_term_set_id_by_term_id() {
        let db_info = create_test_db("test_db_get_term_set_id_by_term_id");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        assert_term_exists(&db_info, "term_1", "en");

        let term_set_id = get_term_set_id_by_term_id(State(db_info.clone()), 1).unwrap();
        assert_eq!(term_set_id.unwrap(), 1);
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_add_termset_to_term() {
        let db_info = create_test_db("test_db_add_termset_to_term");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        assert_term_exists(&db_info, "term_1", "en");

        let term_set_id = get_term_set_id(State(db_info.clone()), "term_1", "en")
            .unwrap()
            .unwrap();
        let add_result = add_term_to_term_set(State(db_info.clone()), term_set_id, &TERM_SET_2);
        assert!(add_result.is_ok());
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_update_termset() {
        let db_info = create_test_db("test_db_update_termset");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        assert_term_exists(&db_info, "term_1", "en");

        let term_set_id = get_term_set_id(State(db_info.clone()), "term_1", "en")
            .unwrap()
            .unwrap();
        let update_result = update_term(State(db_info.clone()), term_set_id, &TERM_SET_2);
        assert!(update_result.is_ok());
        assert_term_exists(&db_info, "term_2", "nl");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_get_data_all() {
        let db_info = create_test_db("test_db_get_data_all");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        add_term_wrapper(&db_info, &TERM_SET_2).unwrap();

        let all_terms = get_all_terms(State(db_info.clone())).unwrap();
        assert_eq!(all_terms.len(), 2);
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_get_data_select_term_fuzzy() {
        let db_info = create_test_db("test_db_get_data_select_term_fuzzy");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        add_term_wrapper(&db_info, &TERM_SET_2).unwrap();

        let terms = search_terms(State(db_info.clone()), "erm", "nl").unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].term_language_set.term, Some("term_2".to_string()));
        remove_test_db(&db_info);
    }

    #[test]
    fn test_db_delete_termset() {
        let db_info = create_test_db("test_db_delete_termset");
        add_term_wrapper(&db_info, &TERM_SET_1).unwrap();
        assert_term_exists(&db_info, "term_1", "en");

        let term_set_id = get_term_set_id(State(db_info.clone()), "term_1", "en")
            .unwrap()
            .unwrap();
        delete_term(State(db_info.clone()), term_set_id).unwrap();

        let terms = search_terms(State(db_info.clone()), "term_1", "en").unwrap();
        assert_eq!(terms.len(), 0);

        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_single_term_complete() {
        let db_info = create_test_db("test_process_single_term_complete");
        let result = process_single_term(State(db_info.clone()), &TERM_SET_1);
        assert!(result.is_ok());
        assert_term_exists(&db_info, "term_1", "en");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_single_term_partial() {
        let db_info = create_test_db("test_process_single_term_partial");
        let result = process_single_term(State(db_info.clone()), &TERM_SET_3);
        assert!(result.is_ok());
        assert_term_exists(&db_info, "term_3", "fr");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_two_terms() {
        let db_info = create_test_db("test_process_two_terms");
        let result = process_two_terms(State(db_info.clone()), &TERM_SET_1, &TERM_SET_2);
        assert!(result.is_ok());

        assert_term_exists(&db_info, "term_1", "en");
        assert_term_exists(&db_info, "term_2", "nl");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_two_terms_partial() {
        let db_info = create_test_db("test_process_two_terms_partial");
        let result = process_two_terms(State(db_info.clone()), &TERM_SET_3, &TERM_SET_2);
        assert!(result.is_ok());

        assert_term_exists(&db_info, "term_3", "fr");
        assert_term_exists(&db_info, "term_2", "nl");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_three_or_more_terms() {
        let db_info = create_test_db("test_process_three_or_more_terms");
        let terms = vec![TERM_SET_1.clone(), TERM_SET_2.clone(), TERM_SET_3.clone()];
        let result = process_three_or_more_terms(State(db_info.clone()), &terms);
        assert!(result.is_ok());

        assert_term_exists(&db_info, "term_1", "en");
        assert_term_exists(&db_info, "term_2", "nl");
        assert_term_exists(&db_info, "term_3", "fr");
        remove_test_db(&db_info);
    }

    #[test]
    fn test_process_three_or_more_terms_with_missing_term() {
        let db_info = create_test_db("test_process_three_or_more_terms_with_missing_term");
        let terms = vec![TERM_SET_4.clone(), TERM_SET_2.clone(), TERM_SET_3.clone()];
        let result = process_three_or_more_terms(State(db_info.clone()), &terms);
        assert!(result.is_ok());

        // It should skip TERM_SET_4 because term is None, but still process the others
        assert_term_exists(&db_info, "term_2", "nl");
        assert_term_exists(&db_info, "term_3", "fr");
        remove_test_db(&db_info);
    }
}
