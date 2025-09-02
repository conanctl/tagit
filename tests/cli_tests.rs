use tagit::cli::{Cli, Commands};
use clap::Parser;

fn parse_args(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn test_add_command() {
    let cli = parse_args(&["tag", "add", "/test/path", "need to implement caching"]);
    match cli.command {
        Commands::Add { path, message } => {
            assert_eq!(path, "/test/path");
            assert_eq!(message, "need to implement caching");
        }
        _ => panic!("Expected Add command"),
    }

    let cli = parse_args(&["tag", "add", ".", "work in progress"]);
    match cli.command {
        Commands::Add { path, message } => {
            assert_eq!(path, ".");
            assert_eq!(message, "work in progress");
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_ls_command() {
    let cli = parse_args(&["tag", "ls"]);
    match cli.command {
        Commands::Ls { pattern } => {
            assert!(pattern.is_none());
        }
        _ => panic!("Expected Ls command"),
    }

    let cli = parse_args(&["tag", "ls", "filter"]);
    match cli.command {
        Commands::Ls { pattern } => {
            assert_eq!(pattern.unwrap(), "filter");
        }
        _ => panic!("Expected Ls command"),
    }
}

#[test]
fn test_rm_command() {
    let cli = parse_args(&["tag", "rm", "need to implement caching", "-p", "/test/path"]);
    match cli.command {
        Commands::Rm { path, tags, fuzzy } => {
            assert_eq!(path, Some("/test/path".to_string()));
            assert_eq!(tags, Some("need to implement caching".to_string()));
            assert!(!fuzzy);
        }
        _ => panic!("Expected Rm command"),
    }

    let cli = parse_args(&["tag", "rm"]);
    match cli.command {
        Commands::Rm { path, tags, fuzzy } => {
            assert!(path.is_none());
            assert!(tags.is_none());
            assert!(!fuzzy);
        }
        _ => panic!("Expected Rm command"),
    }
}

#[test]
fn test_special_characters_in_args() {
    let cli = parse_args(&["tag", "add", "/path with spaces/", "tag with spaces and @#$% symbols"]);
    match cli.command {
        Commands::Add { path, message } => {
            assert_eq!(path, "/path with spaces/");
            assert_eq!(message, "tag with spaces and @#$% symbols");
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_unicode_in_args() {
    let cli = parse_args(&["tag", "add", "/路径/パス/경로", "标签 タグ 태그"]);
    match cli.command {
        Commands::Add { path, message } => {
            assert_eq!(path, "/路径/パス/경로");
            assert_eq!(message, "标签 タグ 태그");
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_long_arguments() {
    let long_path = "/".repeat(100);
    let long_tag = "a".repeat(100);
    
    let cli = parse_args(&["tag", "add", &long_path, &long_tag]);
    match cli.command {
        Commands::Add { path, message } => {
            assert_eq!(path, long_path);
            assert_eq!(message, long_tag);
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_command_error_cases() {
    assert!(Cli::try_parse_from(&["tag", "add"]).is_err());
    assert!(Cli::try_parse_from(&["tag", "list"]).is_err());
    assert!(Cli::try_parse_from(&["tag", "remove"]).is_err());
}

#[test]
fn test_jump_command() {
    let cli = parse_args(&["tag", "jump"]);
    match cli.command {
        Commands::Jump { pattern } => {
            assert!(pattern.is_none());
        }
        _ => panic!("Expected Jump command"),
    }

    let cli = parse_args(&["tag", "jump", "project"]);
    match cli.command {
        Commands::Jump { pattern } => {
            assert_eq!(pattern.unwrap(), "project");
        }
        _ => panic!("Expected Jump command"),
    }
} 