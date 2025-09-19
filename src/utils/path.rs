use std::path::{Path, PathBuf};
use std::env;
use crate::error::{Result, TagItError};

pub fn resolve_path(path: Option<String>) -> Result<String> {
    let path = match path {
        Some(p) => p,
        None => {
            env::current_dir()?
                .to_string_lossy()
                .into_owned()
        }
    };

    let path = if path.starts_with('~') {
        let home = dirs::home_dir()
            .ok_or_else(|| TagItError::HomeDirNotFound)?;
        let path = path.strip_prefix("~").unwrap();
        let path = path.strip_prefix('/').unwrap_or(path);
        home.join(path)
    } else {
        PathBuf::from(&path)
    };

    let path = if path.is_absolute() {
        path
    } else {
        env::current_dir()?.join(path)
    };

    let path = match path.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => path,
    };

    Ok(path.to_string_lossy().into_owned())
}

pub fn format_path_for_display(path: &str) -> String {
    if let Some(home_dir) = dirs::home_dir() {
        if let Some(home_str) = home_dir.to_str() {
            if path.starts_with(home_str) {
                return path.replacen(home_str, "~", 1);
            }
        }
    }
    path.to_string()
}

pub fn is_git_repo(path: &str) -> bool {
    let path = Path::new(path);
    let mut current = Some(path);

    while let Some(p) = current {
        let git_dir = p.join(".git");
        if git_dir.is_dir() {
            return true;
        }
        current = p.parent();
    }

    false
}

pub fn get_git_root(path: &str) -> Option<String> {
    let path = Path::new(path);
    let mut current = Some(path);

    while let Some(p) = current {
        let git_dir = p.join(".git");
        if git_dir.is_dir() {
            return Some(p.to_string_lossy().into_owned());
        }
        current = p.parent();
    }

    None
} 