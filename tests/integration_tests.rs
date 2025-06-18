use tagit::db;
use tagit::utils::now;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_test_env() -> rusqlite::Connection {
    INIT.call_once(|| {});
    db::initialize_db().unwrap()
}

#[test]
fn test_tag_workflow() {
    let mut conn = setup_test_env();
    let path = "/test/workflow/path";
    let path_str = path.to_string();
    
    let tags = vec!["workflow1".to_string(), "workflow2".to_string()];
    db::create_path_tag_entry(&mut conn, &path_str, &tags, now()).unwrap();

    let path_record = db::get_path(&conn, &path_str).unwrap().unwrap();
    assert_eq!(path_record.path, path_str);

    let stored_tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(stored_tags.len(), 2);
    assert!(stored_tags.contains(&"workflow1".to_string()));
    assert!(stored_tags.contains(&"workflow2".to_string()));

    let more_tags = vec!["workflow3".to_string()];
    db::create_path_tag_entry(&mut conn, &path_str, &more_tags, now()).unwrap();

    let final_tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(final_tags.len(), 3);
    assert!(final_tags.contains(&"workflow3".to_string()));
}

#[test]
fn test_search_workflow() {
    let mut conn = setup_test_env();
    let ts = now();

    let test_data = vec![
        ("/path1", vec!["common", "tag1"]),
        ("/path2", vec!["common", "tag2"]),
        ("/path3", vec!["unique", "tag3"]),
    ];

    for (path, tags) in test_data {
        db::create_path_tag_entry(
            &mut conn,
            path,
            &tags.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            ts,
        ).unwrap();
    }

    let common_paths = db::find_paths_by_tag(&conn, "common").unwrap();
    assert_eq!(common_paths.len(), 2);
    assert!(common_paths.iter().any(|p| p.path.ends_with("path1")));
    assert!(common_paths.iter().any(|p| p.path.ends_with("path2")));

    let unique_paths = db::find_paths_by_tag(&conn, "unique").unwrap();
    assert_eq!(unique_paths.len(), 1);
    assert!(unique_paths[0].path.ends_with("path3"));

    let nonexistent = db::find_paths_by_tag(&conn, "nonexistent").unwrap();
    assert!(nonexistent.is_empty());
}

#[test]
fn test_remove_workflow() {
    let mut conn = setup_test_env();
    let ts = now();

    let path = "/test/remove/path";
    let path_str = path.to_string();
    
    let initial_tags = vec!["tag1", "tag2", "tag3", "tag4"];
    db::create_path_tag_entry(
        &mut conn,
        &path_str,
        &initial_tags.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        ts,
    ).unwrap();

    let remove_tags = vec!["tag1".to_string(), "tag3".to_string()];
    db::remove_tags_from_path(&mut conn, &path_str, &remove_tags).unwrap();

    let path_record = db::get_path(&conn, &path_str).unwrap().unwrap();
    let remaining_tags = db::get_tags_for_path(&conn, path_record.id.unwrap()).unwrap();
    assert_eq!(remaining_tags.len(), 2);
    assert!(remaining_tags.contains(&"tag2".to_string()));
    assert!(remaining_tags.contains(&"tag4".to_string()));

    db::remove_path(&mut conn, &path_str).unwrap();
    assert!(!db::path_exists(&conn, &path_str).unwrap());
}

#[test]
fn test_list_workflow() {
    let mut conn = setup_test_env();
    let ts = now();

    let test_paths = vec![
        "/home/user/docs",
        "/home/user/downloads",
        "/var/log",
        "/etc/config",
    ];

    for path in &test_paths {
        db::create_path_tag_entry(&mut conn, path, &vec!["test".to_string()], ts).unwrap();
    }

    let all_paths = db::list_paths(&conn).unwrap();
    assert_eq!(all_paths.len(), test_paths.len());
    for path in test_paths {
        assert!(all_paths.iter().any(|p| p.path.ends_with(path)));
    }
}

#[test]
fn test_error_workflow() {
    let mut conn = setup_test_env();
    let ts = now();

    assert!(db::get_path(&conn, "/nonexistent").unwrap().is_none());
    assert!(db::remove_path(&mut conn, "/nonexistent").is_ok());

    let path = "/test/error/path";
    let path_str = path.to_string();
    
    db::create_path_tag_entry(&mut conn, &path_str, &Vec::new(), ts).unwrap();
    assert!(db::path_exists(&conn, &path_str).unwrap());
    
    let nonexistent_tags = vec!["nonexistent1".to_string(), "nonexistent2".to_string()];
    assert!(db::remove_tags_from_path(&mut conn, &path_str, &nonexistent_tags).is_ok());

    let result = db::create_path_tag_entry(&mut conn, &path_str, &vec!["tag".to_string()], ts);
    assert!(result.is_ok());
}

#[test]
fn test_usage_tracking_workflow() {
    let mut conn = setup_test_env();
    let path = "/test/usage/path";
    let path_str = path.to_string();
    let initial_ts = now();

    db::create_path_tag_entry(&mut conn, &path_str, &vec!["test".to_string()], initial_ts).unwrap();
    let initial = db::get_path(&conn, &path_str).unwrap().unwrap();
    assert_eq!(initial.freq, 1);
    assert_eq!(initial.last_used, initial_ts);

    let ts1 = initial_ts + 100;
    let ts2 = ts1 + 100;
    
    db::bump_path_usage(&mut conn, &path_str, ts1).unwrap();
    db::bump_path_usage(&mut conn, &path_str, ts2).unwrap();

    let final_path = db::get_path(&conn, &path_str).unwrap().unwrap();
    assert_eq!(final_path.freq, 3);
    assert_eq!(final_path.last_used, ts2);
} 