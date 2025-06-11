mod models;
mod operations;
mod schema;

pub use models::PathEntry;
pub use operations::{open_db, create_path_tag_entry, get_path, get_tags_for_path, find_paths_by_tag, list_paths, remove_tags_from_path, remove_all_tags_from_path, path_exists, bump_path_usage, list_all_tags, get_path_id};
pub use schema::initialize_db; 