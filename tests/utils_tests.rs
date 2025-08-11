use pathbrain::utils::resolve_path;
use std::env;
use std::fs;

#[test]
fn test_resolve_path() {
    let temp_dir = env::temp_dir().join("pathbrain_test");
    fs::create_dir_all(&temp_dir).unwrap();

    let path = resolve_path(Some(temp_dir.to_string_lossy().to_string())).unwrap();
    assert!(path.starts_with('/'));

    let path = resolve_path(None).unwrap();
    assert!(path.starts_with('/'));

    let home = env::var("HOME").unwrap();
    let path = resolve_path(Some("~/".to_string())).unwrap();
    assert!(path.starts_with(&home));

    fs::remove_dir(&temp_dir).unwrap();
} 