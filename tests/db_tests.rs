use tagit::db;
use tagit::utils::now;
use tagit::error::TagItError;
use rusqlite::Connection;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_test_env() -> Connection {
    INIT.call_once(|| {});
    db::initialize_db().unwrap()
}

#[test]
fn test_create_and_get_path() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts = now();

    db::create_path_tag_entry(&mut conn, path, &vec![], ts).unwrap();
    let fetched = db::get_path(&conn, path).unwrap().unwrap();

    assert_eq!(fetched.path, path);
    assert_eq!(fetched.freq, 1);
    assert_eq!(fetched.last_used, ts);
}

#[test]
fn test_path_exists() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts = now();

    assert!(!db::path_exists(&conn, path).unwrap());
    db::create_path_tag_entry(&mut conn, path, &vec![], ts).unwrap();
    assert!(db::path_exists(&conn, path).unwrap());
}

#[test]
fn test_bump_path_usage() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts1 = now();
    let ts2 = ts1 + 100;

    db::create_path_tag_entry(&mut conn, path, &vec![], ts1).unwrap();
    let initial = db::get_path(&conn, path).unwrap().unwrap();
    assert_eq!(initial.freq, 1);
    assert_eq!(initial.last_used, ts1);

    db::bump_path_usage(&mut conn, path, ts2).unwrap();
    let updated = db::get_path(&conn, path).unwrap().unwrap();
    assert_eq!(updated.freq, 2);
    assert_eq!(updated.last_used, ts2);
}

#[test]
fn test_list_paths() {
    let mut conn = setup_test_env();
    let ts = now();
    let paths = vec!["/path1", "/path2", "/path3"];

    for path in &paths {
        db::create_path_tag_entry(&mut conn, path, &vec![], ts).unwrap();
    }

    let listed = db::list_paths(&conn).unwrap();
    assert_eq!(listed.len(), 3);
    for (i, path) in paths.iter().enumerate() {
        assert_eq!(listed[i].path, *path);
        assert_eq!(listed[i].freq, 1);
        assert_eq!(listed[i].last_used, ts);
    }
}

#[test]
fn test_create_and_get_tags() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts = now();

    db::create_path_tag_entry(&mut conn, path, &vec!["tag1".to_string(), "tag2".to_string()], ts).unwrap();
    let path_record = db::get_path(&conn, path).unwrap().unwrap();
    
    let tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"tag1".to_string()));
    assert!(tags.contains(&"tag2".to_string()));
}

#[test]
fn test_find_paths_by_tag() {
    let mut conn = setup_test_env();
    let ts = now();

    db::create_path_tag_entry(&mut conn, "/path1", &vec!["common".to_string(), "unique1".to_string()], ts).unwrap();
    db::create_path_tag_entry(&mut conn, "/path2", &vec!["common".to_string(), "unique2".to_string()], ts).unwrap();

    let common_paths = db::find_paths_by_tag(&conn, "common").unwrap();
    assert_eq!(common_paths.len(), 2);
    assert!(common_paths.iter().any(|p| p.path == "/path1"));
    assert!(common_paths.iter().any(|p| p.path == "/path2"));

    let unique_paths = db::find_paths_by_tag(&conn, "unique1").unwrap();
    assert_eq!(unique_paths.len(), 1);
    assert_eq!(unique_paths[0].path, "/path1");

    let nonexistent = db::find_paths_by_tag(&conn, "nonexistent").unwrap();
    assert!(nonexistent.is_empty());
}

#[test]
fn test_error_cases() {
    let mut conn = setup_test_env();
    let ts = now();

    assert!(db::get_path(&conn, "/nonexistent").unwrap().is_none());

    let result = db::remove_all_tags_from_path(&mut conn, "/nonexistent");
    assert!(matches!(result.unwrap_err(), TagItError::PathNotFound(_)));

    let empty_tags: Vec<String> = vec![];
    db::create_path_tag_entry(&mut conn, "/test/path", &empty_tags, ts).unwrap();
    let path_record = db::get_path(&conn, "/test/path").unwrap();
    assert!(path_record.is_some());
    let tags = db::get_tags_for_path(&conn, path_record.unwrap().id.unwrap()).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn test_remove_tags_from_path() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts = now();
    let initial_tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];

    db::create_path_tag_entry(&mut conn, path, &initial_tags, ts).unwrap();
    let path_record = db::get_path(&conn, path).unwrap().unwrap();

    let remove_tags = vec!["tag2".to_string()];
    db::remove_tags_from_path(&mut conn, path, &remove_tags).unwrap();
    
    let remaining_tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(remaining_tags.len(), 2);
    assert!(remaining_tags.contains(&"tag1".to_string()));
    assert!(remaining_tags.contains(&"tag3".to_string()));
    assert!(!remaining_tags.contains(&"tag2".to_string()));

    let remove_multiple = vec!["tag1".to_string(), "tag3".to_string()];
    db::remove_tags_from_path(&mut conn, path, &remove_multiple).unwrap();
    
    let final_tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert!(final_tags.is_empty());
}

#[test]
fn test_remove_all_tags_from_path() {
    let mut conn = setup_test_env();
    let path = "/test/path";
    let ts = now();
    let initial_tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];

    db::create_path_tag_entry(&mut conn, path, &initial_tags, ts).unwrap();
    let path_record = db::get_path(&conn, path).unwrap().unwrap();

    let tags_before = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(tags_before.len(), 3);

    db::remove_all_tags_from_path(&mut conn, path).unwrap();
    
    let tags_after = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert!(tags_after.is_empty());
}

#[test]
fn test_tag_removal_error_cases() {
    let mut conn = setup_test_env();
    let path = "/test/error/path";
    let ts = now();

    let nonexistent_path = "/nonexistent/path";
    let result = db::remove_tags_from_path(&mut conn, nonexistent_path, &vec!["tag".to_string()]);
    assert!(matches!(result.unwrap_err(), TagItError::PathNotFound(_)));

    let result = db::remove_all_tags_from_path(&mut conn, nonexistent_path);
    assert!(matches!(result.unwrap_err(), TagItError::PathNotFound(_)));

    db::create_path_tag_entry(&mut conn, path, &vec!["existing".to_string()], ts).unwrap();
    let result = db::remove_tags_from_path(&mut conn, path, &vec!["nonexistent".to_string()]);
    assert!(result.is_ok());

    assert!(db::path_exists(&conn, path).unwrap());
    assert!(!db::path_exists(&conn, nonexistent_path).unwrap());
}

#[test]
fn test_tag_operations() {
    let mut conn = setup_test_env();
    let ts = now();
    let path1 = "/test/tag/path1";
    let path2 = "/test/tag/path2";
    let tags1 = vec!["common".to_string(), "unique1".to_string()];
    let tags2 = vec!["common".to_string(), "unique2".to_string()];

    db::create_path_tag_entry(&mut conn, path1, &tags1, ts).unwrap();
    db::create_path_tag_entry(&mut conn, path2, &tags2, ts).unwrap();

    db::remove_tags_from_path(&mut conn, path1, &vec!["common".to_string()]).unwrap();
    db::remove_tags_from_path(&mut conn, path2, &vec!["common".to_string()]).unwrap();

    let path1_record = db::get_path(&conn, path1).unwrap().unwrap();
    let path2_record = db::get_path(&conn, path2).unwrap().unwrap();

    let path1_tags = db::get_tags_for_path(&conn, path1_record.id.unwrap()).unwrap();
    let path2_tags = db::get_tags_for_path(&conn, path2_record.id.unwrap()).unwrap();

    assert_eq!(path1_tags.len(), 1);
    assert_eq!(path2_tags.len(), 1);
    assert!(path1_tags.contains(&"unique1".to_string()));
    assert!(path2_tags.contains(&"unique2".to_string()));
}

#[test]
fn test_path_cleanup() {
    let mut conn = setup_test_env();
    let ts = now();
    let path = "/test/cleanup/path";

    db::create_path_tag_entry(&mut conn, path, &vec!["tag1".to_string(), "tag2".to_string()], ts).unwrap();
    let path_record = db::get_path(&conn, path).unwrap().unwrap();

    db::remove_all_tags_from_path(&mut conn, path).unwrap();
    assert!(db::path_exists(&conn, path).unwrap());

    let tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert!(tags.is_empty());
}

